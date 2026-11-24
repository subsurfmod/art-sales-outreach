//! Effortless First-Time Setup Wizard & Initial Market Research Scaffolder.
//!
//! Guides an artist (interactively via terminal OR non-interactively via JSON/CLI flags
//! driven by an AI agent's multiple-choice UI) through:
//!   1. Artist identity, portfolio intake & REAL artwork asset folder setup
//!   2. Social media & publishing integrations (YouTube Shorts, Instagram, TikTok, Pinterest, LinkedIn, Newsletter)
//!   3. Email outreach backend selection (Gmail Drafts, SMTP, Resend, Brevo, or Draft-Only)
//!   4. Initial market research generation (target cities, connector verticals, Google Boolean queries,
//!      peer artist benchmarks, and optional OpenStreetMap local market scan)
//!   5. Instant Preview Studio generation so the artist sees everything before going live!

use crate::assets::{self, AssetsCommand};
use crate::crm;
use crate::discover::{self, DiscoverCommand};
use crate::preview;
use crate::studio;
use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Args, Debug)]
pub struct SetupArgs {
    /// Workspace root directory to initialize
    #[arg(long, default_value = "art-outreach-workspace")]
    pub workspace: PathBuf,

    /// Optional JSON config file for one-shot non-interactive setup (ideal for agent ask_question flows)
    #[arg(long)]
    pub from_json: Option<PathBuf>,

    /// Non-interactive quick-init using defaults + any provided flags (great for CI or fast scaffolding)
    #[arg(long)]
    pub defaults: bool,

    /// Path to an existing folder of REAL artist photos/videos to register immediately
    #[arg(long)]
    pub assets_dir: Option<PathBuf>,

    /// Also run a live free OpenStreetMap local market scan for the primary city during setup
    #[arg(long)]
    pub run_osm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    pub artist_name: String,
    pub studio_name: String,
    pub location: String,
    pub medium: String,
    pub style_summary: String,
    pub style_keywords: Vec<String>,
    pub mood: String,
    pub size_range: String,
    pub price_range: String,
    pub price_tier: String,
    pub trade_discount: String,
    pub representation: String,
    pub portfolio_url: String,
    pub lookbook_url: String,
    pub target_cities: Vec<String>,
    pub target_verticals: Vec<String>,
    pub website_variation: String,
    pub video_formats: Vec<String>,
    pub outreach_angle: String,
    pub social_channels: SocialIntegrations,
    pub email_backend: EmailBackendChoice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialIntegrations {
    pub instagram_handle: String,
    pub youtube_channel: String,
    pub youtube_shorts_enabled: bool,
    pub tiktok_handle: String,
    pub pinterest_handle: String,
    pub linkedin_url: String,
    pub newsletter_platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailBackendChoice {
    /// draft_only | gmail_imap_drafts | gmail_smtp | custom_smtp | resend_api | brevo_api
    pub mode: String,
    pub sender_email: String,
    pub sender_name: String,
}

impl Default for SetupConfig {
    fn default() -> Self {
        Self {
            artist_name: "Your Name".into(),
            studio_name: "Your Name Studio".into(),
            location: "Austin, TX".into(),
            medium: "textured oil & mixed-media paintings on linen".into(),
            style_summary: "large-scale atmospheric abstracts in warm earth, bone, and charcoal tones".into(),
            style_keywords: vec![
                "minimalist".into(),
                "warm modern".into(),
                "organic".into(),
                "abstract".into(),
                "contemporary".into(),
                "textural".into(),
            ],
            mood: "warm minimalist".into(),
            size_range: "36x48\" to 60x84\" (plus custom commissions)".into(),
            price_range: "$1,400–$8,500".into(),
            price_tier: "mid-career".into(),
            trade_discount: "20% trade pricing + complimentary room mockups".into(),
            representation: "independent".into(),
            portfolio_url: "https://yourstudio.com/works".into(),
            lookbook_url: "https://yourstudio.com/viewing-room".into(),
            target_cities: vec!["Austin".into(), "Denver".into(), "Nashville".into()],
            target_verticals: vec![
                "interior_designer".into(),
                "art_consultant".into(),
                "stager".into(),
                "hospitality".into(),
                "gallery".into(),
            ],
            website_variation: "white-cube-monograph".into(),
            video_formats: vec![
                "blender-3d-raking-light".into(),
                "onetake-kinetic-exhibition".into(),
                "remotion-collector-story".into(),
            ],
            outreach_angle: "free-wall-mockup".into(),
            social_channels: SocialIntegrations {
                instagram_handle: "@yourartstudio".into(),
                youtube_channel: String::new(),
                youtube_shorts_enabled: true,
                tiktok_handle: String::new(),
                pinterest_handle: String::new(),
                linkedin_url: String::new(),
                newsletter_platform: "none".into(),
            },
            email_backend: EmailBackendChoice {
                mode: "gmail_imap_drafts".into(),
                sender_email: "hello@yourartstudio.com".into(),
                sender_name: "Your Name".into(),
            },
        }
    }
}

fn prompt_line(label: &str, default: &str) -> Result<String> {
    if default.is_empty() {
        print!("  {label}: ");
    } else {
        print!("  {label} [{default}]: ");
    }
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

fn prompt_choice<'a>(title: &str, options: &[(&'a str, &'a str)], default_idx: usize) -> Result<&'a str> {
    println!("\n{title}");
    for (i, (slug, desc)) in options.iter().enumerate() {
        let marker = if i == default_idx { " (default)" } else { "" };
        println!("  [{}] {} — {}{}", i + 1, slug, desc, marker);
    }
    let ans = prompt_line("Pick number", &(default_idx + 1).to_string())?;
    let idx = ans.parse::<usize>().ok().and_then(|n| n.checked_sub(1));
    Ok(options.get(idx.unwrap_or(default_idx)).unwrap_or(&options[default_idx]).0)
}

fn prompt_multi(title: &str, options: &[(&str, &str)], defaults: &[usize]) -> Result<Vec<String>> {
    println!("\n{title}");
    for (i, (slug, desc)) in options.iter().enumerate() {
        println!("  [{}] {} — {}", i + 1, slug, desc);
    }
    let def_str = defaults
        .iter()
        .map(|i| (i + 1).to_string())
        .collect::<Vec<_>>()
        .join(",");
    let ans = prompt_line("Pick comma-separated numbers (or 'all')", &def_str)?;
    if ans.eq_ignore_ascii_case("all") {
        return Ok(options.iter().map(|(s, _)| (*s).to_string()).collect());
    }
    let mut picked = Vec::new();
    for part in ans.split(',') {
        if let Ok(n) = part.trim().parse::<usize>() {
            if let Some((slug, _)) = n.checked_sub(1).and_then(|idx| options.get(idx)) {
                picked.push((*slug).to_string());
            }
        }
    }
    if picked.is_empty() {
        for &d in defaults {
            if let Some((slug, _)) = options.get(d) {
                picked.push((*slug).to_string());
            }
        }
    }
    Ok(picked)
}

fn run_interactive_wizard() -> Result<SetupConfig> {
    println!("==================================================================");
    println!("  ART SELLER ASSISTANT — FIRST-TIME SETUP WIZARD (RUST EDITION)");
    println!("  Strict Zero-AI-Art / Zero-AI-Artist Authenticity Protocol Active");
    println!("==================================================================");

    let mut cfg = SetupConfig::default();

    println!("\n--- STEP 1/4: ARTIST PROFILE & PORTFOLIO ---");
    cfg.artist_name = prompt_line("Artist full name", &cfg.artist_name)?;
    cfg.studio_name = prompt_line("Studio / brand name", &format!("{} Studio", cfg.artist_name))?;
    cfg.location = prompt_line("Your city & country/state", &cfg.location)?;
    cfg.medium = prompt_line("Primary physical medium (e.g. oil on linen, ceramics, bronze)", &cfg.medium)?;
    cfg.style_summary = prompt_line("1-line description of your visual style", &cfg.style_summary)?;
    cfg.mood = prompt_line("Mood / aesthetic keywords (e.g. warm minimalist, dark moody, bold brutalist)", &cfg.mood)?;
    cfg.size_range = prompt_line("Physical dimension range", &cfg.size_range)?;
    cfg.price_range = prompt_line("Price range", &cfg.price_range)?;

    cfg.price_tier = prompt_choice(
        "Market tier:",
        &[
            ("emerging", "Direct collectors, local shows, $250–$2,000"),
            ("mid-career", "Interior designers, consultants, galleries, $1,500–$12,000"),
            ("gallery-bluechip", "Private Viewing Rooms, Catalogue Raisonné, $10,000+"),
        ],
        1,
    )?
    .to_string();

    cfg.representation = prompt_choice(
        "Gallery representation status:",
        &[
            ("independent", "Direct from artist studio (full freedom)"),
            ("hybrid", "Select gallery shows + direct studio / trade sales"),
            ("gallery-represented", "Represented (route primary territory sales to gallery)"),
        ],
        0,
    )?
    .to_string();

    cfg.portfolio_url = prompt_line("Existing portfolio URL (or planned domain)", &cfg.portfolio_url)?;

    println!("\n--- STEP 2/4: WEBSITE, PORTFOLIO & 3D/2D VIDEO VARIATIONS ---");
    cfg.website_variation = prompt_choice(
        "Choose your Anti-Slop Website & Portfolio Archetype (you can preview ALL 6 live after setup!):",
        &[
            ("white-cube-monograph", "Agnes Martin / Zwirner stillness + View-on-Wall scale toggle"),
            ("catalogue-raisonne", "Gerhard Richter filterable oeuvre index + deep-zoom provenance"),
            ("editorial-brutalist", "Jenny Holzer high-contrast exhibition poster grid"),
            ("atmospheric-salon", "Rothko / Turrell dark-wall luminous private viewing room"),
            ("tactile-studio-notebook", "Olafur Eliasson craft, pigment & studio-process archive"),
            ("kinetic-exhibition", "Pace / Hauser & Wirth horizontal gallery rail + VIP trade drop"),
        ],
        0,
    )?
    .to_string();

    cfg.video_formats = prompt_multi(
        "Select 3D & 2D Social Video formats to prepare (using ONLY your real artwork photos/clips):",
        &[
            ("blender-3d-raking-light", "3D Blender gallery wall + warm raking light sweep across real scan"),
            ("blender-3d-architectural-scale", "3D Blender 1:1 physical scale inside luxury loft / hotel lobby"),
            ("onetake-kinetic-exhibition", "2D OneTake single-shot macro-to-full-frame gallery reveal"),
            ("motionmaxxing-collector-film", "2D+3D MotionMaxxing anti-slop collection launch film"),
            ("remotion-collector-story", "2D Remotion 9:16 narrated studio story Short (+ YouTube Shorts upload)"),
            ("macro-split-process", "2D FFmpeg studio-vs-finished split + real signature draw-on"),
        ],
        &[0, 2, 4],
    )?;

    println!("\n--- STEP 3/4: SOCIAL MEDIA & EMAIL INTEGRATIONS ---");
    cfg.social_channels.instagram_handle = prompt_line("Instagram handle (or blank to skip)", &cfg.social_channels.instagram_handle)?;
    cfg.social_channels.youtube_channel = prompt_line("YouTube channel URL/handle for Shorts (or blank)", "")?;
    cfg.social_channels.tiktok_handle = prompt_line("TikTok handle (or blank)", "")?;
    cfg.social_channels.pinterest_handle = prompt_line("Pinterest handle — great for interior designers! (or blank)", "")?;
    cfg.social_channels.linkedin_url = prompt_line("LinkedIn URL — great for art consultants & architects! (or blank)", "")?;

    cfg.email_backend.mode = prompt_choice(
        "Choose your Email Outreach delivery method:",
        &[
            ("gmail_imap_drafts", "Upload personalized drafts straight into Gmail/Outlook Drafts (safest start)"),
            ("gmail_smtp", "Automated throttled SMTP via Gmail / Google Workspace App Password"),
            ("custom_smtp", "Custom domain SMTP (Zoho, Fastmail, Outlook, Hostinger, etc.)"),
            ("resend_api", "Resend HTTP API (100 emails/day free tier, crisp deliverability)"),
            ("brevo_api", "Brevo HTTP API (300 emails/day free tier)"),
            ("draft_only", "Local Markdown review sheets only (configure sending later)"),
        ],
        0,
    )?
    .to_string();
    cfg.email_backend.sender_email = prompt_line("Your outreach sender email", &cfg.email_backend.sender_email)?;
    cfg.email_backend.sender_name = cfg.artist_name.clone();

    println!("\n--- STEP 4/4: INITIAL MARKET RESEARCH & TARGET BUYERS ---");
    let cities_raw = prompt_line("Target cities/regions (comma-separated)", &cfg.target_cities.join(", "))?;
    cfg.target_cities = cities_raw
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    cfg.target_verticals = prompt_multi(
        "Select high-multiplier buyer verticals for initial market research:",
        &[
            ("interior_designer", "Interior Designers & Luxury Residential Studios (repeat buyers!)"),
            ("art_consultant", "Art Consultants & Corporate Art Advisors (multi-piece budgets)"),
            ("hospitality", "Boutique Hotels, Restaurants & Hospitality Design Firms"),
            ("stager", "Luxury Real Estate Stagers & Model-Home Designers"),
            ("gallery", "Independent Contemporary Galleries"),
            ("architect", "Residential & Commercial Architecture Studios"),
            ("concept_store", "Design Showrooms, Concept Stores & Custom Framers"),
            ("press", "Design Magazines, Art Blogs & Architecture Substacks"),
        ],
        &[0, 1, 2, 3, 4],
    )?;

    cfg.outreach_angle = prompt_choice(
        "Primary outreach hook style (anti-fodder):",
        &[
            ("free-wall-mockup", "Offer a free 24h digital wall-mockup on their active client room"),
            ("connector-trade-program", "15-25% Trade Pricing + custom commission sizing"),
            ("private-viewing-room", "Exclusive link to your curated 5-work Online Viewing Room"),
            ("curatorial-alignment", "Quiet institutional note + Catalogue Raisonné link"),
            ("project-credits-snowball", "Warm local outreach via shared published project credits"),
            ("studio-letter-dispatch", "Intimate collector studio letter + real process video clip"),
        ],
        0,
    )?
    .to_string();

    Ok(cfg)
}

pub async fn run_setup(db_path: &Path, args: SetupArgs) -> Result<()> {
    let cfg: SetupConfig = if let Some(ref jf) = args.from_json {
        serde_json::from_str(
            &fs::read_to_string(jf)
                .with_context(|| format!("Failed to read setup JSON {}", jf.display()))?,
        )?
    } else if args.defaults {
        SetupConfig::default()
    } else {
        run_interactive_wizard()?
    };

    let ws = &args.workspace;
    let real_assets_dir = ws.join("real_assets");
    let research_dir = ws.join("market_research");
    let preview_dir = ws.join("preview");
    let batches_dir = ws.join("batches");
    let website_dir = ws.join("website");
    let videos_dir = ws.join("videos");

    for d in [
        ws,
        &real_assets_dir,
        &real_assets_dir.join("01_artworks"),
        &real_assets_dir.join("02_macro_details"),
        &real_assets_dir.join("03_studio_clips"),
        &real_assets_dir.join("04_artist_portrait"),
        &real_assets_dir.join("05_installations"),
        &research_dir,
        &preview_dir,
        &batches_dir,
        &website_dir,
        &videos_dir,
    ] {
        fs::create_dir_all(d)?;
    }

    // 1. Write real_assets README drop-guide
    let assets_readme = r#"# Drop Your REAL Artist Assets Here (Zero-AI-Art Protocol)

Place ONLY authentic photographs, flat high-res scans, and real studio videos here:
- `01_artworks/`       — Straight-on high-res photos/scans of finished physical artworks
- `02_macro_details/`  — Close-up texture / brushwork / material detail photos
- `03_studio_clips/`   — Short real phone/camera clips of hands working, mixing pigment, framing
- `04_artist_portrait/`— Real photograph of the artist in the studio (never AI-generated)
- `05_installations/`  — Real photos of artworks hung on walls or in exhibitions

After adding your files, register and verify them:
```bash
art-outreach assets register --dir art-outreach-workspace/real_assets
```
"#;
    fs::write(real_assets_dir.join("README.md"), assets_readme)?;

    // Copy or scan supplied assets_dir if provided
    let manifest_path = real_assets_dir.join("manifest.json");
    if let Some(ref src_dir) = args.assets_dir {
        if src_dir.exists() {
            let _ = assets::handle_command(AssetsCommand::Register {
                dir: src_dir.clone(),
                manifest: manifest_path.clone(),
                kind: "artwork".into(),
            })?;
        }
    } else {
        // Initialize empty or scan existing real_assets folder
        let _ = assets::handle_command(AssetsCommand::Register {
            dir: real_assets_dir.clone(),
            manifest: manifest_path.clone(),
            kind: "artwork".into(),
        })?;
    }

    // 2. Write artist_profile.json
    let signature = format!(
        "{}\nVisual Artist | {}\n{}{}",
        cfg.artist_name,
        cfg.studio_name,
        cfg.portfolio_url,
        if cfg.social_channels.instagram_handle.is_empty() {
            String::new()
        } else {
            format!(" | {}", cfg.social_channels.instagram_handle)
        }
    );
    let artist_profile = json!({
        "artist_name": cfg.artist_name,
        "studio_name": cfg.studio_name,
        "location": cfg.location,
        "medium": cfg.medium,
        "style_summary": cfg.style_summary,
        "style_keywords": cfg.style_keywords,
        "mood": cfg.mood,
        "size_range": cfg.size_range,
        "price_range": cfg.price_range,
        "price_tier": cfg.price_tier,
        "trade_discount": cfg.trade_discount,
        "representation": cfg.representation,
        "portfolio_url": cfg.portfolio_url,
        "lookbook_url": cfg.lookbook_url,
        "instagram_handle": cfg.social_channels.instagram_handle,
        "target_cities": cfg.target_cities,
        "target_verticals": cfg.target_verticals,
        "website_variation": cfg.website_variation,
        "video_formats": cfg.video_formats,
        "outreach_angle": cfg.outreach_angle,
        "signature": signature,
        "opt_out_line": "If sourcing original artwork isn't on your radar right now, just reply 'pass' and I won't follow up."
    });
    let profile_path = ws.join("artist_profile.json");
    fs::write(&profile_path, serde_json::to_string_pretty(&artist_profile)?)?;

    // 3. Write integrations.json (detecting local YouTube Shorts OAuth readiness automatically!)
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/slave".into());
    let yt_secret = PathBuf::from(&home).join(".config/agy-youtube/client_secret.json");
    let yt_token = PathBuf::from(&home).join(".config/agy-youtube/token.json");
    let integrations = json!({
        "social_channels": cfg.social_channels,
        "email_backend": cfg.email_backend,
        "detected_local_capabilities": {
            "youtube_shorts_oauth_client_secret": yt_secret.exists(),
            "youtube_shorts_oauth_token_ready": yt_token.exists(),
            "blender_installed": Path::new("/usr/bin/blender").exists(),
            "ffmpeg_installed": Path::new("/usr/bin/ffmpeg").exists(),
            "preview_server": "built-in Rust (`art-outreach preview serve`)"
        },
        "setup_notes": {
            "youtube_shorts": "To enable 1-command Shorts publishing, drop Google OAuth client_secret.json into ~/.config/agy-youtube/ and follow the youtube-shorts-uploader skill.",
            "instagram_tiktok_pinterest": "All vertical 9:16 (1080x1920) reels and caption decks are exported to art-outreach-workspace/videos/ ready to post or schedule."
        }
    });
    let integrations_path = ws.join("integrations.json");
    fs::write(&integrations_path, serde_json::to_string_pretty(&integrations)?)?;

    // 4. Write senders.json tailored to chosen email backend (secrets strictly via env:VAR)
    let sender_entry = match cfg.email_backend.mode.as_str() {
        "resend_api" => json!({
            "enabled": true,
            "type": "resend",
            "from_name": cfg.email_backend.sender_name,
            "email": cfg.email_backend.sender_email,
            "reply_to": cfg.email_backend.sender_email,
            "api_key": "env:RESEND_API_KEY",
            "daily_limit": 25
        }),
        "brevo_api" => json!({
            "enabled": true,
            "type": "brevo",
            "from_name": cfg.email_backend.sender_name,
            "email": cfg.email_backend.sender_email,
            "reply_to": cfg.email_backend.sender_email,
            "api_key": "env:BREVO_API_KEY",
            "daily_limit": 25
        }),
        "custom_smtp" => json!({
            "enabled": true,
            "type": "smtp",
            "from_name": cfg.email_backend.sender_name,
            "email": cfg.email_backend.sender_email,
            "reply_to": cfg.email_backend.sender_email,
            "smtp_host": "smtp.yourprovider.com",
            "smtp_port": 587,
            "smtp_user": cfg.email_backend.sender_email,
            "smtp_password": "env:ART_SMTP_PASSWORD_1",
            "imap_host": "imap.yourprovider.com",
            "imap_port": 993,
            "imap_drafts_folder": "Drafts",
            "daily_limit": 20
        }),
        _ => json!({
            "enabled": cfg.email_backend.mode != "draft_only",
            "type": "smtp",
            "from_name": cfg.email_backend.sender_name,
            "email": cfg.email_backend.sender_email,
            "reply_to": cfg.email_backend.sender_email,
            "smtp_host": "smtp.gmail.com",
            "smtp_port": 587,
            "smtp_user": cfg.email_backend.sender_email,
            "smtp_password": "env:ART_SMTP_PASSWORD_1",
            "imap_host": "imap.gmail.com",
            "imap_port": 993,
            "imap_drafts_folder": "[Gmail]/Drafts",
            "daily_limit": 20
        }),
    };
    let senders_path = ws.join("senders.json");
    fs::write(
        &senders_path,
        serde_json::to_string_pretty(&json!({
            "notes": "Set password/API key as an environment variable (e.g. export ART_SMTP_PASSWORD_1='...') so secrets never live on disk.",
            "mode": cfg.email_backend.mode,
            "senders": [sender_entry]
        }))?,
    )?;

    // 5. Initialize SQLite CRM
    let actual_db = if db_path == Path::new(crm::DEFAULT_DB_REL) {
        ws.join("data/art_leads.db")
    } else {
        db_path.to_path_buf()
    };
    crm::init_db(&actual_db)?;

    // 6. Generate Initial Market Research Pack (Queries + Research Brief + optional live OSM)
    let queries_path = research_dir.join("search_queries.json");
    discover::handle_command(
        &actual_db,
        DiscoverCommand::GenerateQueries {
            cities: cfg.target_cities.join(","),
            verticals: cfg.target_verticals.join(","),
            style: cfg.mood.clone(),
            medium: cfg.medium.clone(),
            output: Some(queries_path.clone()),
        },
    )
    .await?;

    if args.run_osm {
        if let Some(first_city) = cfg.target_cities.first() {
            let _ = discover::handle_command(
                &actual_db,
                DiscoverCommand::OsmLocal {
                    city: first_city.clone(),
                    verticals: "gallery,hospitality,interior_designer,architect,concept_store".into(),
                    limit: 80,
                },
            )
            .await;
        }
    }

    let web_vars = studio::website_variations();
    let chosen_web = web_vars
        .iter()
        .find(|v| v.slug == cfg.website_variation)
        .unwrap_or(&web_vars[0]);

    let research_brief = format!(
        r#"# Initial Market Research & Launch Dossier — {artist}

Generated by `art-outreach setup` | Strict Zero-AI-Art Protocol Active

## 1. Artist Positioning Snapshot
- **Artist / Studio**: {artist} ({studio}) — {loc}
- **Medium & Style**: {medium} — *{style}*
- **Scale & Pricing**: {size} | {price} (`{tier}` tier, `{rep}`)
- **Trade Offer**: {trade}

## 2. Popular Artist Portfolio Benchmark & Anti-Slop Direction
- **Primary Selected Archetype**: `{web_slug}` — **{web_name}**
- **Benchmark**: {bench}
- **Typography**: {typo}
- **Color Tokens**: {pal}
- **Preview All 6 Variations Live**: Run `art-outreach preview serve --dir {prev}` and open `http://127.0.0.1:4173`

## 3. Target Market & High-Multiplier Buyer Matrix
- **Target Cities**: {cities}
- **Selected Verticals**: {verts}
- **Generated Google Boolean Queries**: `{qpath}` (run these via your agent's `search_web` tool or `discover search-api`)
- **Free Local Map Scan**:
  ```bash
  art-outreach --db {db} discover osm-local --city "{first_city}" --limit 150
  ```

## 4. Social Media & 3D/2D Video Plan (Real Assets Only)
- **Selected Video Formats**: {vids}
- **Instagram**: {ig} | **YouTube**: {yt} | **Pinterest**: {pin} | **LinkedIn**: {li}

## 5. Immediate Next 4 Commands
1. Drop your **real** artwork scans/photos/clips into `{assets}` and run:
   `art-outreach assets register --dir {assets}`
2. Open the interactive **Live Preview Studio** to visually compare all 6 website variations, 6 video storyboards, and email templates:
   `art-outreach preview serve --dir {prev} --port 4173`
3. Check your sending domain deliverability (`{sender}`):
   `art-outreach deliverability {sender}`
4. Pull & enrich your first lead batch:
   `art-outreach --db {db} discover osm-local --city "{first_city}" --limit 100 && art-outreach --db {db} enrich --limit 100`
"#,
        artist = cfg.artist_name,
        studio = cfg.studio_name,
        loc = cfg.location,
        medium = cfg.medium,
        style = cfg.style_summary,
        size = cfg.size_range,
        price = cfg.price_range,
        tier = cfg.price_tier,
        rep = cfg.representation,
        trade = cfg.trade_discount,
        web_slug = chosen_web.slug,
        web_name = chosen_web.name,
        bench = chosen_web.artist_benchmark,
        typo = chosen_web.typography_pairing,
        pal = chosen_web.palette,
        prev = preview_dir.display(),
        cities = cfg.target_cities.join(", "),
        verts = cfg.target_verticals.join(", "),
        qpath = queries_path.display(),
        db = actual_db.display(),
        first_city = cfg.target_cities.first().map(|s| s.as_str()).unwrap_or("Austin"),
        vids = cfg.video_formats.join(", "),
        ig = if cfg.social_channels.instagram_handle.is_empty() { "-" } else { &cfg.social_channels.instagram_handle },
        yt = if cfg.social_channels.youtube_channel.is_empty() { "Ready for Shorts" } else { &cfg.social_channels.youtube_channel },
        pin = if cfg.social_channels.pinterest_handle.is_empty() { "-" } else { &cfg.social_channels.pinterest_handle },
        li = if cfg.social_channels.linkedin_url.is_empty() { "-" } else { &cfg.social_channels.linkedin_url },
        assets = real_assets_dir.display(),
        sender = cfg.email_backend.sender_email,
    );
    let brief_path = research_dir.join("research_brief.md");
    fs::write(&brief_path, research_brief)?;

    // 7. Immediately generate the Interactive Live Preview Studio!
    preview::generate_preview_studio(&actual_db, ws, &preview_dir)?;

    println!(
        "\n{}",
        serde_json::to_string_pretty(&json!({
            "status": "SETUP_COMPLETE",
            "workspace": ws.display().to_string(),
            "files_created": {
                "artist_profile": profile_path.display().to_string(),
                "integrations": integrations_path.display().to_string(),
                "senders": senders_path.display().to_string(),
                "real_assets_manifest": manifest_path.display().to_string(),
                "crm_database": actual_db.display().to_string(),
                "market_research_queries": queries_path.display().to_string(),
                "market_research_brief": brief_path.display().to_string(),
                "live_preview_studio": preview_dir.join("index.html").display().to_string(),
            },
            "preview_command": format!("art-outreach preview serve --dir {} --port 4173", preview_dir.display()),
        }))?
    );
    Ok(())
}
