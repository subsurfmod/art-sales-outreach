use crate::crm::{self, Lead};
use anyhow::Result;
use futures::stream::{self, StreamExt};
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use regex::Regex;
use rusqlite::params;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

static EMAIL_STRICT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+\-]{1,64}@[a-zA-Z0-9.\-]{1,253}\.[a-zA-Z]{2,12}$").unwrap()
});

const DISPOSABLE_DOMAINS: &[&str] = &[
    "mailinator.com", "guerrillamail.com", "10minutemail.com", "tempmail.com",
    "throwaway.email", "yopmail.com", "trashmail.com", "sharklasers.com",
];

fn category_weight(cat: &str) -> i64 {
    match cat {
        "art_consultant" => 35,
        "interior_designer" => 34,
        "hospitality" => 31,
        "stager" => 30,
        "gallery" => 29,
        "architect" => 27,
        "concept_store" => 25,
        "press" => 24,
        "collector" => 20,
        _ => 12,
    }
}

pub fn detect_provider(mx_hosts: &[String]) -> &'static str {
    let joined = mx_hosts.join(" ").to_lowercase();
    if joined.contains("google") || joined.contains("aspmx") || joined.contains("gmail") {
        "google_workspace"
    } else if joined.contains("outlook")
        || joined.contains("protection.outlook")
        || joined.contains("microsoft")
    {
        "microsoft_365"
    } else if joined.contains("zoho") {
        "zoho"
    } else if joined.contains("protonmail") {
        "proton"
    } else if joined.contains("fastmail") || joined.contains("messagingengine") {
        "fastmail"
    } else if joined.contains("mimecast")
        || joined.contains("pphosted")
        || joined.contains("barracuda")
    {
        "enterprise_gateway"
    } else if joined.contains("secureserver") {
        "godaddy"
    } else {
        "custom_mx"
    }
}

pub fn build_resolver() -> TokioAsyncResolver {
    TokioAsyncResolver::tokio_from_system_conf()
        .unwrap_or_else(|_| TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default()))
}

async fn check_mx(resolver: &TokioAsyncResolver, domain: &str) -> (bool, String) {
    if domain.is_empty() || DISPOSABLE_DOMAINS.contains(&domain) {
        return (false, "disposable".to_string());
    }
    match resolver.mx_lookup(domain).await {
        Ok(lookup) => {
            let hosts: Vec<String> = lookup
                .iter()
                .map(|mx| mx.exchange().to_utf8().trim_end_matches('.').to_lowercase())
                .filter(|h| !h.is_empty() && h != ".")
                .collect();
            if hosts.is_empty() {
                (false, "null_mx".to_string())
            } else {
                (true, detect_provider(&hosts).to_string())
            }
        }
        Err(_) => (false, "no_mx".to_string()),
    }
}

pub fn score_lead(
    lead: &Lead,
    mx_ok: bool,
    artist_styles: &HashSet<String>,
    target_cities: &HashSet<String>,
) -> (i64, BTreeMap<String, i64>) {
    let mut breakdown = BTreeMap::new();

    // 1. Connector Multiplier (0-35)
    let mult = category_weight(&lead.category.to_lowercase());
    breakdown.insert("connector_multiplier".to_string(), mult);

    // 2. Contact Quality (0-25)
    let mut contact_pts = 0i64;
    if mx_ok {
        contact_pts += 10;
    }
    match lead.email_type.to_lowercase().as_str() {
        "personal" => contact_pts += 10,
        "department" => contact_pts += 9,
        "role" => contact_pts += 6,
        _ => {}
    }
    if !lead.contact_name.is_empty() {
        contact_pts += 5;
    }
    let contact_pts = contact_pts.min(25);
    breakdown.insert("contact_quality".to_string(), contact_pts);

    // 3. Personalization & Fit Depth (0-25)
    let mut fit_pts = 0i64;
    if lead.personalization_hook.len() > 25 {
        fit_pts += 10;
    }
    let signals = crm::parse_json_vec(&lead.signals);
    if !signals.is_empty() {
        fit_pts += (4 * signals.len() as i64).min(8);
    }
    let aesthetics: HashSet<String> = crm::parse_json_vec(&lead.aesthetic_tags)
        .into_iter()
        .map(|s| s.to_lowercase())
        .collect();
    if !artist_styles.is_empty() && !aesthetics.is_empty() {
        let overlap = artist_styles.intersection(&aesthetics).count() as i64;
        if overlap > 0 {
            fit_pts += (overlap * 3 + 1).min(7);
        }
    } else if !aesthetics.is_empty() {
        fit_pts += 3;
    }
    let fit_pts = fit_pts.min(25);
    breakdown.insert("personalization_fit".to_string(), fit_pts);

    // 4. Warmth & Connection Potential (0-15)
    let mut warmth_pts = 0i64;
    if !lead.instagram.is_empty() || !lead.linkedin.is_empty() {
        warmth_pts += 5;
    }
    let loc = lead.location.to_lowercase();
    if !target_cities.is_empty() && target_cities.iter().any(|c| loc.contains(c)) {
        warmth_pts += 5;
    }
    if !lead.connection_notes.is_empty() {
        warmth_pts += 5;
    }
    let warmth_pts = warmth_pts.min(15);
    breakdown.insert("warmth_connections".to_string(), warmth_pts);

    let mut total: i64 = breakdown.values().sum();
    if !mx_ok && !lead.email.is_empty() {
        total = total.min(30);
    }
    (total.min(100), breakdown)
}

pub async fn run_verify(
    db_path: &Path,
    profile_path: Option<&Path>,
    skip_dns: bool,
    concurrency: usize,
) -> Result<()> {
    let mut artist_styles = HashSet::new();
    let mut target_cities = HashSet::new();

    if let Some(p) = profile_path {
        if p.exists() {
            let prof: Value = serde_json::from_str(&fs::read_to_string(p)?)?;
            if let Some(arr) = prof["style_keywords"].as_array() {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        artist_styles.insert(s.trim().to_lowercase());
                    }
                }
            }
            if let Some(arr) = prof["target_cities"].as_array() {
                for v in arr {
                    if let Some(s) = v.as_str() {
                        target_cities.insert(s.trim().to_lowercase());
                    }
                }
            }
        }
    }

    let conn = crm::open_conn(db_path)?;
    let mut stmt = conn.prepare("SELECT * FROM leads WHERE status != 'do_not_contact'")?;
    let leads: Vec<Lead> = stmt
        .query_map([], crm::row_to_lead)?
        .filter_map(|r| r.ok())
        .collect();
    drop(stmt);

    let mut unique_domains: HashSet<String> = HashSet::new();
    for l in &leads {
        let em = crm::normalize_email(&l.email);
        if EMAIL_STRICT_RE.is_match(&em) {
            if let Some((_, d)) = em.split_once('@') {
                unique_domains.insert(d.to_string());
            }
        }
    }

    let mut mx_cache: HashMap<String, (bool, String)> = HashMap::new();
    if !skip_dns && !unique_domains.is_empty() {
        let resolver = build_resolver();
        let doms: Vec<String> = unique_domains.into_iter().collect();
        let checked: Vec<(String, (bool, String))> = stream::iter(doms)
            .map(|d| {
                let r = resolver.clone();
                async move {
                    let res = check_mx(&r, &d).await;
                    (d, res)
                }
            })
            .buffer_unordered(concurrency.max(1))
            .collect()
            .await;
        for (d, res) in checked {
            mx_cache.insert(d, res);
        }
    }

    let now = crm::utc_now();
    let mut verified_count = 0usize;
    let mut invalid_count = 0usize;
    let mut high_score = 0usize;

    for lead in &leads {
        let em = crm::normalize_email(&lead.email);
        if !em.is_empty() && crm::is_suppressed(&conn, &em, &lead.domain)? {
            conn.execute(
                "UPDATE leads SET status = 'do_not_contact', updated_at = ?1 WHERE id = ?2",
                params![now, lead.id],
            )?;
            continue;
        }

        let mut mx_ok = false;
        let mut mx_provider = lead.mx_provider.clone();
        let mut mx_val = -1i64;

        if !em.is_empty() {
            if !EMAIL_STRICT_RE.is_match(&em) {
                mx_val = 0;
                mx_provider = "invalid_syntax".to_string();
                invalid_count += 1;
            } else if skip_dns {
                mx_ok = lead.mx_valid == 1;
                mx_val = lead.mx_valid;
            } else {
                let dom = em.split_once('@').map(|(_, d)| d).unwrap_or("");
                let (ok, prov) = mx_cache
                    .get(dom)
                    .cloned()
                    .unwrap_or((false, "no_mx".to_string()));
                mx_ok = ok;
                mx_provider = prov;
                mx_val = if ok { 1 } else { 0 };
                if ok {
                    verified_count += 1;
                } else {
                    invalid_count += 1;
                }
            }
        }

        let (score, breakdown) = score_lead(lead, mx_ok, &artist_styles, &target_cities);
        if score >= 70 && mx_ok {
            high_score += 1;
        }
        let new_status = if matches!(lead.status.as_str(), "new" | "enriched") && mx_val == 1 {
            "verified"
        } else {
            lead.status.as_str()
        };

        conn.execute(
            "UPDATE leads SET
                mx_valid = ?1, mx_provider = ?2, score = ?3, score_breakdown = ?4,
                status = ?5, verified_at = ?6, updated_at = ?6
             WHERE id = ?7",
            params![
                mx_val,
                mx_provider,
                score,
                serde_json::to_string(&breakdown)?,
                new_status,
                now,
                lead.id,
            ],
        )?;
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "leads_scored": leads.len(),
            "unique_domains_checked": mx_cache.len(),
            "mx_verified": verified_count,
            "mx_invalid": invalid_count,
            "ready_high_score_70plus": high_score,
        }))?
    );
    Ok(())
}
