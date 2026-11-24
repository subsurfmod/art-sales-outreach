use crate::crm::{self, LeadInput};
use anyhow::{bail, Context, Result};
use clap::Subcommand;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use url::Url;

pub const USER_AGENT: &str = "ArtOutreachResearchBot/1.0 (+polite public business contact discovery)";

const SKIP_DOMAINS: &[&str] = &[
    "google.com", "facebook.com", "instagram.com", "linkedin.com", "twitter.com", "x.com",
    "pinterest.com", "youtube.com", "tiktok.com", "yelp.com", "houzz.com", "tripadvisor.com",
    "wikipedia.org", "reddit.com", "amazon.com", "etsy.com", "saatchiart.com", "artsy.net",
    "yellowpages.com", "mapquest.com", "bbb.org", "zoominfo.com", "apollo.io", "crunchbase.com",
    "indeed.com", "glassdoor.com", "apple.com", "play.google.com", "vimeo.com", "behance.net",
];

#[derive(Subcommand, Debug)]
pub enum DiscoverCommand {
    /// Generate high-yield Google Boolean search queries tailored to your style and cities
    GenerateQueries {
        /// Comma-separated target cities or regions
        #[arg(long)]
        cities: String,
        /// Comma-separated verticals (default: all 8)
        #[arg(long, default_value = "")]
        verticals: String,
        /// Optional aesthetic style keyword (e.g. minimalist, warm modern, coastal)
        #[arg(long, default_value = "")]
        style: String,
        /// Optional medium keyword (e.g. oil painting, sculpture, ceramic)
        #[arg(long, default_value = "")]
        medium: String,
        /// Optional output JSON file path
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Run search queries via Serper.dev, Brave Search API, or Google Custom Search API
    SearchApi {
        #[arg(long, default_value = "auto")]
        provider: String,
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        queries_file: Option<PathBuf>,
        #[arg(long, default_value = "unknown")]
        category: String,
        #[arg(long, default_value = "")]
        location: String,
        #[arg(long, default_value_t = 15)]
        num: usize,
    },
    /// Pull local galleries, boutique hotels, designers & architects via free OpenStreetMap Overpass API
    OsmLocal {
        #[arg(long)]
        city: String,
        #[arg(long, default_value = "")]
        verticals: String,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// Extract outbound business websites from a public directory page (honors robots.txt)
    CrawlDirectory {
        #[arg(long)]
        url: String,
        #[arg(long, default_value = "unknown")]
        category: String,
        #[arg(long, default_value = "")]
        location: String,
        #[arg(long, default_value_t = 200)]
        limit: usize,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryItem {
    pub vertical: String,
    pub city: String,
    pub query: String,
}

pub fn is_business_domain(raw_url: &str) -> bool {
    let dom = crm::normalize_domain(raw_url);
    if dom.is_empty() || !dom.contains('.') {
        return false;
    }
    !SKIP_DOMAINS
        .iter()
        .any(|s| dom == *s || dom.ends_with(&format!(".{s}")))
}

fn vertical_templates(vertical: &str) -> &'static [&'static str] {
    match vertical {
        "interior_designer" => &[
            "\"{city}\" \"interior design\" studio (\"projects\" OR \"portfolio\") (\"contact\" OR \"inquiries\") -site:houzz.com -site:yelp.com -site:instagram.com",
            "\"{city}\" luxury residential \"interior designer\" (\"art\" OR \"artwork\" OR \"curated\") contact -site:pinterest.com",
            "\"{city}\" \"interior architecture\" firm (\"trade\" OR \"studio\") \"{style}\" -site:linkedin.com",
        ],
        "art_consultant" => &[
            "\"{city}\" (\"art consultant\" OR \"art advisory\" OR \"art curation\") (\"artists\" OR \"submissions\" OR \"contact\")",
            "(\"corporate art\" OR \"hospitality art\") consultant \"{city}\" (\"portfolio\" OR \"projects\")",
            "\"art advisor\" \"{city}\" (\"original artwork\" OR \"commissions\" OR \"collectors\")",
        ],
        "gallery" => &[
            "\"{city}\" \"contemporary art gallery\" (\"submissions\" OR \"artists\" OR \"contact\") \"{medium}\"",
            "\"{city}\" independent \"art gallery\" (\"exhibitions\" OR \"consignment\" OR \"inquiries\")",
        ],
        "hospitality" => &[
            "\"{city}\" \"boutique hotel\" (\"design\" OR \"local art\" OR \"contact\") -site:booking.com -site:expedia.com -site:tripadvisor.com",
            "\"{city}\" \"hospitality design\" firm (\"hotels\" OR \"restaurants\") portfolio contact",
        ],
        "stager" => &[
            "\"{city}\" (\"luxury home staging\" OR \"real estate staging\") (\"portfolio\" OR \"contact\")",
            "\"{city}\" \"model home\" staging design studio contact",
        ],
        "architect" => &[
            "\"{city}\" residential \"architecture studio\" (\"interiors\" OR \"custom homes\") contact \"{style}\"",
        ],
        "concept_store" => &[
            "\"{city}\" (\"design showroom\" OR \"concept store\" OR \"home decor boutique\") (\"makers\" OR \"artists\" OR \"wholesale\" OR \"contact\")",
            "\"{city}\" \"custom framing\" gallery (\"local artists\" OR \"contact\")",
        ],
        "press" => &[
            "\"{city}\" (\"design magazine\" OR \"home magazine\" OR \"art blog\") (\"pitch\" OR \"editorial\" OR \"submissions\" OR \"contact\")",
        ],
        _ => &[],
    }
}

const ALL_VERTICALS: &[&str] = &[
    "interior_designer",
    "art_consultant",
    "gallery",
    "hospitality",
    "stager",
    "architect",
    "concept_store",
    "press",
];

fn osm_filters(vertical: &str) -> &'static [&'static str] {
    match vertical {
        "gallery" => &["[\"tourism\"=\"gallery\"][\"website\"]", "[\"shop\"=\"art\"][\"website\"]"],
        "hospitality" => &["[\"tourism\"=\"hotel\"][\"website\"]"],
        "interior_designer" => &[
            "[\"office\"=\"interior_design\"][\"website\"]",
            "[\"shop\"=\"interior_decoration\"][\"website\"]",
        ],
        "architect" => &["[\"office\"=\"architect\"][\"website\"]"],
        "concept_store" => &["[\"shop\"=\"frame\"][\"website\"]", "[\"shop\"=\"furniture\"][\"website\"]"],
        _ => &[],
    }
}

pub async fn check_robots_allowed(client: &Client, target_url: &str) -> bool {
    let Ok(parsed) = Url::parse(target_url) else {
        return false;
    };
    let Some(host) = parsed.host_str() else {
        return false;
    };
    let robots_url = format!("{}://{}/robots.txt", parsed.scheme(), host);
    let Ok(resp) = client
        .get(&robots_url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
    else {
        return true;
    };
    if !resp.status().is_success() {
        return true;
    }
    let Ok(body) = resp.text().await else {
        return true;
    };
    let path = parsed.path();
    robots_allows_path(&body, path)
}

pub fn robots_allows_path(robots_txt: &str, path: &str) -> bool {
    let mut active = false;
    let mut disallows: Vec<&str> = Vec::new();
    let mut allows: Vec<&str> = Vec::new();
    for raw_line in robots_txt.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        let key = k.trim().to_lowercase();
        let val = v.trim();
        match key.as_str() {
            "user-agent" => {
                let ua = val.to_lowercase();
                active = ua == "*" || ua.contains("artoutreachresearchbot");
            }
            "disallow" if active && !val.is_empty() => disallows.push(val),
            "allow" if active && !val.is_empty() => allows.push(val),
            _ => {}
        }
    }
    let longest_disallow = disallows
        .iter()
        .filter(|d| path.starts_with(*d))
        .map(|d| d.len())
        .max()
        .unwrap_or(0);
    let longest_allow = allows
        .iter()
        .filter(|a| path.starts_with(*a))
        .map(|a| a.len())
        .max()
        .unwrap_or(0);
    longest_allow >= longest_disallow
}

pub async fn handle_command(db_path: &Path, cmd: DiscoverCommand) -> Result<()> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(20))
        .build()?;

    match cmd {
        DiscoverCommand::GenerateQueries {
            cities,
            verticals,
            style,
            medium,
            output,
        } => {
            let city_list: Vec<String> = cities
                .split(',')
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty())
                .collect();
            let vert_list: Vec<String> = if verticals.trim().is_empty() {
                ALL_VERTICALS.iter().map(|s| s.to_string()).collect()
            } else {
                verticals
                    .split(',')
                    .map(|v| v.trim().to_lowercase())
                    .filter(|v| !v.is_empty())
                    .collect()
            };
            let empty_quote = Regex::new(r#"""\s*"#)?;
            let multi_ws = Regex::new(r"\s{2,}")?;

            let mut queries = Vec::new();
            for city in &city_list {
                for vert in &vert_list {
                    for tmpl in vertical_templates(vert) {
                        let q = tmpl
                            .replace("{city}", city)
                            .replace("{style}", &style)
                            .replace("{medium}", &medium);
                        let q = empty_quote.replace_all(&q, "");
                        let q = multi_ws.replace_all(&q, " ").trim().to_string();
                        queries.push(QueryItem {
                            vertical: vert.clone(),
                            city: city.clone(),
                            query: q,
                        });
                    }
                }
            }
            let out_json = serde_json::to_string_pretty(&queries)?;
            if let Some(out_path) = output {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&out_path, &out_json)?;
                println!(
                    "Wrote {} targeted search queries to {}",
                    queries.len(),
                    out_path.display()
                );
            } else {
                println!("{out_json}");
            }
        }
        DiscoverCommand::SearchApi {
            provider,
            query,
            queries_file,
            category,
            location,
            num,
        } => {
            let resolved_provider = if provider == "auto" {
                if env::var("SERPER_API_KEY").is_ok() {
                    "serper".to_string()
                } else if env::var("BRAVE_API_KEY").is_ok() {
                    "brave".to_string()
                } else if env::var("GOOGLE_CSE_API_KEY").is_ok() && env::var("GOOGLE_CSE_CX").is_ok() {
                    "google_cse".to_string()
                } else {
                    bail!(
                        "No search API key found (SERPER_API_KEY, BRAVE_API_KEY, or GOOGLE_CSE_API_KEY+GOOGLE_CSE_CX). Use `discover generate-queries` with the agent's search_web tool or `discover osm-local`."
                    );
                }
            } else {
                provider
            };

            let items: Vec<QueryItem> = if let Some(qf) = queries_file {
                serde_json::from_str(&fs::read_to_string(&qf)?)?
            } else if let Some(q) = query {
                vec![QueryItem {
                    vertical: category.clone(),
                    city: location.clone(),
                    query: q,
                }]
            } else {
                bail!("Provide --query or --queries-file");
            };

            let conn = crm::init_db(db_path)?;
            let mut added = 0usize;
            let mut merged = 0usize;
            let mut total_hits = 0usize;

            for item in &items {
                let hits = run_search_provider(&client, &resolved_provider, &item.query, num).await;
                let Ok(hits) = hits else {
                    eprintln!("[warn] query failed: {}", item.query);
                    continue;
                };
                total_hits += hits.len();
                for (title, link) in hits {
                    let dom = crm::normalize_domain(&link);
                    if dom.is_empty() || crm::is_suppressed(&conn, "", &dom)? {
                        continue;
                    }
                    let company = title
                        .split('|')
                        .next()
                        .unwrap_or(&title)
                        .split(" - ")
                        .next()
                        .unwrap_or(&title)
                        .trim()
                        .chars()
                        .take(120)
                        .collect();
                    let (_, created) = crm::upsert_lead(
                        &conn,
                        &LeadInput {
                            company,
                            website: format!("https://{dom}"),
                            domain: dom,
                            category: item.vertical.clone(),
                            location: item.city.clone(),
                            source_url: link,
                            ..Default::default()
                        },
                    )?;
                    if created {
                        added += 1;
                    } else {
                        merged += 1;
                    }
                }
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "provider": resolved_provider,
                    "queries_run": items.len(),
                    "hits": total_hits,
                    "added": added,
                    "merged": merged,
                }))?
            );
        }
        DiscoverCommand::OsmLocal {
            city,
            verticals,
            limit,
        } => {
            let places: Vec<Value> = client
                .get("https://nominatim.openstreetmap.org/search")
                .query(&[("q", city.as_str()), ("format", "json"), ("limit", "1")])
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            let Some(first) = places.first() else {
                bail!("City not found in OpenStreetMap Nominatim: {city}");
            };
            let bb = first["boundingbox"]
                .as_array()
                .context("Missing boundingbox in Nominatim response")?;
            let bbox = format!(
                "{},{},{},{}",
                bb[0].as_str().unwrap_or("0"),
                bb[2].as_str().unwrap_or("0"),
                bb[1].as_str().unwrap_or("0"),
                bb[3].as_str().unwrap_or("0"),
            );

            let vert_list: Vec<String> = if verticals.trim().is_empty() {
                vec![
                    "gallery".into(),
                    "hospitality".into(),
                    "interior_designer".into(),
                    "architect".into(),
                    "concept_store".into(),
                ]
            } else {
                verticals
                    .split(',')
                    .map(|v| v.trim().to_lowercase())
                    .filter(|v| !v.is_empty())
                    .collect()
            };

            let mut selectors = Vec::new();
            for v in &vert_list {
                for f in osm_filters(v) {
                    selectors.push(format!("node{f}({bbox});"));
                    selectors.push(format!("way{f}({bbox});"));
                }
            }
            let ql = format!(
                "[out:json][timeout:25];(\n{}\n);out center tags;",
                selectors.join("\n")
            );
            let resp: Value = client
                .post("https://overpass-api.de/api/interpreter")
                .form(&[("data", ql.as_str())])
                .timeout(Duration::from_secs(35))
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            let elements = resp["elements"].as_array().cloned().unwrap_or_default();

            let conn = crm::init_db(db_path)?;
            let mut added = 0usize;
            let mut merged = 0usize;

            for el in elements.iter().take(limit) {
                let tags = &el["tags"];
                let website = tags["website"]
                    .as_str()
                    .or_else(|| tags["contact:website"].as_str())
                    .unwrap_or("");
                if !is_business_domain(website) {
                    continue;
                }
                let dom = crm::normalize_domain(website);
                if crm::is_suppressed(&conn, "", &dom)? {
                    continue;
                }
                let cat = if tags["tourism"].as_str() == Some("gallery")
                    || tags["shop"].as_str() == Some("art")
                {
                    "gallery"
                } else if tags["tourism"].as_str() == Some("hotel") {
                    "hospitality"
                } else if tags["office"].as_str() == Some("interior_design")
                    || tags["shop"].as_str() == Some("interior_decoration")
                {
                    "interior_designer"
                } else if tags["office"].as_str() == Some("architect") {
                    "architect"
                } else if matches!(tags["shop"].as_str(), Some("frame" | "furniture")) {
                    "concept_store"
                } else {
                    "unknown"
                };

                let (_, created) = crm::upsert_lead(
                    &conn,
                    &LeadInput {
                        company: tags["name"].as_str().unwrap_or("").to_string(),
                        website: crm::normalize_url(website),
                        domain: dom,
                        email: tags["email"]
                            .as_str()
                            .or_else(|| tags["contact:email"].as_str())
                            .unwrap_or("")
                            .to_string(),
                        phone: tags["phone"]
                            .as_str()
                            .or_else(|| tags["contact:phone"].as_str())
                            .unwrap_or("")
                            .to_string(),
                        category: cat.to_string(),
                        location: tags["addr:city"].as_str().unwrap_or(&city).to_string(),
                        source_url: format!(
                            "osm:{}/{}",
                            el["type"].as_str().unwrap_or("node"),
                            el["id"].as_i64().unwrap_or(0)
                        ),
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
                    "city": city,
                    "osm_elements": elements.len(),
                    "added": added,
                    "merged": merged,
                }))?
            );
        }
        DiscoverCommand::CrawlDirectory {
            url,
            category,
            location,
            limit,
        } => {
            if !check_robots_allowed(&client, &url).await {
                bail!("Blocked by robots.txt — respecting site policy for {url}");
            }
            let resp = client.get(&url).send().await?;
            let status = resp.status().as_u16();
            if matches!(status, 403 | 429 | 503) {
                bail!("HTTP {status} (gated or rate-limited) on {url}. Skipping cleanly.");
            }
            let final_url = resp.url().clone();
            let html_text = resp.error_for_status()?.text().await?;
            let source_dom = crm::normalize_domain(final_url.as_str());

            let doc = Html::parse_document(&html_text);
            let sel = Selector::parse("a[href]").unwrap();
            let mut found: BTreeMap<String, (String, String)> = BTreeMap::new();

            for a in doc.select(&sel) {
                let Some(href) = a.value().attr("href") else {
                    continue;
                };
                let Ok(abs) = final_url.join(href.trim()) else {
                    continue;
                };
                if !matches!(abs.scheme(), "http" | "https") {
                    continue;
                }
                let dom = crm::normalize_domain(abs.as_str());
                if dom.is_empty() || dom == source_dom || !is_business_domain(abs.as_str()) {
                    continue;
                }
                let text: String = a
                    .text()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
                    .chars()
                    .take(100)
                    .collect();
                found.entry(dom.clone()).or_insert_with(|| {
                    let comp = if text.len() > 2 { text } else { String::new() };
                    (format!("https://{dom}"), comp)
                });
            }

            let conn = crm::init_db(db_path)?;
            let mut added = 0usize;
            let mut merged = 0usize;
            for (dom, (site, comp)) in found.iter().take(limit) {
                if crm::is_suppressed(&conn, "", dom)? {
                    continue;
                }
                let (_, created) = crm::upsert_lead(
                    &conn,
                    &LeadInput {
                        company: comp.clone(),
                        website: site.clone(),
                        domain: dom.clone(),
                        category: category.clone(),
                        location: location.clone(),
                        source_url: url.clone(),
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
                    "directory": url,
                    "external_domains_found": found.len(),
                    "added": added,
                    "merged": merged,
                }))?
            );
        }
    }
    Ok(())
}

async fn run_search_provider(
    client: &Client,
    provider: &str,
    query: &str,
    num: usize,
) -> Result<Vec<(String, String)>> {
    let mut out = Vec::new();
    match provider {
        "serper" => {
            let key = env::var("SERPER_API_KEY")?;
            let data: Value = client
                .post("https://google.serper.dev/search")
                .header("X-API-KEY", key)
                .json(&json!({ "q": query, "num": num }))
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            if let Some(items) = data["organic"].as_array() {
                for it in items {
                    let link = it["link"].as_str().unwrap_or("");
                    if is_business_domain(link) {
                        out.push((it["title"].as_str().unwrap_or("").to_string(), link.to_string()));
                    }
                }
            }
        }
        "brave" => {
            let key = env::var("BRAVE_API_KEY")?;
            let data: Value = client
                .get("https://api.search.brave.com/res/v1/web/search")
                .header("Accept", "application/json")
                .header("X-Subscription-Token", key)
                .query(&[("q", query), ("count", &num.min(20).to_string())])
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            if let Some(items) = data["web"]["results"].as_array() {
                for it in items {
                    let link = it["url"].as_str().unwrap_or("");
                    if is_business_domain(link) {
                        out.push((it["title"].as_str().unwrap_or("").to_string(), link.to_string()));
                    }
                }
            }
        }
        "google_cse" => {
            let key = env::var("GOOGLE_CSE_API_KEY")?;
            let cx = env::var("GOOGLE_CSE_CX")?;
            let data: Value = client
                .get("https://www.googleapis.com/customsearch/v1")
                .query(&[
                    ("key", key.as_str()),
                    ("cx", cx.as_str()),
                    ("q", query),
                    ("num", &num.min(10).to_string()),
                ])
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            if let Some(items) = data["items"].as_array() {
                for it in items {
                    let link = it["link"].as_str().unwrap_or("");
                    if is_business_domain(link) {
                        out.push((it["title"].as_str().unwrap_or("").to_string(), link.to_string()));
                    }
                }
            }
        }
        other => bail!("Unsupported search provider: {other}"),
    }
    Ok(out)
}
