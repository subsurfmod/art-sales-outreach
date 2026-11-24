//! Interactive Local Preview Studio & Pure-Rust HTTP Preview Server.
//!
//! Lets the artist visually preview EVERYTHING before committing or publishing:
//!   - Live interactive switcher across all 6 Anti-Slop Website/Portfolio archetypes
//!     (including ASCII wireframe toggle + 60"-hang View-on-Wall scale preview + real registered artwork photos)
//!   - 3D & 2D Social Media Video Storyboards (9:16 & 16:9 frames + auto-embedded rendered MP4 drafts)
//!   - Rendered Multi-Touch Outreach Email previews (Touch 1 / 2 / 3 across buyer verticals)
//!   - Initial Market Research, Search Queries & Real-Asset Authenticity Dashboard
//!
//! Served locally via pure Rust std::net::TcpListener (zero external server dependencies).

use crate::assets::AssetManifest;
use crate::crm;
use crate::studio;
use anyhow::{bail, Context, Result};
use clap::Subcommand;
use serde_json::{json, Value};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::thread;

#[derive(Subcommand, Debug)]
pub enum PreviewCommand {
    /// Generate/refresh the interactive HTML Preview Studio from your workspace profile, real assets, CRM & research
    Generate {
        #[arg(long, default_value = "art-outreach-workspace")]
        workspace: PathBuf,
        #[arg(long)]
        output_dir: Option<PathBuf>,
    },
    /// Serve the Preview Studio (or any built website/video directory) on localhost via pure-Rust HTTP
    Serve {
        /// Directory to serve (defaults to art-outreach-workspace/preview)
        #[arg(long, default_value = "art-outreach-workspace/preview")]
        dir: PathBuf,
        /// Local port to bind on 127.0.0.1
        #[arg(long, default_value_t = 4173)]
        port: u16,
    },
}

fn read_json_or_empty(path: &Path) -> Value {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .unwrap_or_else(|| json!({}))
}

fn discover_local_videos(videos_dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    if let Ok(rd) = fs::read_dir(videos_dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
                    if matches!(ext.to_lowercase().as_str(), "mp4" | "webm" | "mov") {
                        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                            found.push(format!("../videos/{name}"));
                        }
                    }
                }
            }
        }
    }
    found.sort();
    found
}

pub fn generate_preview_studio(db_path: &Path, workspace: &Path, preview_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(preview_dir)?;

    let profile = read_json_or_empty(&workspace.join("artist_profile.json"));
    let integrations = read_json_or_empty(&workspace.join("integrations.json"));
    let queries = read_json_or_empty(&workspace.join("market_research/search_queries.json"));

    let manifest_path = workspace.join("real_assets/manifest.json");
    let manifest: Option<AssetManifest> = fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());

    // Copy verified real images into preview/assets/ so the preview server can serve them safely inside preview_dir
    let preview_assets_dir = preview_dir.join("real_assets_preview");
    fs::create_dir_all(&preview_assets_dir)?;
    let mut real_assets_for_ui = Vec::new();
    if let Some(ref m) = manifest {
        for a in &m.assets {
            let src_candidates = [
                workspace.join("real_assets").join(&a.rel_path),
                workspace.join("real_assets").join(&a.filename),
            ];
            let mut served_rel = String::new();
            for cand in &src_candidates {
                if cand.exists() {
                    let dst = preview_assets_dir.join(&a.filename);
                    let _ = fs::copy(cand, &dst);
                    served_rel = format!("real_assets_preview/{}", a.filename);
                    break;
                }
            }
            real_assets_for_ui.push(json!({
                "id": a.id,
                "title": a.title,
                "filename": a.filename,
                "kind": a.kind,
                "medium": a.medium,
                "dimensions_in": a.dimensions_in,
                "dimensions_cm": a.dimensions_cm,
                "price": a.price,
                "availability": a.availability,
                "sha256_short": a.sha256.chars().take(12).collect::<String>(),
                "url": served_rel,
            }));
        }
    }

    // Grab CRM summary + any drafted emails
    let mut crm_summary = json!({"total_leads": 0, "mx_verified": 0, "drafts": []});
    if db_path.exists() {
        if let Ok(conn) = crm::open_conn(db_path) {
            let total: i64 = conn
                .query_row("SELECT COUNT(*) FROM leads", [], |r| r.get(0))
                .unwrap_or(0);
            let mx_ok: i64 = conn
                .query_row("SELECT COUNT(*) FROM leads WHERE mx_valid = 1", [], |r| r.get(0))
                .unwrap_or(0);
            let mut drafts = Vec::new();
            if let Ok(mut stmt) = conn.prepare(
                "SELECT recipient_email, subject, body_text, sequence_step, status FROM outreach_log ORDER BY id DESC LIMIT 12",
            ) {
                if let Ok(rows) = stmt.query_map([], |r| {
                    Ok(json!({
                        "to": r.get::<_, String>(0)?,
                        "subject": r.get::<_, String>(1)?,
                        "body": r.get::<_, String>(2)?,
                        "step": r.get::<_, i64>(3)?,
                        "status": r.get::<_, String>(4)?,
                    }))
                }) {
                    for item in rows.flatten() {
                        drafts.push(item);
                    }
                }
            }
            crm_summary = json!({
                "total_leads": total,
                "mx_verified": mx_ok,
                "drafts": drafts,
            });
        }
    }

    let state = json!({
        "profile": profile,
        "integrations": integrations,
        "real_assets": real_assets_for_ui,
        "website_variations": studio::website_variations(),
        "video_variations": studio::video_variations(),
        "outreach_variations": studio::outreach_variations(),
        "local_videos": discover_local_videos(&workspace.join("videos")),
        "queries": queries,
        "crm": crm_summary,
    });

    // Escape </script> defensively when embedding JSON into HTML
    let safe_json = serde_json::to_string(&state)?.replace("</", "<\\/");
    let html = PREVIEW_HTML_TEMPLATE.replace("__PREVIEW_STATE_JSON__", &safe_json);
    let out_file = preview_dir.join("index.html");
    fs::write(&out_file, html)?;
    Ok(out_file)
}

pub fn serve_directory(dir: &Path, port: u16) -> Result<()> {
    if !dir.exists() {
        bail!(
            "Preview directory does not exist: {}. Run `art-outreach setup` or `art-outreach preview generate` first.",
            dir.display()
        );
    }
    let root = fs::canonicalize(dir)?;
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)
        .with_context(|| format!("Failed to bind local preview server on {addr}"))?;

    println!("==================================================================");
    println!("  ART SELLER PREVIEW STUDIO LIVE AT: http://{addr}/");
    println!("  Serving: {}", root.display());
    println!("  Press Ctrl+C to stop.");
    println!("==================================================================");

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let root_clone = root.clone();
        thread::spawn(move || {
            let _ = handle_http_connection(stream, &root_clone);
        });
    }
    Ok(())
}

fn handle_http_connection(mut stream: std::net::TcpStream, root: &Path) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut req_line = String::new();
    if reader.read_line(&mut req_line)? == 0 {
        return Ok(());
    }
    let mut parts = req_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let raw_target = parts.next().unwrap_or("/");
    if method != "GET" && method != "HEAD" {
        let resp = "HTTP/1.1 405 Method Not Allowed\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    let clean_path = raw_target.split('?').next().unwrap_or("/");
    let rel = clean_path.trim_start_matches('/');
    let mut target = if rel.is_empty() {
        root.join("index.html")
    } else {
        root.join(rel)
    };
    if target.is_dir() {
        target = target.join("index.html");
    }

    let canonical = match fs::canonicalize(&target) {
        Ok(c) if c.starts_with(root) && c.is_file() => c,
        _ => {
            let body = b"404 Not Found";
            let hdr = format!(
                "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(hdr.as_bytes())?;
            if method == "GET" {
                stream.write_all(body)?;
            }
            return Ok(());
        }
    };

    let bytes = fs::read(&canonical)?;
    let mime = guess_mime(&canonical);
    let hdr = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        bytes.len()
    );
    stream.write_all(hdr.as_bytes())?;
    if method == "GET" {
        stream.write_all(&bytes)?;
    }
    stream.flush()?;
    Ok(())
}

fn guess_mime(p: &Path) -> &'static str {
    match p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

pub fn handle_command(db_path: &Path, cmd: PreviewCommand) -> Result<()> {
    match cmd {
        PreviewCommand::Generate {
            workspace,
            output_dir,
        } => {
            let out = output_dir.unwrap_or_else(|| workspace.join("preview"));
            let actual_db = if db_path == Path::new(crm::DEFAULT_DB_REL) {
                workspace.join("data/art_leads.db")
            } else {
                db_path.to_path_buf()
            };
            let idx = generate_preview_studio(&actual_db, &workspace, &out)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "preview_html": idx.display().to_string(),
                    "serve_command": format!("art-outreach preview serve --dir {} --port 4173", out.display()),
                }))?
            );
        }
        PreviewCommand::Serve { dir, port } => {
            serve_directory(&dir, port)?;
        }
    }
    Ok(())
}

const PREVIEW_HTML_TEMPLATE: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8" />
<meta name="viewport" content="width=device-width, initial-scale=1.0" />
<title>Art Seller Preview Studio — Pre-Launch Visual Inspection</title>
<link rel="preconnect" href="https://fonts.googleapis.com" />
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
<link href="https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:wght@500;700&family=Cormorant+Garamond:ital,wght@0,400;0,600;1,400&family=Fraunces:opsz,wght@9..144,400;9..144,600&family=IBM+Plex+Mono:wght@400;500&family=Instrument+Serif:ital@0;1&family=Newsreader:ital,opsz,wght@0,6..72,400;1,6..72,400&family=Syne:wght@700;800&display=swap" rel="stylesheet" />
<style>
  :root {
    --shell-bg: #111110;
    --shell-surface: #1A1A18;
    --shell-border: #2D2C28;
    --shell-ink: #F2EFE9;
    --shell-muted: #A39E93;
    --shell-accent: #C89B54;
    --ok: #3D9A6B;
    --warn: #D97736;
  }
  * { box-sizing: border-box; }
  body {
    margin: 0;
    background: var(--shell-bg);
    color: var(--shell-ink);
    font-family: 'IBM Plex Mono', monospace;
    font-size: 13px;
    line-height: 1.55;
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 18px 28px;
    border-bottom: 1px solid var(--shell-border);
    background: #0D0D0C;
    position: sticky;
    top: 0;
    z-index: 40;
  }
  .brand { font-family: 'Cormorant Garamond', serif; font-size: 24px; letter-spacing: 0.04em; }
  .badge {
    display: inline-block;
    padding: 3px 9px;
    border: 1px solid var(--shell-border);
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--shell-accent);
  }
  .badge.ok { color: var(--ok); border-color: #26573E; }
  nav.tabs { display: flex; gap: 8px; flex-wrap: wrap; }
  nav.tabs button {
    background: transparent;
    border: 1px solid var(--shell-border);
    color: var(--shell-muted);
    padding: 8px 14px;
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.15s;
  }
  nav.tabs button.active, nav.tabs button:hover {
    color: var(--shell-ink);
    border-color: var(--shell-accent);
    background: #211F1B;
  }
  main { max-width: 1380px; margin: 0 auto; padding: 28px; }
  section.panel { display: none; }
  section.panel.active { display: block; }
  .controls {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 20px;
    align-items: center;
  }
  .pill {
    background: var(--shell-surface);
    border: 1px solid var(--shell-border);
    color: var(--shell-ink);
    padding: 7px 12px;
    font-family: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .pill.active { border-color: var(--shell-accent); color: var(--shell-accent); background: #221E16; }
  .grid-2 { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; }
  .grid-3 { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 18px; }
  @media (max-width: 960px) { .grid-2 { grid-template-columns: 1fr; } }
  .card {
    background: var(--shell-surface);
    border: 1px solid var(--shell-border);
    padding: 18px;
  }
  pre.ascii {
    background: #0A0A09;
    border: 1px solid var(--shell-border);
    padding: 16px;
    overflow-x: auto;
    color: #DED9CE;
    font-size: 11.5px;
    line-height: 1.35;
  }
  /* Live Website Archetype Stage */
  #web-stage {
    border: 1px solid var(--shell-border);
    min-height: 540px;
    padding: 36px;
    transition: background 0.25s, color 0.25s;
  }
  .stage-nav {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    padding-bottom: 18px;
    margin-bottom: 36px;
    border-bottom: 1px solid currentColor;
    opacity: 0.85;
  }
  .stage-hero {
    display: grid;
    grid-template-columns: 1.15fr 0.85fr;
    gap: 36px;
    align-items: center;
  }
  @media (max-width: 820px) { .stage-hero { grid-template-columns: 1fr; } }
  .art-frame {
    aspect-ratio: 4 / 5;
    max-height: 440px;
    margin: 0 auto;
    width: 100%;
    max-width: 360px;
    border: 1px solid rgba(128,128,128,0.4);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 18px;
    text-align: center;
    position: relative;
    overflow: hidden;
  }
  .art-frame img {
    width: 100%;
    height: 100%;
    object-fit: contain;
  }
  .wall-room {
    background: linear-gradient(180deg, rgba(128,128,128,0.12) 0%, rgba(128,128,128,0.03) 78%, rgba(128,128,128,0.22) 78%, rgba(128,128,128,0.14) 100%);
    padding: 28px 24px 64px;
    border: 1px dashed rgba(128,128,128,0.45);
    text-align: center;
  }
  .wall-room .art-frame { max-width: 210px; max-height: 260px; box-shadow: 0 18px 32px rgba(0,0,0,0.25); }
  .scale-note { font-size: 11px; opacity: 0.72; margin-top: 14px; letter-spacing: 0.06em; text-transform: uppercase; }
  .video-frame {
    border: 1px solid var(--shell-border);
    background: #0C0C0B;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 16px;
  }
  .aspect-tag {
    display: inline-block;
    font-size: 10px;
    padding: 2px 7px;
    border: 1px solid var(--shell-accent);
    color: var(--shell-accent);
    margin-bottom: 8px;
  }
  .email-preview {
    background: #FAF8F4;
    color: #1A1917;
    padding: 22px;
    border: 1px solid #D6D1C7;
    font-family: Georgia, serif;
    white-space: pre-wrap;
    line-height: 1.65;
  }
  .meta-row { color: var(--shell-muted); font-size: 12px; margin-bottom: 8px; }
  h2 { font-family: 'Cormorant Garamond', serif; font-size: 28px; font-weight: 400; margin: 0 0 12px; }
  h3 { font-size: 14px; text-transform: uppercase; letter-spacing: 0.08em; color: var(--shell-accent); margin: 0 0 10px; }
</style>
</head>
<body>
<header>
  <div>
    <div class="brand" id="hdr-brand">Art Seller Preview Studio</div>
    <div style="margin-top:4px; display:flex; gap:8px; flex-wrap:wrap;">
      <span class="badge ok">ZERO-AI-ART PROTOCOL ACTIVE</span>
      <span class="badge" id="hdr-assets">0 REAL ASSETS</span>
      <span class="badge" id="hdr-leads">0 CRM LEADS</span>
    </div>
  </div>
  <nav class="tabs">
    <button class="active" data-tab="website">1. Website & Portfolio (6 Variations)</button>
    <button data-tab="video">2. 3D & 2D Social Videos (6 Formats)</button>
    <button data-tab="outreach">3. Outreach Emails (6 Angles)</button>
    <button data-tab="research">4. Market Research & Setup</button>
  </nav>
</header>

<main>
  <!-- TAB 1: WEBSITE & PORTFOLIO -->
  <section id="tab-website" class="panel active">
    <h2>Live Anti-Slop Website & Portfolio Variation Preview</h2>
    <p class="meta-row">Compare all 6 museum-grade archetypes live using your real profile & registered artwork scans — toggle ASCII Wireframe & 60"-hang View-on-Wall scale before building.</p>
    <div class="controls" id="web-picker"></div>
    <div class="controls">
      <button class="pill active" id="mode-visual">Visual Prototype</button>
      <button class="pill" id="mode-wall">60&quot; View-on-Wall Scale</button>
      <button class="pill" id="mode-ascii">ASCII Wireframe Spec</button>
    </div>
    <div id="web-meta" class="card" style="margin-bottom:16px;"></div>
    <div id="web-stage"></div>
    <pre id="web-ascii" class="ascii" style="display:none;"></pre>
  </section>

  <!-- TAB 2: 3D & 2D VIDEO -->
  <section id="tab-video" class="panel">
    <h2>3D & 2D Social Media Artwork Video Pre-Visualization</h2>
    <p class="meta-row">Every format strictly uses your verified real artwork scans & studio clips (zero AI-generated art). Orchestrates your locally installed Blender, OneTake, MotionMaxxing, Remotion, HyperFrames, FFmpeg & YouTube Shorts Uploader skills.</p>
    <div id="video-local-renders" style="margin-bottom:20px;"></div>
    <div class="grid-3" id="video-grid"></div>
  </section>

  <!-- TAB 3: OUTREACH -->
  <section id="tab-outreach" class="panel">
    <h2>Non-Fodder Outreach Variations & Live Email Preview</h2>
    <p class="meta-row">Plain-text-first, under 120 words, zero spam attachments — preview your rendered pitch before uploading to Drafts or sending.</p>
    <div class="controls" id="outreach-picker"></div>
    <div class="grid-2">
      <div class="card" id="outreach-strategy"></div>
      <div>
        <div class="meta-row" id="email-subject-bar"></div>
        <div class="email-preview" id="email-body-box"></div>
      </div>
    </div>
    <div id="crm-drafts-wrap" style="margin-top:24px;"></div>
  </section>

  <!-- TAB 4: MARKET RESEARCH -->
  <section id="tab-research" class="panel">
    <h2>Initial Market Research, Real Assets & Integration Readiness</h2>
    <div class="grid-2">
      <div class="card" id="res-profile"></div>
      <div class="card" id="res-integrations"></div>
    </div>
    <div class="card" style="margin-top:20px;" id="res-queries"></div>
  </section>
</main>

<script>
const STATE = __PREVIEW_STATE_JSON__;
const P = STATE.profile || {};
const artistName = P.artist_name || "Your Name";
const studioName = P.studio_name || "Your Studio";
const medium = P.medium || "Original Physical Artwork";
const styleSummary = P.style_summary || "Contemporary physical works";
const sizeRange = P.size_range || "36x48\" – 60x84\"";
const priceRange = P.price_range || "Price on Request";
const loc = P.location || "Studio Location";
const realAssets = STATE.real_assets || [];
const heroAsset = realAssets.find(a => a.kind === "artwork" && a.url) || realAssets.find(a => a.url) || null;

document.getElementById("hdr-brand").textContent = `${artistName} — Art Seller Preview Studio`;
document.getElementById("hdr-assets").textContent = `${realAssets.length} REAL ASSETS REGISTERED`;
document.getElementById("hdr-leads").textContent = `${(STATE.crm && STATE.crm.total_leads) || 0} CRM LEADS`;

// Tabs
document.querySelectorAll("nav.tabs button").forEach(btn => {
  btn.onclick = () => {
    document.querySelectorAll("nav.tabs button").forEach(b => b.classList.remove("active"));
    document.querySelectorAll("section.panel").forEach(p => p.classList.remove("active"));
    btn.classList.add("active");
    document.getElementById("tab-" + btn.dataset.tab).classList.add("active");
  };
});

// Archetype CSS themes
const THEMES = {
  "white-cube-monograph": { bg: "#F7F5F0", fg: "#181816", accent: "#9A6B3F", display: "'Cormorant Garamond', serif", mono: "'IBM Plex Mono', monospace" },
  "catalogue-raisonne":   { bg: "#F3F1EC", fg: "#111111", accent: "#1E4E8C", display: "'Newsreader', serif", mono: "'IBM Plex Mono', monospace" },
  "editorial-brutalist":  { bg: "#F2EFE9", fg: "#0D0D0C", accent: "#D9381E", display: "'Syne', sans-serif", mono: "'IBM Plex Mono', monospace" },
  "atmospheric-salon":    { bg: "#111214", fg: "#ECE7DD", accent: "#C8A25A", display: "'Instrument Serif', serif", mono: "'IBM Plex Mono', monospace" },
  "tactile-studio-notebook": { bg: "#EFEAE0", fg: "#261E18", accent: "#B85233", display: "'Fraunces', serif", mono: "'IBM Plex Mono', monospace" },
  "kinetic-exhibition":   { bg: "#FAF9F6", fg: "#141417", accent: "#1C3FD9", display: "'Bricolage Grotesque', sans-serif", mono: "'IBM Plex Mono', monospace" },
};

let activeWeb = (P.website_variation && THEMES[P.website_variation]) ? P.website_variation : "white-cube-monograph";
let webMode = "visual";

function renderWeb() {
  const v = STATE.website_variations.find(x => x.slug === activeWeb) || STATE.website_variations[0];
  const t = THEMES[v.slug] || THEMES["white-cube-monograph"];
  document.querySelectorAll("#web-picker button").forEach(b => b.classList.toggle("active", b.dataset.slug === v.slug));
  document.getElementById("web-meta").innerHTML = `
    <h3>${v.name} (<code>${v.slug}</code>)</h3>
    <div><strong>Benchmark:</strong> ${v.artist_benchmark}</div>
    <div><strong>Anti-Slop Typography:</strong> ${v.typography_pairing}</div>
    <div><strong>Palette:</strong> ${v.palette}</div>
    <div><strong>Layout Signature:</strong> ${v.signature_layout}</div>
  `;
  const stage = document.getElementById("web-stage");
  const ascii = document.getElementById("web-ascii");
  if (webMode === "ascii") {
    stage.style.display = "none";
    ascii.style.display = "block";
    ascii.textContent = v.ascii_wireframe.trim();
    return;
  }
  ascii.style.display = "none";
  stage.style.display = "block";
  stage.style.background = t.bg;
  stage.style.color = t.fg;
  stage.style.fontFamily = t.mono;

  const artBlock = heroAsset
    ? `<img src="${heroAsset.url}" alt="${heroAsset.title}" />`
    : `<div style="opacity:0.75; font-size:12px;">
         <div style="font-family:${t.display}; font-size:22px; margin-bottom:8px;">[ YOUR REAL ARTWORK PHOTO ]</div>
         <div>Drop authentic scans into<br/><code>real_assets/01_artworks/</code><br/>&amp; re-run <code>assets register</code>.<br/><br/>Zero AI-generated placeholders.</div>
       </div>`;

  const artWrap = webMode === "wall"
    ? `<div class="wall-room"><div class="art-frame">${artBlock}</div><div class="scale-note">60&quot; Eye-Level Gallery Hang · Scale Preview (${sizeRange})</div></div>`
    : `<div class="art-frame">${artBlock}</div>`;

  stage.innerHTML = `
    <div class="stage-nav">
      <div style="font-family:${t.display}; font-size:26px; letter-spacing:0.02em;">${artistName}</div>
      <div style="font-size:11px; letter-spacing:0.08em; text-transform:uppercase; display:flex; gap:18px;">
        <span>Works</span><span>Viewing Room</span><span>Archive / CV</span><span style="color:${t.accent};">Inquire</span>
      </div>
    </div>
    <div class="stage-hero">
      <div>${artWrap}</div>
      <div>
        <div style="font-size:11px; text-transform:uppercase; letter-spacing:0.12em; color:${t.accent}; margin-bottom:10px;">${v.name} · ${loc}</div>
        <div style="font-family:${t.display}; font-size:42px; line-height:1.1; margin-bottom:16px;">${heroAsset ? heroAsset.title : "Selected Works — 2026"}</div>
        <p style="font-size:13px; opacity:0.82; max-width:44ch; margin-bottom:20px;">${medium}. ${styleSummary}.</p>
        <div style="font-size:11px; opacity:0.7; margin-bottom:22px;">Dimensions: ${sizeRange} · ${priceRange} · Provenance: ${studioName}</div>
        <div style="display:flex; gap:12px; flex-wrap:wrap;">
          <span style="border:1px solid ${t.accent}; color:${t.accent}; padding:10px 16px; font-size:11px; letter-spacing:0.08em; text-transform:uppercase;">Inquire Privately</span>
          <span style="border:1px solid currentColor; opacity:0.65; padding:10px 16px; font-size:11px; letter-spacing:0.08em; text-transform:uppercase;">Enter Viewing Room</span>
        </div>
      </div>
    </div>
  `;
}

const wp = document.getElementById("web-picker");
STATE.website_variations.forEach(v => {
  const b = document.createElement("button");
  b.className = "pill";
  b.dataset.slug = v.slug;
  b.textContent = v.name;
  b.onclick = () => { activeWeb = v.slug; renderWeb(); };
  wp.appendChild(b);
});
document.getElementById("mode-visual").onclick = (e) => { webMode = "visual"; setModeBtn(e.target); renderWeb(); };
document.getElementById("mode-wall").onclick = (e) => { webMode = "wall"; setModeBtn(e.target); renderWeb(); };
document.getElementById("mode-ascii").onclick = (e) => { webMode = "ascii"; setModeBtn(e.target); renderWeb(); };
function setModeBtn(el) {
  ["mode-visual", "mode-wall", "mode-ascii"].forEach(id => document.getElementById(id).classList.remove("active"));
  el.classList.add("active");
}
renderWeb();

// TAB 2: VIDEOS
const vgrid = document.getElementById("video-grid");
STATE.video_variations.forEach(v => {
  const div = document.createElement("div");
  div.className = "video-frame";
  div.innerHTML = `
    <div>
      <span class="aspect-tag">${v.dimension} · ${v.slug}</span>
      <h3>${v.name}</h3>
      <p style="font-size:12px; color:#D8D4CA;">${v.visual_concept}</p>
    </div>
    <div style="margin-top:12px; padding-top:12px; border-top:1px solid #2D2C28; font-size:11px; color:#A39E93;">
      <div><strong>Real Assets Needed:</strong> ${v.real_assets_required}</div>
      <div style="margin-top:4px;"><strong>Local Skills:</strong> <code>${v.local_skills.join(", ")}</code></div>
      <div style="margin-top:4px;"><strong>Platforms:</strong> ${v.platform_fit}</div>
    </div>
  `;
  vgrid.appendChild(div);
});

// TAB 3: OUTREACH
let activeOut = (P.outreach_angle) || STATE.outreach_variations[0].slug;
const op = document.getElementById("outreach-picker");
STATE.outreach_variations.forEach(o => {
  const b = document.createElement("button");
  b.className = "pill";
  b.dataset.slug = o.slug;
  b.textContent = o.name;
  b.onclick = () => { activeOut = o.slug; renderOutreach(); };
  op.appendChild(b);
});

function renderOutreach() {
  const o = STATE.outreach_variations.find(x => x.slug === activeOut) || STATE.outreach_variations[0];
  document.querySelectorAll("#outreach-picker button").forEach(b => b.classList.toggle("active", b.dataset.slug === o.slug));
  document.getElementById("outreach-strategy").innerHTML = `
    <h3>${o.name}</h3>
    <p><strong>Target:</strong> ${o.target_audience}</p>
    <p><strong>Hook Mechanism:</strong> ${o.hook_mechanism}</p>
    <p><strong>Anti-Fodder Rule:</strong> ${o.anti_fodder_rule}</p>
  `;
  const sampleSubj = `Original ${medium} for Studio Sol Interiors?`;
  const sampleBody = `Hi Elena,

I really admire the warm minimalist sensibility across Studio Sol Interiors' recent projects.

I'm ${artistName} (${loc}), creating ${styleSummary} (${sizeRange}).

${o.hook_mechanism}

Portfolio / Viewing Room: ${P.portfolio_url || "https://yourstudio.com"}

Best,
${P.signature || artistName}

${P.opt_out_line || "If sourcing original artwork isn't on your radar right now, just reply 'pass' and I won't follow up."}`;
  document.getElementById("email-subject-bar").textContent = "SUBJECT: " + sampleSubj;
  document.getElementById("email-body-box").textContent = sampleBody;
}
renderOutreach();

if (STATE.crm && STATE.crm.drafts && STATE.crm.drafts.length > 0) {
  const wrap = document.getElementById("crm-drafts-wrap");
  wrap.innerHTML = `<h3>Live Drafts Currently In Your SQLite CRM (${STATE.crm.drafts.length})</h3>` +
    STATE.crm.drafts.map(d => `<div class="card" style="margin-bottom:10px;"><div><strong>To:</strong> ${d.to} | <strong>Touch ${d.step}</strong> | Status: ${d.status}</div><div><strong>Subject:</strong> ${d.subject}</div></div>`).join("");
}

// TAB 4: MARKET RESEARCH
document.getElementById("res-profile").innerHTML = `
  <h3>Artist Profile & Target Market</h3>
  <div><strong>Artist:</strong> ${artistName} (${studioName})</div>
  <div><strong>Medium:</strong> ${medium}</div>
  <div><strong>Price Tier:</strong> ${P.price_tier || "mid-career"} (${priceRange})</div>
  <div><strong>Target Cities:</strong> ${(P.target_cities || []).join(", ") || "Not set"}</div>
  <div><strong>Target Verticals:</strong> ${(P.target_verticals || []).join(", ") || "Not set"}</div>
`;
const caps = (STATE.integrations && STATE.integrations.detected_local_capabilities) || {};
document.getElementById("res-integrations").innerHTML = `
  <h3>Social & Local Toolchain Readiness</h3>
  <div><strong>Email Mode:</strong> ${(STATE.integrations && STATE.integrations.email_backend && STATE.integrations.email_backend.mode) || "not configured"}</div>
  <div><strong>Blender 3D Installed:</strong> ${caps.blender_installed ? "YES" : "NO"}</div>
  <div><strong>FFmpeg Installed:</strong> ${caps.ffmpeg_installed ? "YES" : "NO"}</div>
  <div><strong>YouTube Shorts OAuth Ready:</strong> ${caps.youtube_shorts_oauth_token_ready ? "YES" : "Needs OAuth setup"}</div>
`;
const qList = Array.isArray(STATE.queries) ? STATE.queries : [];
document.getElementById("res-queries").innerHTML = `
  <h3>Initial Market Research Queries (${qList.length} generated)</h3>
  ${qList.slice(0, 12).map(q => `<div style="margin-bottom:6px;"><span class="badge">${q.vertical}</span> <span style="color:#A39E93;">[${q.city}]</span> <code>${q.query}</code></div>`).join("") || "<div>Run <code>art-outreach setup</code> to generate initial queries.</div>"}
`;
</script>
</body>
</html>
"##;
