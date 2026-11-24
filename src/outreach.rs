use crate::crm::{self, Lead};
use crate::imap_client::ImapSession;
use anyhow::{bail, Context, Result};
use chrono::{Duration as ChronoDuration, Utc};
use clap::Subcommand;
use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use rand::Rng;
use regex::Regex;
use reqwest::Client;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Duration;
use uuid::Uuid;

static MULTI_SPACE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[ \t]{2,}").unwrap());
static MULTI_NL_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());
static EMAIL_FIND_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,12}").unwrap()
});

const OPT_OUT_PHRASES: &[&str] = &[
    "unsubscribe", "remove me", "take me off", "stop emailing", "do not contact",
    "opt out", "opt-out", "not interested", "please remove",
];
const BOUNCE_SENDERS: &[&str] = &[
    "mailer-daemon", "postmaster", "mail delivery subsystem", "bounce",
];

#[derive(Subcommand, Debug)]
pub enum OutreachCommand {
    /// Generate personalized Touch 1 / 2 / 3 drafts into CRM and optional Markdown review sheet
    GenerateDrafts {
        #[arg(long)]
        profile: PathBuf,
        #[arg(long)]
        sequences: Option<PathBuf>,
        #[arg(long, default_value = "launch-1")]
        campaign: String,
        #[arg(long, default_value_t = 1)]
        step: i64,
        #[arg(long, default_value = "")]
        category: String,
        #[arg(long, default_value_t = 55)]
        min_score: i64,
        #[arg(long, default_value_t = 25)]
        limit: usize,
        #[arg(long)]
        allow_unverified_mx: bool,
        #[arg(long)]
        review_out: Option<PathBuf>,
    },
    /// Upload pending drafts straight into Gmail/Outlook/Fastmail Drafts folder via IMAP
    UploadImapDrafts {
        #[arg(long)]
        senders: PathBuf,
        #[arg(long, default_value = "launch-1")]
        campaign: String,
        #[arg(long, default_value_t = 25)]
        limit: usize,
        #[arg(long)]
        dry_run: bool,
    },
    /// Send approved drafts using multi-inbox rotated SMTP or Resend/Brevo API
    Send {
        #[arg(long)]
        senders: PathBuf,
        #[arg(long, default_value = "launch-1")]
        campaign: String,
        #[arg(long, default_value_t = 25)]
        limit: usize,
        #[arg(long, default_value_t = 45.0)]
        min_delay: f64,
        #[arg(long, default_value_t = 120.0)]
        max_delay: f64,
        #[arg(long, default_value_t = 4)]
        followup_days_1: i64,
        #[arg(long, default_value_t = 6)]
        followup_days_2: i64,
        #[arg(long)]
        allow_unverified_mx: bool,
        #[arg(long)]
        dry_run: bool,
    },
    /// Scan configured IMAP inboxes for replies, bounces, and opt-outs to auto-halt follow-ups
    CheckReplies {
        #[arg(long)]
        senders: PathBuf,
        #[arg(long, default_value_t = 14)]
        days: i64,
    },
}

#[derive(Debug, Deserialize, Clone)]
pub struct SenderConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(rename = "type", default = "default_smtp")]
    pub kind: String,
    #[serde(default)]
    pub from_name: String,
    pub email: String,
    #[serde(default)]
    pub reply_to: String,
    #[serde(default)]
    pub smtp_host: String,
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
    #[serde(default)]
    pub smtp_user: String,
    #[serde(default)]
    pub smtp_password: String,
    #[serde(default)]
    pub imap_host: String,
    #[serde(default = "default_imap_port")]
    pub imap_port: u16,
    #[serde(default)]
    pub imap_user: String,
    #[serde(default)]
    pub imap_password: String,
    #[serde(default = "default_drafts_folder")]
    pub imap_drafts_folder: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_daily_limit")]
    pub daily_limit: usize,
}

fn default_true() -> bool { true }
fn default_smtp() -> String { "smtp".into() }
fn default_smtp_port() -> u16 { 587 }
fn default_imap_port() -> u16 { 993 }
fn default_drafts_folder() -> String { "[Gmail]/Drafts".into() }
fn default_daily_limit() -> usize { 25 }

#[derive(Debug, Deserialize)]
struct SendersFile {
    senders: Vec<SenderConfig>,
}

#[derive(Debug, Serialize)]
struct DraftSummary {
    draft_id: i64,
    lead_id: i64,
    score: i64,
    category: String,
    company: String,
    to: String,
    step: i64,
    subject: String,
    body: String,
}

#[derive(Debug)]
struct PendingDraftRow {
    id: i64,
    lead_id: i64,
    sequence_step: i64,
    recipient_email: String,
    subject: String,
    body_text: String,
    domain: String,
    mx_valid: i64,
    lead_status: String,
}

fn default_sequences_path() -> PathBuf {
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent().and_then(|p| p.parent()) {
            let cand = parent.join("templates").join("email_sequences.json");
            if cand.exists() {
                return cand;
            }
        }
    }
    PathBuf::from("templates/email_sequences.json")
}

fn resolve_secret(val: &str) -> Result<String> {
    if let Some(var_name) = val.strip_prefix("env:") {
        let key = var_name.trim();
        env::var(key).with_context(|| format!("Environment variable {key} is not set"))
    } else {
        Ok(val.to_string())
    }
}

fn load_senders(path: &Path) -> Result<Vec<SenderConfig>> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("Failed to read senders file {}", path.display()))?;
    let parsed: SendersFile = serde_json::from_str(&raw)?;
    Ok(parsed.senders.into_iter().filter(|s| s.enabled).collect())
}

fn first_name(full: &str) -> String {
    let first = full.split_whitespace().next().unwrap_or("");
    if first.len() >= 2 && first.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        first.to_string()
    } else {
        String::new()
    }
}

fn build_opener(lead: &Lead) -> String {
    let company = if !lead.company.is_empty() {
        lead.company.as_str()
    } else if !lead.domain.is_empty() {
        lead.domain.as_str()
    } else {
        "your studio"
    };
    let tags = crm::parse_json_vec(&lead.aesthetic_tags);
    let signals = crm::parse_json_vec(&lead.signals);

    if signals.iter().any(|s| s == "accepts_submissions") {
        format!("I came across {company} while researching spaces that review new artist portfolios")
    } else if signals.iter().any(|s| s == "art_advisory") {
        let loc_part = if lead.location.is_empty() {
            String::new()
        } else {
            format!(" in {}", lead.location)
        };
        format!("I've been following {company}'s art curation work{loc_part}")
    } else if !tags.is_empty() {
        let style_str = tags.iter().take(2).cloned().collect::<Vec<_>>().join(" and ");
        format!("I really admire the {style_str} sensibility across {company}'s recent projects")
    } else if !lead.location.is_empty() {
        format!("I've been admiring {company}'s work around {}", lead.location)
    } else {
        format!("I came across {company}'s portfolio recently and really enjoyed your aesthetic")
    }
}

fn render_template(tmpl: &str, ctx: &HashMap<&str, String>) -> String {
    let mut out = tmpl.to_string();
    for (k, v) in ctx {
        out = out.replace(&format!("{{{{{k}}}}}"), v);
    }
    let out = MULTI_SPACE_RE.replace_all(&out, " ");
    let out = MULTI_NL_RE.replace_all(&out, "\n\n");
    out.trim().to_string()
}

fn sent_today_count(conn: &rusqlite::Connection, sender_email: &str) -> Result<usize> {
    let prefix = format!("{}%", Utc::now().format("%Y-%m-%d"));
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM outreach_log WHERE sender_email = ?1 AND status = 'sent' AND sent_at LIKE ?2",
        params![sender_email, prefix],
        |r| r.get(0),
    )?;
    Ok(count.max(0) as usize)
}

fn build_lettre_message(
    sender: &SenderConfig,
    recipient: &str,
    subject: &str,
    body: &str,
) -> Result<(Message, String)> {
    let from_mbox: Mailbox = if sender.from_name.is_empty() {
        sender.email.parse()?
    } else {
        format!("{} <{}>", sender.from_name, sender.email).parse()?
    };
    let to_mbox: Mailbox = recipient.parse()?;
    let dom = sender.email.split_once('@').map(|(_, d)| d).unwrap_or("localhost");
    let msg_id = format!("<{}@{dom}>", Uuid::new_v4().simple());

    let mut builder = Message::builder()
        .from(from_mbox)
        .to(to_mbox)
        .subject(subject)
        .message_id(Some(msg_id.clone()))
        .header(ContentType::TEXT_PLAIN);
    if !sender.reply_to.is_empty() {
        builder = builder.reply_to(sender.reply_to.parse()?);
    }
    let msg = builder.body(body.to_string())?;
    Ok((msg, msg_id))
}

async fn send_via_smtp(sender: &SenderConfig, msg: Message) -> Result<()> {
    let user = if sender.smtp_user.is_empty() {
        sender.email.clone()
    } else {
        sender.smtp_user.clone()
    };
    let password = resolve_secret(&sender.smtp_password)?;
    let creds = Credentials::new(user, password);

    let transport = if sender.smtp_port == 465 {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&sender.smtp_host)?
            .port(465)
            .credentials(creds)
            .timeout(Some(Duration::from_secs(25)))
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&sender.smtp_host)?
            .port(sender.smtp_port)
            .credentials(creds)
            .timeout(Some(Duration::from_secs(25)))
            .build()
    };
    transport.send(msg).await?;
    Ok(())
}

async fn send_via_api(
    client: &Client,
    sender: &SenderConfig,
    recipient: &str,
    subject: &str,
    body: &str,
) -> Result<String> {
    let api_key = resolve_secret(&sender.api_key)?;
    match sender.kind.to_lowercase().as_str() {
        "resend" => {
            let from = if sender.from_name.is_empty() {
                sender.email.clone()
            } else {
                format!("{} <{}>", sender.from_name, sender.email)
            };
            let mut payload = json!({
                "from": from,
                "to": [recipient],
                "subject": subject,
                "text": body,
            });
            if !sender.reply_to.is_empty() {
                payload["reply_to"] = json!(sender.reply_to);
            }
            let resp: Value = client
                .post("https://api.resend.com/emails")
                .bearer_auth(api_key)
                .json(&payload)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            Ok(resp["id"].as_str().unwrap_or("").to_string())
        }
        "brevo" => {
            let mut payload = json!({
                "sender": {
                    "name": if sender.from_name.is_empty() { &sender.email } else { &sender.from_name },
                    "email": sender.email,
                },
                "to": [{"email": recipient}],
                "subject": subject,
                "textContent": body,
            });
            if !sender.reply_to.is_empty() {
                payload["replyTo"] = json!({"email": sender.reply_to});
            }
            let resp: Value = client
                .post("https://api.brevo.com/v3/smtp/email")
                .header("api-key", api_key)
                .json(&payload)
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            Ok(resp["messageId"].as_str().unwrap_or("").to_string())
        }
        other => bail!("Unsupported API email provider: {other}"),
    }
}

pub async fn handle_command(db_path: &Path, cmd: OutreachCommand) -> Result<()> {
    match cmd {
        OutreachCommand::GenerateDrafts {
            profile,
            sequences,
            campaign,
            step,
            category,
            min_score,
            limit,
            allow_unverified_mx,
            review_out,
        } => {
            if !(1..=3).contains(&step) {
                bail!("--step must be 1, 2, or 3");
            }
            let prof: Value = serde_json::from_str(&fs::read_to_string(&profile)?)?;
            let seq_path = sequences.unwrap_or_else(default_sequences_path);
            let seqs: Value = serde_json::from_str(
                &fs::read_to_string(&seq_path)
                    .with_context(|| format!("Failed to read sequences {}", seq_path.display()))?,
            )?;

            let conn = crm::open_conn(db_path)?;
            let now = crm::utc_now();

            let mut stmt = conn.prepare("SELECT * FROM leads ORDER BY score DESC, id ASC")?;
            let leads: Vec<Lead> = stmt
                .query_map([], crm::row_to_lead)?
                .filter_map(|r| r.ok())
                .filter(|l| !l.email.is_empty())
                .filter(|l| category.is_empty() || l.category == category)
                .filter(|l| {
                    if step == 1 {
                        matches!(l.status.as_str(), "verified" | "enriched" | "new")
                            && l.sequence_step == 0
                            && l.score >= min_score
                            && (allow_unverified_mx || l.mx_valid == 1)
                    } else {
                        let prev = if step == 2 { "sent" } else { "followup_1" };
                        l.status == prev
                            && l.sequence_step == step - 1
                            && !l.next_followup_at.is_empty()
                            && l.next_followup_at <= now
                    }
                })
                .take(limit)
                .collect();
            drop(stmt);

            let pstr = |k: &str, def: &str| -> String {
                prof[k]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .unwrap_or(def)
                    .to_string()
            };

            let mut created = Vec::new();
            for lead in leads {
                if crm::is_suppressed(&conn, &lead.email, &lead.domain)? {
                    continue;
                }
                let dup: Option<i64> = conn
                    .query_row(
                        "SELECT 1 FROM outreach_log WHERE lead_id = ?1 AND sequence_step = ?2 AND status IN ('draft','imap_draft','sent') LIMIT 1",
                        params![lead.id, step],
                        |r| r.get(0),
                    )
                    .optional()?;
                if dup.is_some() {
                    continue;
                }

                let step_key = format!("touch_{step}");
                let cat_key = lead.category.to_lowercase();
                let tmpl = seqs
                    .get(&cat_key)
                    .and_then(|c| c.get(&step_key))
                    .or_else(|| seqs.get("default").and_then(|c| c.get(&step_key)))
                    .context("Missing template in email_sequences.json")?;

                let fname = first_name(&lead.contact_name);
                let company = if !lead.company.is_empty() {
                    lead.company.clone()
                } else if !lead.domain.is_empty() {
                    lead.domain.clone()
                } else {
                    "your team".to_string()
                };
                let greeting = if !fname.is_empty() {
                    format!("Hi {fname},")
                } else {
                    format!("Hi {company} team,")
                };

                let portfolio_url = pstr("portfolio_url", "");
                let lookbook_url = pstr("lookbook_url", &portfolio_url);
                let artist_name = pstr("artist_name", "");

                let mut ctx: HashMap<&str, String> = HashMap::new();
                ctx.insert("greeting", greeting);
                ctx.insert("first_name", if fname.is_empty() { "there".into() } else { fname });
                ctx.insert("company", company.clone());
                ctx.insert("location", lead.location.clone());
                ctx.insert("custom_opener", build_opener(&lead));
                ctx.insert("personalization_hook", lead.personalization_hook.clone());
                ctx.insert("artist_name", artist_name.clone());
                ctx.insert("studio_name", pstr("studio_name", &artist_name));
                ctx.insert("artist_location", pstr("location", ""));
                ctx.insert("medium", pstr("medium", "original artwork"));
                ctx.insert("style_summary", pstr("style_summary", ""));
                ctx.insert("size_range", pstr("size_range", ""));
                ctx.insert("price_range", pstr("price_range", ""));
                ctx.insert("trade_discount", pstr("trade_discount", "20%"));
                ctx.insert("portfolio_url", portfolio_url);
                ctx.insert("lookbook_url", lookbook_url);
                ctx.insert("instagram_handle", pstr("instagram_handle", ""));
                ctx.insert("signature", pstr("signature", &artist_name));
                ctx.insert(
                    "opt_out_line",
                    pstr(
                        "opt_out_line",
                        "If artwork sourcing isn't on your radar right now, just reply 'no thanks' and I won't follow up.",
                    ),
                );

                let subject = render_template(tmpl["subject"].as_str().unwrap_or(""), &ctx);
                let body = render_template(tmpl["body"].as_str().unwrap_or(""), &ctx);

                conn.execute(
                    "INSERT INTO outreach_log
                        (lead_id, campaign_name, sequence_step, recipient_email, subject, body_text, status, created_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'draft', ?7)",
                    params![lead.id, campaign, step, lead.email, subject, body, now],
                )?;
                let draft_id = conn.last_insert_rowid();
                if step == 1 {
                    conn.execute(
                        "UPDATE leads SET status = 'drafted', updated_at = ?1 WHERE id = ?2",
                        params![now, lead.id],
                    )?;
                }
                created.push(DraftSummary {
                    draft_id,
                    lead_id: lead.id,
                    score: lead.score,
                    category: lead.category,
                    company,
                    to: lead.email,
                    step,
                    subject,
                    body,
                });
            }

            if let Some(ref out_path) = review_out {
                if !created.is_empty() {
                    if let Some(parent) = out_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    let mut md = format!("# Outreach Review Batch — Campaign: {campaign} (Touch {step})\n\n");
                    for d in &created {
                        md.push_str(&format!(
                            "## Draft #{} — {} (`{}`) | Score: {} | {}\n**Subject:** {}\n\n```text\n{}\n```\n\n",
                            d.draft_id, d.company, d.to, d.score, d.category, d.subject, d.body
                        ));
                    }
                    fs::write(out_path, md)?;
                }
            }

            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "campaign": campaign,
                    "step": step,
                    "drafts_created": created.len(),
                    "review_file": review_out.map(|p| p.display().to_string()).unwrap_or_default(),
                    "sample": created.iter().take(2).collect::<Vec<_>>(),
                }))?
            );
        }
        OutreachCommand::UploadImapDrafts {
            senders,
            campaign,
            limit,
            dry_run,
        } => {
            let sender_list = load_senders(&senders)?;
            let Some(sender) = sender_list.into_iter().find(|s| !s.imap_host.is_empty()) else {
                bail!("No enabled sender with imap_host configured in {}", senders.display());
            };
            let conn = crm::open_conn(db_path)?;
            let mut stmt = conn.prepare(
                "SELECT id, lead_id, sequence_step, recipient_email, subject, body_text
                 FROM outreach_log
                 WHERE status = 'draft' AND campaign_name = ?1
                 ORDER BY id ASC LIMIT ?2",
            )?;
            let drafts: Vec<(i64, String, String, String)> = stmt
                .query_map(params![campaign, limit as i64], |r| {
                    Ok((r.get(0)?, r.get(3)?, r.get(4)?, r.get(5)?))
                })?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);

            if drafts.is_empty() {
                println!("{}", serde_json::to_string_pretty(&json!({"uploaded": 0, "message": "No pending drafts."}))?);
                return Ok(());
            }
            if dry_run {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "dry_run": true,
                        "would_upload": drafts.len(),
                        "imap_host": sender.imap_host,
                        "folder": sender.imap_drafts_folder,
                    }))?
                );
                return Ok(());
            }

            let user = if sender.imap_user.is_empty() {
                sender.email.clone()
            } else {
                sender.imap_user.clone()
            };
            let pw_raw = if sender.imap_password.is_empty() {
                &sender.smtp_password
            } else {
                &sender.imap_password
            };
            let pw = resolve_secret(pw_raw)?;

            let mut imap = ImapSession::connect(&sender.imap_host, sender.imap_port, &user, &pw)?;
            let mut uploaded = 0usize;
            for (did, rcpt, subj, body) in drafts {
                let (msg, msg_id) = build_lettre_message(&sender, &rcpt, &subj, &body)?;
                imap.append_draft(&sender.imap_drafts_folder, &msg.formatted())?;
                conn.execute(
                    "UPDATE outreach_log SET status = 'imap_draft', sender_email = ?1, message_id = ?2 WHERE id = ?3",
                    params![sender.email, msg_id, did],
                )?;
                uploaded += 1;
            }
            imap.logout();
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "uploaded_to_drafts": uploaded,
                    "folder": sender.imap_drafts_folder,
                    "account": sender.email,
                }))?
            );
        }
        OutreachCommand::Send {
            senders,
            campaign,
            limit,
            min_delay,
            max_delay,
            followup_days_1,
            followup_days_2,
            allow_unverified_mx,
            dry_run,
        } => {
            let sender_list = load_senders(&senders)?;
            if sender_list.is_empty() {
                bail!("No enabled senders configured in {}", senders.display());
            }

            let conn = crm::open_conn(db_path)?;
            let mut stmt = conn.prepare(
                "SELECT o.id, o.lead_id, o.sequence_step, o.recipient_email, o.subject, o.body_text,
                        l.domain, l.mx_valid, l.status
                 FROM outreach_log o
                 JOIN leads l ON l.id = o.lead_id
                 WHERE o.status = 'draft' AND o.campaign_name = ?1
                 ORDER BY l.score DESC, o.id ASC
                 LIMIT ?2",
            )?;
            let drafts: Vec<PendingDraftRow> = stmt
                .query_map(params![campaign, limit as i64], |r| {
                    Ok(PendingDraftRow {
                        id: r.get(0)?,
                        lead_id: r.get(1)?,
                        sequence_step: r.get(2)?,
                        recipient_email: r.get(3)?,
                        subject: r.get(4)?,
                        body_text: r.get(5)?,
                        domain: r.get(6)?,
                        mx_valid: r.get(7)?,
                        lead_status: r.get(8)?,
                    })
                })?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);

            if drafts.is_empty() {
                println!("{}", serde_json::to_string_pretty(&json!({"sent": 0, "message": "No pending drafts for campaign."}))?);
                return Ok(());
            }

            let mut quotas: HashMap<String, usize> = HashMap::new();
            for s in &sender_list {
                let already = sent_today_count(&conn, &s.email)?;
                quotas.insert(s.email.clone(), s.daily_limit.saturating_sub(already));
            }

            let http = Client::builder().timeout(Duration::from_secs(20)).build()?;
            let mut sender_idx = 0usize;
            let mut sent_ok = 0usize;
            let mut skipped = 0usize;
            let mut errors: Vec<String> = Vec::new();
            let total = drafts.len();

            for (idx, d) in drafts.into_iter().enumerate() {
                if crm::is_suppressed(&conn, &d.recipient_email, &d.domain)?
                    || matches!(d.lead_status.as_str(), "do_not_contact" | "replied" | "bounced")
                {
                    skipped += 1;
                    continue;
                }
                if !allow_unverified_mx && d.mx_valid != 1 {
                    skipped += 1;
                    continue;
                }

                let mut chosen: Option<&SenderConfig> = None;
                for _ in 0..sender_list.len() {
                    let cand = &sender_list[sender_idx % sender_list.len()];
                    sender_idx += 1;
                    if quotas.get(&cand.email).copied().unwrap_or(0) > 0 {
                        chosen = Some(cand);
                        break;
                    }
                }
                let Some(sender) = chosen else {
                    errors.push("All configured sender mailboxes hit their daily warmup limit. Remaining drafts stay queued.".into());
                    break;
                };

                if dry_run {
                    sent_ok += 1;
                    if let Some(q) = quotas.get_mut(&sender.email) {
                        *q = q.saturating_sub(1);
                    }
                    continue;
                }

                let send_res: Result<String> = if sender.kind.eq_ignore_ascii_case("smtp") {
                    match build_lettre_message(sender, &d.recipient_email, &d.subject, &d.body_text) {
                        Ok((msg, mid)) => send_via_smtp(sender, msg).await.map(|_| mid),
                        Err(e) => Err(e),
                    }
                } else {
                    send_via_api(&http, sender, &d.recipient_email, &d.subject, &d.body_text).await
                };

                match send_res {
                    Ok(msg_id) => {
                        let now_dt = Utc::now();
                        let now_str = now_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
                        let (new_status, next_fu) = match d.sequence_step {
                            1 => (
                                "sent",
                                (now_dt + ChronoDuration::days(followup_days_1))
                                    .format("%Y-%m-%dT%H:%M:%SZ")
                                    .to_string(),
                            ),
                            2 => (
                                "followup_1",
                                (now_dt + ChronoDuration::days(followup_days_2))
                                    .format("%Y-%m-%dT%H:%M:%SZ")
                                    .to_string(),
                            ),
                            _ => ("followup_2", String::new()),
                        };

                        conn.execute(
                            "UPDATE outreach_log SET status = 'sent', sender_email = ?1, message_id = ?2, sent_at = ?3 WHERE id = ?4",
                            params![sender.email, msg_id, now_str, d.id],
                        )?;
                        conn.execute(
                            "UPDATE leads SET status = ?1, sequence_step = ?2, last_contacted_at = ?3, next_followup_at = ?4, updated_at = ?3 WHERE id = ?5",
                            params![new_status, d.sequence_step, now_str, next_fu, d.lead_id],
                        )?;
                        if let Some(q) = quotas.get_mut(&sender.email) {
                            *q = q.saturating_sub(1);
                        }
                        sent_ok += 1;

                        if idx + 1 < total && min_delay > 0.0 {
                            let hi = max_delay.max(min_delay);
                            let delay = rand::thread_rng().gen_range(min_delay..=hi);
                            tokio::time::sleep(Duration::from_secs_f64(delay)).await;
                        }
                    }
                    Err(e) => {
                        let msg_err = e.to_string();
                        errors.push(format!("draft#{} ({}): {msg_err}", d.id, d.recipient_email));
                        conn.execute(
                            "UPDATE outreach_log SET status = 'failed', error_message = ?1 WHERE id = ?2",
                            params![msg_err.chars().take(300).collect::<String>(), d.id],
                        )?;
                    }
                }
            }

            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "dry_run": dry_run,
                    "campaign": campaign,
                    "sent": sent_ok,
                    "skipped": skipped,
                    "remaining_daily_quotas": quotas,
                    "errors": errors,
                }))?
            );
        }
        OutreachCommand::CheckReplies { senders, days } => {
            let sender_list: Vec<SenderConfig> = load_senders(&senders)?
                .into_iter()
                .filter(|s| !s.imap_host.is_empty())
                .collect();
            if sender_list.is_empty() {
                bail!("No enabled senders with imap_host configured.");
            }

            let conn = crm::open_conn(db_path)?;
            let mut stmt = conn.prepare(
                "SELECT id, email FROM leads WHERE email != '' AND status IN ('sent', 'followup_1', 'followup_2')",
            )?;
            let known: HashMap<String, i64> = stmt
                .query_map([], |r| Ok((r.get::<_, String>(1)?, r.get::<_, i64>(0)?)))?
                .filter_map(|r| r.ok())
                .collect();
            drop(stmt);

            if known.is_empty() {
                println!("{}", serde_json::to_string_pretty(&json!({"checked": 0, "message": "No active contacted leads awaiting replies."}))?);
                return Ok(());
            }

            let since = (Utc::now() - ChronoDuration::days(days))
                .format("%d-%b-%Y")
                .to_string();
            let now = crm::utc_now();
            let mut replies = Vec::new();
            let mut opt_outs = Vec::new();
            let mut bounces = Vec::new();

            for sender in sender_list {
                let user = if sender.imap_user.is_empty() {
                    sender.email.clone()
                } else {
                    sender.imap_user.clone()
                };
                let pw_raw = if sender.imap_password.is_empty() {
                    &sender.smtp_password
                } else {
                    &sender.imap_password
                };
                let pw = resolve_secret(pw_raw)?;

                let mut imap = ImapSession::connect(&sender.imap_host, sender.imap_port, &user, &pw)?;
                let ids = imap.search_since("INBOX", &since)?;
                let start = ids.len().saturating_sub(300);

                for id in &ids[start..] {
                    let Ok(raw) = imap.fetch_raw_message(id) else {
                        continue;
                    };
                    let (from_hdr, subj) = parse_basic_headers(&raw);
                    let from_low = from_hdr.to_lowercase();
                    let raw_low = raw.to_lowercase();

                    if BOUNCE_SENDERS.iter().any(|b| from_low.contains(b))
                        || subj.to_lowercase().contains("delivery status notification")
                    {
                        for (lead_em, lid) in &known {
                            if raw_low.contains(lead_em) {
                                bounces.push(lead_em.clone());
                                conn.execute(
                                    "UPDATE leads SET status = 'bounced', next_followup_at = '', updated_at = ?1 WHERE id = ?2",
                                    params![now, lid],
                                )?;
                                conn.execute(
                                    "INSERT OR IGNORE INTO suppression_list (email, domain, reason, created_at) VALUES (?1, '', 'bounced', ?2)",
                                    params![lead_em, now],
                                )?;
                            }
                        }
                        continue;
                    }

                    if let Some(m) = EMAIL_FIND_RE.find(&from_low) {
                        let sender_em = m.as_str().to_string();
                        if let Some(&lid) = known.get(&sender_em) {
                            if OPT_OUT_PHRASES.iter().any(|p| raw_low.contains(p)) {
                                opt_outs.push(sender_em.clone());
                                conn.execute(
                                    "UPDATE leads SET status = 'do_not_contact', next_followup_at = '', updated_at = ?1 WHERE id = ?2",
                                    params![now, lid],
                                )?;
                                conn.execute(
                                    "INSERT OR IGNORE INTO suppression_list (email, domain, reason, created_at) VALUES (?1, '', 'opt_out_reply', ?2)",
                                    params![sender_em, now],
                                )?;
                            } else {
                                replies.push(json!({"email": sender_em, "subject": subj}));
                                conn.execute(
                                    "UPDATE leads SET status = 'replied', next_followup_at = '', updated_at = ?1 WHERE id = ?2",
                                    params![now, lid],
                                )?;
                            }
                        }
                    }
                }
                imap.logout();
            }

            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "replies": replies,
                    "opt_outs": opt_outs,
                    "bounces": bounces,
                }))?
            );
        }
    }
    Ok(())
}

fn parse_basic_headers(raw: &str) -> (String, String) {
    let mut from = String::new();
    let mut subject = String::new();
    for line in raw.lines() {
        if line.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            let kl = k.trim().to_lowercase();
            if kl == "from" && from.is_empty() {
                from = v.trim().to_string();
            } else if kl == "subject" && subject.is_empty() {
                subject = v.trim().to_string();
            }
        }
    }
    (from, subject)
}
