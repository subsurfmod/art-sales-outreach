use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::Subcommand;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

pub const DEFAULT_DB_REL: &str = "art-outreach-workspace/data/art_leads.db";

#[derive(Subcommand, Debug)]
pub enum CrmCommand {
    /// Initialize the SQLite CRM database
    Init,
    /// Import discovered URLs or domains
    ImportUrls {
        /// Text file with one URL per line
        #[arg(long)]
        file: Option<PathBuf>,
        /// URLs passed directly on the CLI
        #[arg(long, num_args = 0..)]
        urls: Vec<String>,
        /// Lead category (interior_designer, art_consultant, gallery, hospitality, stager, architect, concept_store, press, collector)
        #[arg(long, default_value = "unknown")]
        category: String,
        /// City or region tag
        #[arg(long, default_value = "")]
        location: String,
        /// Source label
        #[arg(long, default_value = "search")]
        source: String,
    },
    /// Import leads from a CSV file
    ImportCsv {
        #[arg(long)]
        file: PathBuf,
        #[arg(long, default_value = "unknown")]
        category: String,
    },
    /// Show pipeline statistics
    Stats,
    /// List leads with optional filters
    List {
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        category: Option<String>,
        #[arg(long)]
        min_score: Option<i64>,
        #[arg(long)]
        has_email: bool,
        #[arg(long)]
        mx_valid: bool,
        #[arg(long, default_value_t = 50)]
        limit: i64,
        #[arg(long)]
        json: bool,
    },
    /// Export leads to CSV or JSON
    Export {
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        min_score: Option<i64>,
    },
    /// Add an email or domain to the permanent suppression (do-not-contact) list
    Suppress {
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        domain: Option<String>,
        #[arg(long, default_value = "unsubscribed")]
        reason: String,
    },
    /// Update a lead's status
    MarkStatus {
        #[arg(long)]
        lead_id: Option<i64>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        status: String,
    },
    /// Show leads currently due for follow-up
    NextFollowups {
        #[arg(long, default_value_t = 50)]
        limit: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Lead {
    pub id: i64,
    pub company: String,
    pub contact_name: String,
    pub role: String,
    pub category: String,
    pub email: String,
    pub alt_emails: String,
    pub email_type: String,
    pub mx_valid: i64,
    pub mx_provider: String,
    pub website: String,
    pub domain: String,
    pub instagram: String,
    pub linkedin: String,
    pub pinterest: String,
    pub phone: String,
    pub location: String,
    pub aesthetic_tags: String,
    pub signals: String,
    pub personalization_hook: String,
    pub connection_notes: String,
    pub source_url: String,
    pub score: i64,
    pub score_breakdown: String,
    pub status: String,
    pub sequence_step: i64,
    pub last_contacted_at: String,
    pub next_followup_at: String,
    pub enriched_at: String,
    pub verified_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Default)]
pub struct LeadInput {
    pub company: String,
    pub contact_name: String,
    pub role: String,
    pub category: String,
    pub email: String,
    pub website: String,
    pub domain: String,
    pub instagram: String,
    pub linkedin: String,
    pub pinterest: String,
    pub phone: String,
    pub location: String,
    pub connection_notes: String,
    pub source_url: String,
}

pub fn utc_now() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

pub fn normalize_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let candidate = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let Ok(parsed) = Url::parse(&candidate) else {
        return String::new();
    };
    let Some(host) = parsed.host_str() else {
        return String::new();
    };
    let scheme = match parsed.scheme() {
        "http" => "http",
        _ => "https",
    };
    let path = parsed.path().trim_end_matches('/');
    if path.is_empty() {
        format!("{scheme}://{host}")
    } else {
        format!("{scheme}://{host}{path}")
    }
}

pub fn normalize_domain(raw: &str) -> String {
    let trimmed = raw.trim().to_lowercase();
    if trimmed.is_empty() {
        return String::new();
    }
    let candidate = if trimmed.contains("://") {
        trimmed
    } else {
        format!("https://{trimmed}")
    };
    let Ok(parsed) = Url::parse(&candidate) else {
        return String::new();
    };
    let Some(host) = parsed.host_str() else {
        return String::new();
    };
    host.strip_prefix("www.").unwrap_or(host).to_string()
}

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS leads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    company TEXT DEFAULT '',
    contact_name TEXT DEFAULT '',
    role TEXT DEFAULT '',
    category TEXT DEFAULT 'unknown',
    email TEXT DEFAULT '',
    alt_emails TEXT DEFAULT '[]',
    email_type TEXT DEFAULT 'unknown',
    mx_valid INTEGER DEFAULT -1,
    mx_provider TEXT DEFAULT '',
    website TEXT DEFAULT '',
    domain TEXT DEFAULT '',
    instagram TEXT DEFAULT '',
    linkedin TEXT DEFAULT '',
    pinterest TEXT DEFAULT '',
    phone TEXT DEFAULT '',
    location TEXT DEFAULT '',
    aesthetic_tags TEXT DEFAULT '[]',
    signals TEXT DEFAULT '[]',
    personalization_hook TEXT DEFAULT '',
    connection_notes TEXT DEFAULT '',
    source_url TEXT DEFAULT '',
    score INTEGER DEFAULT 0,
    score_breakdown TEXT DEFAULT '{}',
    status TEXT DEFAULT 'new',
    sequence_step INTEGER DEFAULT 0,
    last_contacted_at TEXT DEFAULT '',
    next_followup_at TEXT DEFAULT '',
    enriched_at TEXT DEFAULT '',
    verified_at TEXT DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_leads_domain_email
    ON leads(domain, email) WHERE domain != '' OR email != '';

CREATE INDEX IF NOT EXISTS idx_leads_status ON leads(status);
CREATE INDEX IF NOT EXISTS idx_leads_score ON leads(score DESC);
CREATE INDEX IF NOT EXISTS idx_leads_category ON leads(category);
CREATE INDEX IF NOT EXISTS idx_leads_email ON leads(email);

CREATE TABLE IF NOT EXISTS campaigns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    artwork_collection TEXT DEFAULT '',
    target_category TEXT DEFAULT '',
    notes TEXT DEFAULT '',
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS outreach_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    lead_id INTEGER NOT NULL,
    campaign_name TEXT DEFAULT 'default',
    sequence_step INTEGER NOT NULL DEFAULT 1,
    sender_email TEXT DEFAULT '',
    recipient_email TEXT NOT NULL,
    subject TEXT NOT NULL,
    body_text TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    message_id TEXT DEFAULT '',
    error_message TEXT DEFAULT '',
    created_at TEXT NOT NULL,
    sent_at TEXT DEFAULT '',
    FOREIGN KEY(lead_id) REFERENCES leads(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_outreach_lead ON outreach_log(lead_id);
CREATE INDEX IF NOT EXISTS idx_outreach_status ON outreach_log(status);

CREATE TABLE IF NOT EXISTS suppression_list (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT UNIQUE,
    domain TEXT,
    reason TEXT NOT NULL DEFAULT 'unsubscribed',
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_suppression_domain ON suppression_list(domain);
"#;

pub fn open_conn(db_path: &Path) -> Result<Connection> {
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(db_path)
        .with_context(|| format!("Failed to open SQLite DB at {}", db_path.display()))?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    Ok(conn)
}

pub fn init_db(db_path: &Path) -> Result<Connection> {
    let conn = open_conn(db_path)?;
    conn.execute_batch(SCHEMA_SQL)?;
    Ok(conn)
}

pub fn row_to_lead(row: &rusqlite::Row<'_>) -> rusqlite::Result<Lead> {
    Ok(Lead {
        id: row.get("id")?,
        company: row.get("company")?,
        contact_name: row.get("contact_name")?,
        role: row.get("role")?,
        category: row.get("category")?,
        email: row.get("email")?,
        alt_emails: row.get("alt_emails")?,
        email_type: row.get("email_type")?,
        mx_valid: row.get("mx_valid")?,
        mx_provider: row.get("mx_provider")?,
        website: row.get("website")?,
        domain: row.get("domain")?,
        instagram: row.get("instagram")?,
        linkedin: row.get("linkedin")?,
        pinterest: row.get("pinterest")?,
        phone: row.get("phone")?,
        location: row.get("location")?,
        aesthetic_tags: row.get("aesthetic_tags")?,
        signals: row.get("signals")?,
        personalization_hook: row.get("personalization_hook")?,
        connection_notes: row.get("connection_notes")?,
        source_url: row.get("source_url")?,
        score: row.get("score")?,
        score_breakdown: row.get("score_breakdown")?,
        status: row.get("status")?,
        sequence_step: row.get("sequence_step")?,
        last_contacted_at: row.get("last_contacted_at")?,
        next_followup_at: row.get("next_followup_at")?,
        enriched_at: row.get("enriched_at")?,
        verified_at: row.get("verified_at")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn is_suppressed(conn: &Connection, email: &str, domain: &str) -> Result<bool> {
    let em = normalize_email(email);
    let mut dom = normalize_domain(domain);
    if dom.is_empty() {
        if let Some((_, d)) = em.split_once('@') {
            dom = d.to_string();
        }
    }
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM suppression_list
             WHERE (email IS NOT NULL AND email != '' AND email = ?1)
                OR (domain IS NOT NULL AND domain != '' AND domain = ?2)
             LIMIT 1",
            params![em, dom],
            |r| r.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

pub fn upsert_lead(conn: &Connection, input: &LeadInput) -> Result<(i64, bool)> {
    let now = utc_now();
    let email = normalize_email(&input.email);
    let website = normalize_url(&input.website);
    let domain = if !input.domain.trim().is_empty() {
        normalize_domain(&input.domain)
    } else if !website.is_empty() {
        normalize_domain(&website)
    } else if let Some((_, d)) = email.split_once('@') {
        d.to_string()
    } else {
        String::new()
    };

    if domain.is_empty() && email.is_empty() {
        bail!("Lead must have at least a website/domain or email");
    }

    let existing: Option<Lead> = if !email.is_empty() {
        conn.query_row(
            "SELECT * FROM leads WHERE email = ?1 LIMIT 1",
            params![email],
            row_to_lead,
        )
        .optional()?
    } else {
        None
    };

    let existing = match existing {
        Some(l) => Some(l),
        None if !domain.is_empty() => conn
            .query_row(
                "SELECT * FROM leads WHERE domain = ?1 LIMIT 1",
                params![domain],
                row_to_lead,
            )
            .optional()?,
        None => None,
    };

    let category = if input.category.trim().is_empty() {
        "unknown".to_string()
    } else {
        input.category.trim().to_lowercase()
    };

    if let Some(old) = existing {
        let pick = |new_v: &str, old_v: &str| -> String {
            if !new_v.trim().is_empty() && (old_v.is_empty() || old_v == "unknown") {
                new_v.trim().to_string()
            } else {
                old_v.to_string()
            }
        };
        conn.execute(
            "UPDATE leads SET
                company = ?1, contact_name = ?2, role = ?3, category = ?4,
                email = ?5, website = ?6, domain = ?7, instagram = ?8,
                linkedin = ?9, pinterest = ?10, phone = ?11, location = ?12,
                connection_notes = ?13, source_url = ?14, updated_at = ?15
             WHERE id = ?16",
            params![
                pick(&input.company, &old.company),
                pick(&input.contact_name, &old.contact_name),
                pick(&input.role, &old.role),
                pick(&category, &old.category),
                pick(&email, &old.email),
                pick(&website, &old.website),
                pick(&domain, &old.domain),
                pick(&input.instagram, &old.instagram),
                pick(&input.linkedin, &old.linkedin),
                pick(&input.pinterest, &old.pinterest),
                pick(&input.phone, &old.phone),
                pick(&input.location, &old.location),
                pick(&input.connection_notes, &old.connection_notes),
                pick(&input.source_url, &old.source_url),
                now,
                old.id,
            ],
        )?;
        return Ok((old.id, false));
    }

    conn.execute(
        "INSERT INTO leads (
            company, contact_name, role, category, email, website, domain,
            instagram, linkedin, pinterest, phone, location, connection_notes,
            source_url, status, created_at, updated_at
        ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,'new',?15,?15)",
        params![
            input.company.trim(),
            input.contact_name.trim(),
            input.role.trim(),
            category,
            email,
            website,
            domain,
            input.instagram.trim(),
            input.linkedin.trim(),
            input.pinterest.trim(),
            input.phone.trim(),
            input.location.trim(),
            input.connection_notes.trim(),
            input.source_url.trim(),
            now,
        ],
    )?;
    Ok((conn.last_insert_rowid(), true))
}

pub fn handle_command(db_path: &Path, cmd: CrmCommand) -> Result<()> {
    match cmd {
        CrmCommand::Init => {
            init_db(db_path)?;
            println!("Initialized CRM database at: {}", db_path.display());
        }
        CrmCommand::ImportUrls {
            file,
            mut urls,
            category,
            location,
            source,
        } => {
            if let Some(f) = file {
                let text = fs::read_to_string(&f)
                    .with_context(|| format!("Failed to read {}", f.display()))?;
                for line in text.lines() {
                    let t = line.trim();
                    if !t.is_empty() && !t.starts_with('#') {
                        urls.push(t.to_string());
                    }
                }
            }
            let conn = init_db(db_path)?;
            let mut added = 0usize;
            let mut merged = 0usize;
            let mut skipped = 0usize;
            for u in urls {
                let norm = normalize_url(&u);
                let dom = normalize_domain(&norm);
                if dom.is_empty() || is_suppressed(&conn, "", &dom)? {
                    skipped += 1;
                    continue;
                }
                let (_, created) = upsert_lead(
                    &conn,
                    &LeadInput {
                        website: norm,
                        domain: dom,
                        category: category.clone(),
                        location: location.clone(),
                        source_url: source.clone(),
                        ..Default::default()
                    },
                )?;
                if created {
                    added += 1;
                } else {
                    merged += 1;
                }
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "added": added, "merged": merged, "skipped": skipped
                }))?
            );
        }
        CrmCommand::ImportCsv { file, category } => {
            let conn = init_db(db_path)?;
            let mut rdr = csv::Reader::from_path(&file)
                .with_context(|| format!("Failed to open CSV {}", file.display()))?;
            let headers: Vec<String> = rdr
                .headers()?
                .iter()
                .map(|h| h.trim().trim_start_matches('\u{feff}').to_lowercase())
                .collect();

            let mut added = 0usize;
            let mut merged = 0usize;
            let mut skipped = 0usize;
            let src_name = file
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("csv")
                .to_string();

            for rec in rdr.records() {
                let rec = rec?;
                let mut map: HashMap<&str, String> = HashMap::new();
                for (i, h) in headers.iter().enumerate() {
                    if let Some(val) = rec.get(i) {
                        map.insert(h.as_str(), val.trim().to_string());
                    }
                }
                let get_any = |keys: &[&str]| -> String {
                    for k in keys {
                        if let Some(v) = map.get(k) {
                            if !v.is_empty() {
                                return v.clone();
                            }
                        }
                    }
                    String::new()
                };
                let email = get_any(&["email", "email_address"]);
                let website = get_any(&["website", "url", "domain"]);
                if email.is_empty() && website.is_empty() {
                    skipped += 1;
                    continue;
                }
                if is_suppressed(&conn, &email, &website)? {
                    skipped += 1;
                    continue;
                }
                let cat = {
                    let c = get_any(&["category"]);
                    if c.is_empty() {
                        category.clone()
                    } else {
                        c
                    }
                };
                let (_, created) = upsert_lead(
                    &conn,
                    &LeadInput {
                        company: get_any(&["company", "studio", "name"]),
                        contact_name: get_any(&["contact_name", "contact", "first_name"]),
                        role: get_any(&["role", "title"]),
                        category: cat,
                        email,
                        website,
                        domain: String::new(),
                        instagram: get_any(&["instagram"]),
                        linkedin: get_any(&["linkedin"]),
                        pinterest: get_any(&["pinterest"]),
                        phone: get_any(&["phone"]),
                        location: get_any(&["location", "city"]),
                        connection_notes: get_any(&["connection_notes", "notes"]),
                        source_url: {
                            let s = get_any(&["source_url"]);
                            if s.is_empty() { src_name.clone() } else { s }
                        },
                    },
                )?;
                if created {
                    added += 1;
                } else {
                    merged += 1;
                }
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "added": added, "merged": merged, "skipped": skipped
                }))?
            );
        }
        CrmCommand::Stats => {
            if !db_path.exists() {
                bail!("Database not found: {}", db_path.display());
            }
            let conn = open_conn(db_path)?;
            let total: i64 = conn.query_row("SELECT COUNT(*) FROM leads", [], |r| r.get(0))?;
            let with_email: i64 = conn.query_row(
                "SELECT COUNT(*) FROM leads WHERE email != ''",
                [],
                |r| r.get(0),
            )?;
            let mx_verified: i64 = conn.query_row(
                "SELECT COUNT(*) FROM leads WHERE mx_valid = 1",
                [],
                |r| r.get(0),
            )?;
            let suppressed: i64 =
                conn.query_row("SELECT COUNT(*) FROM suppression_list", [], |r| r.get(0))?;

            let count_map = |sql: &str| -> Result<BTreeMap<String, i64>> {
                let mut stmt = conn.prepare(sql)?;
                let mut map = BTreeMap::new();
                let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
                for r in rows {
                    let (k, v) = r?;
                    map.insert(k, v);
                }
                Ok(map)
            };

            let out = json!({
                "db": db_path.display().to_string(),
                "total_leads": total,
                "with_email": with_email,
                "mx_verified": mx_verified,
                "suppressed_entries": suppressed,
                "leads_by_status": count_map("SELECT status, COUNT(*) FROM leads GROUP BY status")?,
                "leads_by_category": count_map("SELECT category, COUNT(*) FROM leads GROUP BY category")?,
                "outreach_by_status": count_map("SELECT status, COUNT(*) FROM outreach_log GROUP BY status")?,
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        CrmCommand::List {
            status,
            category,
            min_score,
            has_email,
            mx_valid,
            limit,
            json: as_json,
        } => {
            let conn = open_conn(db_path)?;
            let mut stmt = conn.prepare("SELECT * FROM leads ORDER BY score DESC, id ASC")?;
            let mut leads: Vec<Lead> = stmt
                .query_map([], row_to_lead)?
                .filter_map(|r| r.ok())
                .filter(|l| {
                    if let Some(ref s) = status {
                        if &l.status != s {
                            return false;
                        }
                    }
                    if let Some(ref c) = category {
                        if &l.category != c {
                            return false;
                        }
                    }
                    if let Some(ms) = min_score {
                        if l.score < ms {
                            return false;
                        }
                    }
                    if has_email && l.email.is_empty() {
                        return false;
                    }
                    if mx_valid && l.mx_valid != 1 {
                        return false;
                    }
                    true
                })
                .take(limit.max(0) as usize)
                .collect();

            if as_json {
                println!("{}", serde_json::to_string_pretty(&leads)?);
                return Ok(());
            }
            if leads.is_empty() {
                println!("No matching leads.");
                return Ok(());
            }
            let header = format!(
                "{:>4} {:>3} {:<18} {:<10} {:<26} {:<32} {:<16}",
                "ID", "SCR", "CATEGORY", "STATUS", "COMPANY", "EMAIL", "LOCATION"
            );
            println!("{header}");
            println!("{}", "-".repeat(header.len()));
            for l in leads.drain(..) {
                let comp = if l.company.is_empty() { &l.website } else { &l.company };
                let em = if l.email.is_empty() { "-" } else { &l.email };
                let loc = if l.location.is_empty() { "-" } else { &l.location };
                println!(
                    "{:>4} {:>3} {:<18} {:<10} {:<26} {:<32} {:<16}",
                    l.id,
                    l.score,
                    truncate(&l.category, 18),
                    truncate(&l.status, 10),
                    truncate(comp, 26),
                    truncate(em, 32),
                    truncate(loc, 16)
                );
            }
        }
        CrmCommand::Export {
            output,
            status,
            min_score,
        } => {
            let conn = open_conn(db_path)?;
            let mut stmt = conn.prepare("SELECT * FROM leads ORDER BY score DESC")?;
            let leads: Vec<Lead> = stmt
                .query_map([], row_to_lead)?
                .filter_map(|r| r.ok())
                .filter(|l| {
                    if let Some(ref s) = status {
                        if &l.status != s {
                            return false;
                        }
                    }
                    if let Some(ms) = min_score {
                        if l.score < ms {
                            return false;
                        }
                    }
                    true
                })
                .collect();

            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            if output.extension().and_then(|e| e.to_str()) == Some("json") {
                fs::write(&output, serde_json::to_string_pretty(&leads)?)?;
            } else {
                let mut wtr = csv::Writer::from_path(&output)?;
                for l in &leads {
                    wtr.serialize(l)?;
                }
                wtr.flush()?;
            }
            println!("Exported {} leads to {}", leads.len(), output.display());
        }
        CrmCommand::Suppress {
            email,
            domain,
            reason,
        } => {
            let conn = init_db(db_path)?;
            let em = email.as_deref().map(normalize_email).unwrap_or_default();
            let dom = domain.as_deref().map(normalize_domain).unwrap_or_default();
            if em.is_empty() && dom.is_empty() {
                bail!("Provide --email or --domain");
            }
            let now = utc_now();
            let em_opt: Option<&str> = if em.is_empty() { None } else { Some(&em) };
            conn.execute(
                "INSERT OR IGNORE INTO suppression_list (email, domain, reason, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![em_opt, dom, reason, now],
            )?;
            if !em.is_empty() {
                conn.execute(
                    "UPDATE leads SET status = 'do_not_contact', next_followup_at = '', updated_at = ?1 WHERE email = ?2",
                    params![now, em],
                )?;
            }
            if !dom.is_empty() {
                conn.execute(
                    "UPDATE leads SET status = 'do_not_contact', next_followup_at = '', updated_at = ?1 WHERE domain = ?2",
                    params![now, dom],
                )?;
            }
            println!(
                "Suppressed email={} domain={} reason={}",
                if em.is_empty() { "-" } else { &em },
                if dom.is_empty() { "-" } else { &dom },
                reason
            );
        }
        CrmCommand::MarkStatus {
            lead_id,
            email,
            status,
        } => {
            let conn = open_conn(db_path)?;
            let now = utc_now();
            if let Some(id) = lead_id {
                conn.execute(
                    "UPDATE leads SET status = ?1, updated_at = ?2 WHERE id = ?3",
                    params![status, now, id],
                )?;
            } else if let Some(em) = email {
                conn.execute(
                    "UPDATE leads SET status = ?1, updated_at = ?2 WHERE email = ?3",
                    params![status, now, normalize_email(&em)],
                )?;
            } else {
                bail!("Provide --lead-id or --email");
            }
            println!("Updated status -> {status}");
        }
        CrmCommand::NextFollowups { limit } => {
            let conn = open_conn(db_path)?;
            let now = utc_now();
            let mut stmt = conn.prepare(
                "SELECT id, company, contact_name, email, category, sequence_step, last_contacted_at, next_followup_at
                 FROM leads
                 WHERE status IN ('sent', 'followup_1')
                   AND sequence_step < 3
                   AND next_followup_at != ''
                   AND next_followup_at <= ?1
                 ORDER BY score DESC
                 LIMIT ?2",
            )?;
            let rows: Vec<Value> = stmt
                .query_map(params![now, limit], |r| {
                    Ok(json!({
                        "id": r.get::<_, i64>(0)?,
                        "company": r.get::<_, String>(1)?,
                        "contact_name": r.get::<_, String>(2)?,
                        "email": r.get::<_, String>(3)?,
                        "category": r.get::<_, String>(4)?,
                        "sequence_step": r.get::<_, i64>(5)?,
                        "last_contacted_at": r.get::<_, String>(6)?,
                        "next_followup_at": r.get::<_, String>(7)?,
                    }))
                })?
                .filter_map(|r| r.ok())
                .collect();
            println!("{}", serde_json::to_string_pretty(&rows)?);
        }
    }
    Ok(())
}

pub fn parse_json_vec(raw: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
}

pub fn merge_unique_vecs(a: Vec<String>, b: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for item in a.into_iter().chain(b.into_iter()) {
        if !item.is_empty() && seen.insert(item.clone()) {
            out.push(item);
        }
    }
    out
}

fn truncate(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}
