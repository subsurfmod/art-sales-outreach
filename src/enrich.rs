use crate::crm::{self, Lead};
use crate::discover::{check_robots_allowed, USER_AGENT};
use anyhow::Result;
use futures::stream::{self, StreamExt};
use regex::Regex;
use reqwest::Client;
use rusqlite::params;
use scraper::{Html, Selector};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;
use url::Url;

static EMAIL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[a-zA-Z0-9._%+\-]{1,64}@[a-zA-Z0-9.\-]{1,253}\.[a-zA-Z]{2,12}\b").unwrap()
});
static OBFUSCATED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b([a-zA-Z0-9._%+\-]{1,40})\s*(?:\[\s*at\s*\]|\(\s*at\s*\))\s*([a-zA-Z0-9.\-]{1,100})\s*(?:\[\s*dot\s*\]|\(\s*dot\s*\)|\.)\s*([a-zA-Z]{2,10})\b").unwrap()
});
static PHONE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:\+?\d{1,3}[\s.\-]?)?(?:\(?\d{3}\)?[\s.\-]?\d{3}[\s.\-]?\d{4})").unwrap()
});
static TITLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(Founder|Co-Founder|Principal|Creative Director|Art Director|Gallery Director|Curator|Head of Design|Managing Partner|Senior Designer|Owner|Lead Designer|Art Advisor|Art Consultant)\b").unwrap()
});
static PERSON_NAME_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b([A-Z][a-z]{1,20}\s+[A-Z][a-z]{1,20})\b").unwrap()
});

static SIGNAL_PATTERNS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    vec![
        ("accepts_submissions", Regex::new(r"(?i)(artist\s+submission|submit\s+your\s+work|portfolio\s+submission|open\s+call|we\s+review\s+portfolios)").unwrap()),
        ("trade_program", Regex::new(r"(?i)(trade\s+program|to\s+the\s+trade|designer\s+trade|trade\s+account)").unwrap()),
        ("art_advisory", Regex::new(r"(?i)(art\s+advisory|art\s+consult|art\s+curat|art\s+procurement|site-specific\s+art|custom\s+commission)").unwrap()),
        ("hospitality_projects", Regex::new(r"(?i)(boutique\s+hotel|hospitality\s+design|restaurant\s+design|resort\s+design)").unwrap()),
        ("luxury_staging", Regex::new(r"(?i)(luxury\s+staging|home\s+staging|model\s+home|real\s+estate\s+staging)").unwrap()),
        ("consignment", Regex::new(r"(?i)(consignment|local\s+artists|featured\s+artists|exhibition)").unwrap()),
    ]
});

const JUNK_PREFIXES: &[&str] = &[
    "noreply", "no-reply", "donotreply", "do-not-reply", "mailer-daemon", "postmaster",
    "abuse", "privacy", "legal", "dmca", "security", "webmaster", "support", "billing",
    "careers", "jobs", "hr", "unsubscribe", "sentry", "example", "test", "user", "email",
];
const JUNK_DOMAINS: &[&str] = &[
    "example.com", "domain.com", "email.com", "wixpress.com", "sentry.io", "squarespace.com",
    "shopify.com", "wordpress.org", "wordpress.com", "w3.org", "schema.org", "googleapis.com",
    "gstatic.com", "cloudflare.com", "gravatar.com", "placeholder.com", "yourdomain.com",
];
const ASSET_EXTS: &[&str] = &[".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp", ".ico", ".css", ".js"];

const ROLE_PREFIXES: &[&str] = &[
    "info", "hello", "contact", "studio", "inquiries", "inquiry", "enquiries", "enquiry",
    "press", "media", "curator", "curatorial", "submissions", "art", "gallery", "projects",
    "design", "trade", "sales", "team", "office", "admin", "newbusiness",
];
const DEPT_PREFIXES: &[&str] = &[
    "curator", "curatorial", "submissions", "art", "trade", "projects", "gallery",
];
const HIGH_VALUE_PATHS: &[&str] = &[
    "contact", "about", "team", "people", "studio", "submissions", "submit",
    "artists", "trade", "projects", "portfolio", "work", "press", "inquiry", "inquiries",
];
const AESTHETIC_KEYWORDS: &[&str] = &[
    "minimalist", "contemporary", "modern", "organic", "coastal", "warm modern",
    "luxury", "bespoke", "editorial", "hospitality", "residential", "commercial",
    "biophilic", "mid-century", "brutalist", "scandinavian", "bohemian", "eclectic",
    "abstract", "figurative", "landscape", "sculptural", "textural", "timeless",
    "boutique", "curated", "sustainable", "artisan", "gallery", "architectural",
];

#[derive(Debug, Default)]
pub struct PageExtract {
    pub title: String,
    pub meta_desc: String,
    pub emails: Vec<String>,
    pub socials: HashMap<String, String>,
    pub phone: String,
    pub aesthetics: Vec<String>,
    pub signals: Vec<String>,
    pub people: Vec<(String, String)>,
    pub headings: Vec<String>,
    pub candidate_links: Vec<String>,
}

pub fn clean_emails(raw_emails: &[String], site_domain: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut valid = Vec::new();
    for raw in raw_emails {
        let em = raw
            .trim()
            .to_lowercase()
            .trim_matches(|c: char| ".,;:<>()[]\"'".contains(c))
            .to_string();
        if em.is_empty() || em.len() > 100 {
            continue;
        }
        if ASSET_EXTS.iter().any(|ext| em.ends_with(ext)) {
            continue;
        }
        let Some((local, dom)) = em.rsplit_once('@') else {
            continue;
        };
        if local.is_empty() || !dom.contains('.') {
            continue;
        }
        if JUNK_PREFIXES.iter().any(|p| {
            local == *p
                || local.starts_with(&format!("{p}+"))
                || local.starts_with(&format!("{p}."))
        }) {
            continue;
        }
        if JUNK_DOMAINS
            .iter()
            .any(|jd| dom == *jd || dom.ends_with(&format!(".{jd}")))
        {
            continue;
        }
        if seen.insert(em.clone()) {
            valid.push(em);
        }
    }

    valid.sort_by_key(|e| {
        let (local, dom) = e.rsplit_once('@').unwrap_or(("", ""));
        let same_dom = if !site_domain.is_empty()
            && (dom == site_domain || dom.ends_with(&format!(".{site_domain}")))
        {
            0
        } else {
            1
        };
        let prio = if DEPT_PREFIXES.contains(&local) {
            0
        } else if !ROLE_PREFIXES.contains(&local) {
            1
        } else if matches!(local, "studio" | "hello" | "inquiries" | "contact" | "design") {
            2
        } else {
            3
        };
        (same_dom, prio, e.clone())
    });
    valid
}

pub fn classify_email(email: &str) -> &'static str {
    let Some((local, _)) = email.split_once('@') else {
        return "none";
    };
    let l = local.to_lowercase();
    if DEPT_PREFIXES.contains(&l.as_str()) {
        "department"
    } else if ROLE_PREFIXES.contains(&l.as_str()) {
        "role"
    } else {
        "personal"
    }
}

pub fn extract_from_html(html: &str, page_url: &str) -> PageExtract {
    let doc = Html::parse_document(html);
    let mut out = PageExtract::default();

    let title_sel = Selector::parse("title").unwrap();
    if let Some(t) = doc.select(&title_sel).next() {
        out.title = collapse_ws(&t.text().collect::<String>())
            .chars()
            .take(140)
            .collect();
    }

    let meta_sel = Selector::parse("meta[name], meta[property]").unwrap();
    for m in doc.select(&meta_sel) {
        let name = m.value().attr("name").unwrap_or("").to_lowercase();
        let prop = m.value().attr("property").unwrap_or("").to_lowercase();
        if name == "description" || prop == "og:description" {
            if let Some(c) = m.value().attr("content") {
                if out.meta_desc.is_empty() {
                    out.meta_desc = collapse_ws(c).chars().take(240).collect();
                }
            }
        }
    }

    // JSON-LD structured data
    let ld_sel = Selector::parse("script[type='application/ld+json']").unwrap();
    for s in doc.select(&ld_sel) {
        let raw = s.text().collect::<String>();
        if let Ok(val) = serde_json::from_str::<Value>(&raw) {
            let items: Vec<&Value> = match &val {
                Value::Array(arr) => arr.iter().collect(),
                obj => vec![obj],
            };
            for obj in items {
                if let Some(em) = obj["email"].as_str() {
                    out.emails.push(em.trim_start_matches("mailto:").to_string());
                }
                let founder = obj.get("founder").or_else(|| obj.get("employee"));
                if let Some(Value::Object(map)) = founder {
                    if let Some(name) = map.get("name").and_then(|v| v.as_str()) {
                        let jt = map
                            .get("jobTitle")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Founder");
                        out.people.push((name.to_string(), jt.to_string()));
                    }
                } else if let Some(Value::Array(arr)) = founder {
                    for f in arr.iter().take(3) {
                        if let Some(name) = f["name"].as_str() {
                            let jt = f["jobTitle"].as_str().unwrap_or("");
                            out.people.push((name.to_string(), jt.to_string()));
                        }
                    }
                }
            }
        }
    }

    let base_url = Url::parse(page_url).ok();
    let base_dom = crm::normalize_domain(page_url);
    let a_sel = Selector::parse("a[href]").unwrap();
    let mut link_seen = HashSet::new();

    for a in doc.select(&a_sel) {
        let Some(href) = a.value().attr("href") else {
            continue;
        };
        let href = href.trim();
        if href.to_lowercase().starts_with("mailto:") {
            let raw = href[7..].split('?').next().unwrap_or("");
            out.emails.push(raw.to_string());
            continue;
        }
        let Some(abs) = base_url.as_ref().and_then(|u| u.join(href).ok()) else {
            continue;
        };
        let abs_str = abs.as_str();
        let lower = abs_str.to_lowercase();
        if lower.contains("instagram.com/") && !lower.contains("/p/") {
            out.socials
                .entry("instagram".to_string())
                .or_insert_with(|| abs_str.split('?').next().unwrap_or(abs_str).to_string());
        } else if lower.contains("linkedin.com/") {
            out.socials
                .entry("linkedin".to_string())
                .or_insert_with(|| abs_str.split('?').next().unwrap_or(abs_str).to_string());
        } else if lower.contains("pinterest.com/") {
            out.socials
                .entry("pinterest".to_string())
                .or_insert_with(|| abs_str.split('?').next().unwrap_or(abs_str).to_string());
        } else if matches!(abs.scheme(), "http" | "https")
            && crm::normalize_domain(abs_str) == base_dom
        {
            let path = abs.path().to_lowercase();
            let trimmed_path = path.trim_matches('/');
            if !trimmed_path.is_empty()
                && HIGH_VALUE_PATHS.iter().any(|k| trimmed_path.contains(k))
                && !ASSET_EXTS.iter().any(|ext| trimmed_path.ends_with(ext))
            {
                let clean_u = abs_str
                    .split('#')
                    .next()
                    .unwrap_or(abs_str)
                    .split('?')
                    .next()
                    .unwrap_or(abs_str)
                    .to_string();
                if link_seen.insert(clean_u.clone()) {
                    out.candidate_links.push(clean_u);
                }
            }
        }
    }

    // Extract visible text (ignoring script/style/noscript/svg)
    let mut body_parts = Vec::new();
    for node in doc.root_element().descendants() {
        if let Some(txt) = node.value().as_text() {
            let inside_skip = node.ancestors().any(|anc| {
                anc.value()
                    .as_element()
                    .map(|el| matches!(el.name(), "script" | "style" | "noscript" | "svg"))
                    .unwrap_or(false)
            });
            if !inside_skip {
                let s = txt.trim();
                if !s.is_empty() {
                    body_parts.push(s);
                }
            }
        }
    }
    let body_text = collapse_ws(&body_parts.join(" "));
    let full_text = format!("{} {} {}", out.title, out.meta_desc, body_text);

    for m in EMAIL_RE.find_iter(&body_text) {
        out.emails.push(m.as_str().to_string());
    }
    for cap in OBFUSCATED_RE.captures_iter(&body_text) {
        out.emails.push(format!("{}@{}.{}", &cap[1], &cap[2], &cap[3]));
    }
    if let Some(m) = PHONE_RE.find(&body_text) {
        out.phone = m.as_str().to_string();
    }

    let lower_full = full_text.to_lowercase();
    for kw in AESTHETIC_KEYWORDS {
        if lower_full.contains(kw) {
            out.aesthetics.push((*kw).to_string());
        }
    }
    for (sig_name, pat) in SIGNAL_PATTERNS.iter() {
        if pat.is_match(&full_text) {
            out.signals.push((*sig_name).to_string());
        }
    }

    // Fallback decision-maker extraction near title mentions
    if out.people.is_empty() {
        let block_sel = Selector::parse("h1, h2, h3, h4, p, li").unwrap();
        for el in doc.select(&block_sel) {
            let chunk = collapse_ws(&el.text().collect::<String>());
            if chunk.len() > 5 && chunk.len() < 110 {
                if let Some(tm) = TITLE_RE.find(&chunk) {
                    if let Some(nm) = PERSON_NAME_RE.find(&chunk) {
                        let cand = nm.as_str().to_lowercase();
                        if !matches!(
                            cand.as_str(),
                            "interior design"
                                | "creative director"
                                | "art director"
                                | "gallery director"
                        ) {
                            out.people
                                .push((nm.as_str().to_string(), tm.as_str().to_string()));
                            break;
                        }
                    }
                }
            }
        }
    }

    let h_sel = Selector::parse("h1, h2").unwrap();
    for h in doc.select(&h_sel) {
        let ht = collapse_ws(&h.text().collect::<String>());
        let hl = ht.to_lowercase();
        if (8..=85).contains(&ht.len())
            && !matches!(
                hl.as_str(),
                "contact" | "contact us" | "about" | "about us" | "projects" | "portfolio" | "menu"
            )
        {
            out.headings.push(ht);
            if out.headings.len() >= 5 {
                break;
            }
        }
    }

    out
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

struct EnrichOutcome {
    id: i64,
    ok: bool,
    company: String,
    contact_name: String,
    role: String,
    email: String,
    alt_emails: Vec<String>,
    email_type: String,
    instagram: String,
    linkedin: String,
    pinterest: String,
    phone: String,
    aesthetic_tags: Vec<String>,
    signals: Vec<String>,
    personalization_hook: String,
}

async fn enrich_one(client: &Client, lead: Lead, max_subpages: usize) -> EnrichOutcome {
    let mut out = EnrichOutcome {
        id: lead.id,
        ok: false,
        company: String::new(),
        contact_name: String::new(),
        role: String::new(),
        email: String::new(),
        alt_emails: Vec::new(),
        email_type: "none".to_string(),
        instagram: String::new(),
        linkedin: String::new(),
        pinterest: String::new(),
        phone: String::new(),
        aesthetic_tags: Vec::new(),
        signals: Vec::new(),
        personalization_hook: String::new(),
    };

    let url = crm::normalize_url(if !lead.website.is_empty() {
        &lead.website
    } else {
        &lead.domain
    });
    if url.is_empty() {
        return out;
    }
    let dom = crm::normalize_domain(&url);

    if !check_robots_allowed(client, &url).await {
        return out;
    }

    let Ok(resp) = client.get(&url).timeout(Duration::from_secs(12)).send().await else {
        return out;
    };
    let status = resp.status().as_u16();
    if status >= 400 {
        return out;
    }
    let ctype = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();
    if !ctype.contains("text/html") {
        return out;
    }
    let final_url = resp.url().to_string();
    let Ok(home_html) = resp.text().await else {
        return out;
    };

    let home = extract_from_html(&home_html, &final_url);
    let mut sub_links = home.candidate_links.clone();
    sub_links.sort_by_key(|u| {
        let p = Url::parse(u)
            .map(|x| x.path().to_lowercase())
            .unwrap_or_default();
        const ORDER: &[&str] = &[
            "contact", "submissions", "trade", "team", "about", "projects", "portfolio", "press",
        ];
        ORDER.iter().position(|k| p.contains(k)).unwrap_or(99)
    });
    sub_links.truncate(max_subpages);
    if !sub_links.iter().any(|u| u.to_lowercase().contains("contact")) && max_subpages > 0 {
        if let Ok(base) = Url::parse(&format!("{}/", url.trim_end_matches('/'))) {
            if let Ok(c) = base.join("contact") {
                sub_links.push(c.to_string());
            }
        }
    }

    let mut pages = vec![home];
    for sub in sub_links.into_iter().take(max_subpages) {
        if !check_robots_allowed(client, &sub).await {
            continue;
        }
        if let Ok(sr) = client.get(&sub).timeout(Duration::from_secs(9)).send().await {
            if sr.status().as_u16() == 200 {
                let sct = sr
                    .headers()
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("")
                    .to_lowercase();
                if sct.contains("text/html") {
                    let su = sr.url().to_string();
                    if let Ok(shtml) = sr.text().await {
                        pages.push(extract_from_html(&shtml, &su));
                    }
                }
            }
        }
    }

    let mut all_emails = Vec::new();
    let mut socials: HashMap<String, String> = HashMap::new();
    let mut aesthetics = Vec::new();
    let mut signals = Vec::new();
    let mut people = Vec::new();
    let mut headings = Vec::new();
    let mut phone = String::new();
    let mut title = String::new();
    let mut meta_desc = String::new();

    for p in pages {
        all_emails.extend(p.emails);
        for (k, v) in p.socials {
            socials.entry(k).or_insert(v);
        }
        aesthetics.extend(p.aesthetics);
        signals.extend(p.signals);
        people.extend(p.people);
        headings.extend(p.headings);
        if phone.is_empty() && !p.phone.is_empty() {
            phone = p.phone;
        }
        if title.is_empty() && !p.title.is_empty() {
            title = p.title;
        }
        if meta_desc.is_empty() && !p.meta_desc.is_empty() {
            meta_desc = p.meta_desc;
        }
    }

    let cleaned = clean_emails(&all_emails, &dom);
    let primary_email = cleaned
        .first()
        .cloned()
        .unwrap_or_else(|| lead.email.clone());
    let alt_emails: Vec<String> = cleaned.into_iter().skip(1).take(4).collect();
    let aesthetics: Vec<String> = crm::merge_unique_vecs(Vec::new(), aesthetics)
        .into_iter()
        .take(10)
        .collect();
    let signals = crm::merge_unique_vecs(Vec::new(), signals);

    let company = if !lead.company.is_empty() {
        lead.company
    } else if !title.is_empty() {
        title
            .split('|')
            .next()
            .unwrap_or(&title)
            .split('—')
            .next()
            .unwrap_or(&title)
            .split(" - ")
            .next()
            .unwrap_or(&title)
            .trim()
            .to_string()
    } else {
        dom
    };

    let contact_name = if !lead.contact_name.is_empty() {
        lead.contact_name
    } else {
        people.first().map(|p| p.0.clone()).unwrap_or_default()
    };
    let role = if !lead.role.is_empty() {
        lead.role
    } else {
        people.first().map(|p| p.1.clone()).unwrap_or_default()
    };

    let mut hook_parts = Vec::new();
    if !meta_desc.is_empty() {
        hook_parts.push(meta_desc.chars().take(160).collect::<String>());
    } else if !headings.is_empty() {
        hook_parts.push(
            headings
                .iter()
                .take(2)
                .cloned()
                .collect::<Vec<_>>()
                .join(" | ")
                .chars()
                .take(160)
                .collect(),
        );
    }
    if !aesthetics.is_empty() {
        hook_parts.push(format!(
            "Aesthetic: {}",
            aesthetics.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
        ));
    }
    if !signals.is_empty() {
        hook_parts.push(format!("Signals: {}", signals.join(", ")));
    }
    let hook: String = hook_parts.join(" — ").chars().take(360).collect();

    out.ok = true;
    out.company = company.chars().take(120).collect();
    out.contact_name = contact_name.chars().take(80).collect();
    out.role = role.chars().take(80).collect();
    out.email_type = classify_email(&primary_email).to_string();
    out.email = primary_email;
    out.alt_emails = alt_emails;
    out.instagram = socials.remove("instagram").unwrap_or_default();
    out.linkedin = socials.remove("linkedin").unwrap_or_default();
    out.pinterest = socials.remove("pinterest").unwrap_or_default();
    out.phone = phone.chars().take(40).collect();
    out.aesthetic_tags = aesthetics;
    out.signals = signals;
    out.personalization_hook = hook;
    out
}

pub async fn run_enrich(
    db_path: &Path,
    limit: usize,
    concurrency: usize,
    max_subpages: usize,
    category: &str,
    force: bool,
) -> Result<()> {
    let conn = crm::open_conn(db_path)?;
    let mut stmt = conn.prepare("SELECT * FROM leads WHERE status != 'do_not_contact' ORDER BY id ASC")?;
    let leads: Vec<Lead> = stmt
        .query_map([], crm::row_to_lead)?
        .filter_map(|r| r.ok())
        .filter(|l| force || l.enriched_at.is_empty())
        .filter(|l| category.is_empty() || l.category == category)
        .take(limit)
        .collect();
    drop(stmt);

    if leads.is_empty() {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "processed": 0,
                "message": "No unenriched leads found."
            }))?
        );
        return Ok(());
    }

    let client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(12))
        .build()?;

    let total = leads.len();
    let results: Vec<EnrichOutcome> = stream::iter(leads)
        .map(|lead| {
            let c = client.clone();
            async move { enrich_one(&c, lead, max_subpages).await }
        })
        .buffer_unordered(concurrency.max(1))
        .collect()
        .await;

    let now = crm::utc_now();
    let mut enriched_ok = 0usize;
    let mut emails_found = 0usize;
    let mut failed = 0usize;

    for res in results {
        if !res.ok {
            failed += 1;
            conn.execute(
                "UPDATE leads SET enriched_at = ?1, updated_at = ?1 WHERE id = ?2",
                params![now, res.id],
            )?;
            continue;
        }
        enriched_ok += 1;
        if !res.email.is_empty() {
            emails_found += 1;
        }
        conn.execute(
            "UPDATE leads SET
                company = CASE WHEN ?1 != '' THEN ?1 ELSE company END,
                contact_name = CASE WHEN ?2 != '' THEN ?2 ELSE contact_name END,
                role = CASE WHEN ?3 != '' THEN ?3 ELSE role END,
                email = CASE WHEN ?4 != '' THEN ?4 ELSE email END,
                alt_emails = ?5,
                email_type = ?6,
                instagram = CASE WHEN ?7 != '' THEN ?7 ELSE instagram END,
                linkedin = CASE WHEN ?8 != '' THEN ?8 ELSE linkedin END,
                pinterest = CASE WHEN ?9 != '' THEN ?9 ELSE pinterest END,
                phone = CASE WHEN ?10 != '' THEN ?10 ELSE phone END,
                aesthetic_tags = ?11,
                signals = ?12,
                personalization_hook = CASE WHEN ?13 != '' THEN ?13 ELSE personalization_hook END,
                status = CASE WHEN status = 'new' THEN 'enriched' ELSE status END,
                enriched_at = ?14,
                updated_at = ?14
             WHERE id = ?15",
            params![
                res.company,
                res.contact_name,
                res.role,
                res.email,
                serde_json::to_string(&res.alt_emails)?,
                res.email_type,
                res.instagram,
                res.linkedin,
                res.pinterest,
                res.phone,
                serde_json::to_string(&res.aesthetic_tags)?,
                serde_json::to_string(&res.signals)?,
                res.personalization_hook,
                now,
                res.id,
            ],
        )?;
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "processed": total,
            "enriched_ok": enriched_ok,
            "emails_found": emails_found,
            "failed_or_gated": failed,
        }))?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_emails_jsonld_signals_and_aesthetics() {
        let html = r#"
        <html><head>
        <title>Studio Sol Interiors | Warm Modern Residential Design</title>
        <meta name="description" content="Bespoke minimalist and warm modern interiors in Austin, TX.">
        <script type="application/ld+json">
        {"@type":"Organization","name":"Studio Sol","email":"mailto:elena@studiosol.test","founder":{"name":"Elena Vance","jobTitle":"Principal Designer"}}
        </script>
        </head><body>
        <h1>Warm Modern Residential Projects</h1>
        <p>Contact studio [at] studiosol [dot] test or trade@studiosol.test for our designer trade program and custom art advisory.</p>
        <a href="https://instagram.com/studiosol">IG</a>
        <a href="/contact">Contact</a>
        <img src="logo@2x.png">
        </body></html>"#;

        let ext = extract_from_html(html, "https://studiosol.test");
        let cleaned = clean_emails(&ext.emails, "studiosol.test");
        assert!(cleaned.contains(&"elena@studiosol.test".to_string()));
        assert!(cleaned.contains(&"trade@studiosol.test".to_string()));
        assert!(cleaned.contains(&"studio@studiosol.test".to_string()));
        assert!(!cleaned.iter().any(|e| e.contains("logo@2x.png")));
        assert!(ext.signals.contains(&"trade_program".to_string()));
        assert!(ext.aesthetics.contains(&"minimalist".to_string()));
        assert_eq!(ext.people[0].0, "Elena Vance");
    }
}
