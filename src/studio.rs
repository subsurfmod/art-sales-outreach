//! Anti-Slop Studio Variation Engine, ASCII Wireframe Generator & Master Launch Blueprint.
//!
//! Synthesizes:
//!   - Anthropic `frontend-design` & `prompting_for_frontend_aesthetics` anti-slop discipline
//!   - ASCII wireframe-first layout constraints (bareminimum.design / waiframe methodology)
//!   - `obra/superpowers` Brainstorming -> Spec -> Wireframe -> Plan -> Verify workflow
//!   - Museum-grade contemporary artist portfolio architecture (Viewing Rooms, Catalogue Raisonné, Academic CV)
//!   - Local installed 3D (Blender RTX 3050) & 2D (OneTake, MotionMaxxing, Remotion, HyperFrames, FFmpeg) skill routing
//!   - Strict Zero-AI-Artwork / Zero-AI-Artist Authenticity Protocol

use anyhow::{bail, Result};
use clap::Subcommand;
use serde::Serialize;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum StudioCommand {
    /// List all distinct anti-slop variations across Website/Portfolio, 3D & 2D Social Video, and Outreach
    Variations {
        /// Filter pillar: all, website, video, outreach
        #[arg(long, default_value = "all")]
        pillar: String,
    },
    /// Generate a box-drawn ASCII layout wireframe + anti-slop Design DNA tokens for a website/portfolio variation
    Wireframe {
        /// Variation slug: white-cube-monograph | catalogue-raisonne | editorial-brutalist | atmospheric-salon | tactile-studio-notebook | kinetic-exhibition
        #[arg(default_value = "white-cube-monograph")]
        variation: String,
    },
    /// Generate a bespoke 4-pillar Superpowers Launch Blueprint tailored to an artist's medium, mood, and price tier
    Blueprint {
        /// Primary medium (e.g. "oil painting", "ceramic sculpture", "charcoal drawing", "mixed media")
        #[arg(long)]
        medium: String,
        /// Visual mood (e.g. "quiet minimalist", "bold brutalist", "moody dark", "organic warm", "chromatic kinetic")
        #[arg(long)]
        mood: String,
        /// Price tier: emerging | mid-career | gallery-bluechip
        #[arg(long, default_value = "mid-career")]
        price_tier: String,
        /// Target city or region
        #[arg(long, default_value = "New York")]
        city: String,
        /// Output as JSON instead of Markdown
        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Serialize)]
pub struct WebsiteVariation {
    pub slug: &'static str,
    pub name: &'static str,
    pub artist_benchmark: &'static str,
    pub typography_pairing: &'static str,
    pub palette: &'static str,
    pub signature_layout: &'static str,
    pub best_for: &'static str,
    pub ascii_wireframe: &'static str,
}

#[derive(Debug, Serialize)]
pub struct VideoVariation {
    pub slug: &'static str,
    pub dimension: &'static str,
    pub name: &'static str,
    pub local_skills: &'static [&'static str],
    pub real_assets_required: &'static str,
    pub visual_concept: &'static str,
    pub platform_fit: &'static str,
}

#[derive(Debug, Serialize)]
pub struct OutreachVariation {
    pub slug: &'static str,
    pub name: &'static str,
    pub target_audience: &'static str,
    pub hook_mechanism: &'static str,
    pub anti_fodder_rule: &'static str,
}

pub fn website_variations() -> Vec<WebsiteVariation> {
    vec![
        WebsiteVariation {
            slug: "white-cube-monograph",
            name: "White-Cube Archival Monograph",
            artist_benchmark: "Agnes Martin / Lee Ufan / David Zwirner OVR",
            typography_pairing: "Cormorant Garamond (Display) + IBM Plex Mono (Placards/Dimensions)",
            palette: "Gallery Plaster #F7F5F0 | Graphite Ink #181816 | Linen Hairline #DFDBD2 | Ochre Accent #9A6B3F",
            signature_layout: "Massive negative space, single centered full-scale real work hero, toggleable 60\"-hang View-on-Wall scale simulator, discreet slide-out Collector Inquiry drawer (no 'Add to Cart')",
            best_for: "Minimalist, meditative, abstract, monochrome, and high-ticket fine art ($2,000–$25,000+)",
            ascii_wireframe: r#"
+------------------------------------------------------------------------------+
|  ARTIST NAME                      Works   Viewing Room   CV   Studio   Inquire|
+------------------------------------------------------------------------------+
|                                                                              |
|                        +------------------------------+                      |
|                        |                              |                      |
|                        |   [ REAL ARTWORK HERO SCAN ] |                      |
|                        |    (subtle 1px museum mat)   |                      |
|                        |                              |                      |
|                        +------------------------------+                      |
|           *Untitled IV*, 2026 — Oil & cold wax on raw linen, 60 x 48 in      |
|                 [ View to Scale on Wall ]   [ Inquire Privately ]            |
+------------------------------------------------------------------------------+
|  SELECTED SERIES                                                             |
|  +-----------------------+     +-----------------------+                     |
|  | [REAL PHOTO: SERIES A]|     |                       |                     |
|  | 12 Works · 2025–2026  |     | [REAL PHOTO: SERIES B]|  (Asymmetric offset |
|  +-----------------------+     | 08 Works · 2024–2025  |   gallery hang)     |
|                                +-----------------------+                     |
+------------------------------------------------------------------------------+
|  PRIVATE VIEWING ROOM  ->  "Autumn Linen Studies" (6 available works + PDF)  |
|  EXHIBITIONS & CV      ->  Solo / Group / Collections / Download Academic CV |
+------------------------------------------------------------------------------+"#,
        },
        WebsiteVariation {
            slug: "catalogue-raisonne",
            name: "Scholarly Catalogue Raisonné Index",
            artist_benchmark: "Gerhard Richter Online Archive / Institutional Oeuvre Database",
            typography_pairing: "Newsreader (Editorial) + JetBrains Mono (Catalogue IDs & Provenance)",
            palette: "Archival Paper #F3F1EC | Carbon #111111 | Slate Meta #64625D | Cobalt Status #1E4E8C",
            signature_layout: "Filterable database ledger by Year / Series / Medium / Availability + sticky high-res deep-zoom plate viewer with provenance & exhibition history",
            best_for: "Prolific painters, sculptors, printmakers, art consultants, and museum curators needing instant inventory lookup",
            ascii_wireframe: r#"
+------------------------------------------------------------------------------+
|  ARTIST ARCHIVE [194 WORKS]   Filter: [All Years v] [Medium v] [Available v] |
+------------------------------------+-----------------------------------------+
|  INDEX LEDGER                      |  SELECTED PLATE (DEEP ZOOM)             |
|  --------------------------------  |  +-----------------------------------+  |
| > CR-2026-04 | Nocturne II | Avail |  |                                   |  |
|   CR-2026-03 | Salt Flat   | Sold  |  |    [ REAL ARTWORK HIGH-RES ]      |  |
|   CR-2026-02 | Basalt Study| Avail |  |    [ + Detail Crop 1 ] [ + Crop 2]|  |
|   CR-2025-19 | Iron Study  | Hold  |  +-----------------------------------+  |
|   CR-2025-18 | Drift IV    | Sold  |  CR-2026-04 · *Nocturne II* (2026)      |
|   CR-2025-17 | Ash Line    | Avail |  Oil on Belgian linen · 152 x 122 cm    |
|                                    |  Provenance: Artist Studio              |
|  [ Switch: Table | Contact Sheet ] |  [ Request Condition & Trade Sheet ]    |
+------------------------------------+-----------------------------------------+
|  SCHOLARLY CV  |  EXHIBITION CHECKLISTS  |  PRESS BIBLIOGRAPHY               |
+------------------------------------------------------------------------------+"#,
        },
        WebsiteVariation {
            slug: "editorial-brutalist",
            name: "Editorial Brutalist Manifesto",
            artist_benchmark: "Jenny Holzer / Barbara Kruger / Kunsthalle Exhibition Posters",
            typography_pairing: "Syne (Oversized Heavy Display) + Space Mono (Rigid Grid Captions)",
            palette: "Raw Newsprint #F2EFE9 | Pitch Ink #0D0D0C | Vermilion Alert #D9381E | Concrete #C8C4BC",
            signature_layout: "Full-bleed razor-sharp 1px structural borders, giant typographic exhibition masthead, stark asymmetric photography blocks, direct studio manifest",
            best_for: "Bold contemporary, political, urban, mixed-media, expressive figurative, and sculptural statement works",
            ascii_wireframe: r#"
+==============================================================================+
|  LASTNAME  //  STUDIO  //  CITY                 [AVAILABLE]  [CV]  [INQUIRE] |
+==============================================================================+
|  RAW                                 | +-----------------------------------+ |
|  MATTER                              | |                                   | |
|  2026                                | |    [ REAL ARTWORK FULL-BLEED ]    | |
|  ----------------------------------- | |                                   | |
|  CURRENT BODY OF WORK:               | +-----------------------------------+ |
|  14 Large-Scale Works on Steel &     | FIG 01. 72x54 IN / DIRECT FROM STUDIO |
|  Charred Cedar.                      |                                       |
|  [ ENTER VIEWING ROOM -> ]           | [ NEXT PLATE 01/14 -> ]               |
+--------------------------------------+---------------------------------------+
|  01 WORKS        |  02 STUDIO VIDEO (REAL)   |  03 EXHIBITION LEDGER         |
|  [REAL THUMB]    |  [REAL PROCESS CLIP]      |  2026 Solo — Gallery X        |
|  [REAL THUMB]    |  00:42 · Silent Studio    |  2025 Group — Kunsthalle Y    |
+------------------+---------------------------+-------------------------------+"#,
        },
        WebsiteVariation {
            slug: "atmospheric-salon",
            name: "Atmospheric Nocturnal Salon",
            artist_benchmark: "Mark Rothko / James Turrell / Dark-Wall Private Viewing Room",
            typography_pairing: "Instrument Serif (Poetic Display) + DM Sans / Mono (Quiet Captions)",
            palette: "Charcoal Velvet #111214 | Warm Bone #ECE7DD | Muted Stone #8A857C | Brass Highlight #C8A25A",
            signature_layout: "Dark museum gallery walls that make luminous paintings glow, spotlight vignette framing around real artwork photos, slow chaptered scroll narrative",
            best_for: "Moody landscapes, chiaroscuro oils, glowing abstracts, bronze/glass sculpture, luxury hospitality collections",
            ascii_wireframe: r#"
+------------------------------------------------------------------------------+
|  ARTIST                                       Viewing Room   Archive   Contact|
+------------------------------------------------------------------------------+
|:::::::::::::::::::::::::::: DARK GALLERY WALL :::::::::::::::::::::::::::::::|
|                                                                              |
|                   +------------------------------------+                     |
|                   |                                    |                     |
|                   |     [ REAL PAINTING ILLUMINATED ]  |                     |
|                   |        (soft museum wall glow)     |                     |
|                   |                                    |                     |
|                   +------------------------------------+                     |
|                                                                              |
|                   "Light Held in Mineral Pigment" — 2026                     |
|                        Scroll slowly to enter the room                       |
+------------------------------------------------------------------------------+
|  CHAPTER I: STUDIO LIGHT          |  [ REAL MACRO TEXTURE PHOTO ]            |
|  Curatorial essay in narrow       |  Side-by-side detail of raw pigment      |
|  48ch literary column...          |  and linen weave.                        |
+-----------------------------------+------------------------------------------+
|  PRIVATE ACQUISITION CONCIERGE ->  Request Trade PDF or Wall Mockup          |
+------------------------------------------------------------------------------+"#,
        },
        WebsiteVariation {
            slug: "tactile-studio-notebook",
            name: "Tactile Studio Notebook & Material Archive",
            artist_benchmark: "Olafur Eliasson Studio / Louise Bourgeois Process Diaries",
            typography_pairing: "Fraunces (Warm Tactile Serif) + JetBrains Mono (Material Recipes/Dates)",
            palette: "Raw Linen #EFEAE0 | Walnut Ink #261E18 | Terracotta #B85233 | Sage Note #5D6F5B",
            signature_layout: "Two-column sketchbook rhythm pairing finished real artworks with real studio process photos, material notes, pigment provenance, and studio visit booking",
            best_for: "Ceramics, textiles, natural pigment painters, printmakers, botanical/earth artists, and collectors who buy the story of craft",
            ascii_wireframe: r#"
+------------------------------------------------------------------------------+
|  STUDIO OF ARTIST NAME    Works · Material Notes · Studio Visits · Inquiries |
+------------------------------------------------------------------------------+
|  [ REAL STUDIO PHOTO: HANDS / KILN / EASEL ]   |  STUDIO NOTE #14            |
|                                                |  "Hand-ground ochre from    |
|                                                |   quarry clay + cold wax."  |
|                                                |  -> See 6 works in series   |
+------------------------------------------------+-----------------------------+
|  WORK PLATE                      |  DETAIL + PROVENANCE                      |
|  +----------------------------+  |  +-------------------+                    |
|  | [ REAL FINISHED ARTWORK ]  |  |  | [REAL MACRO CROP] |  Dimensions: 40x30 |
|  |                            |  |  +-------------------+  Status: Available |
|  +----------------------------+  |  [ Book Studio Visit ] [ Inquire ]        |
+----------------------------------+-------------------------------------------+"#,
        },
        WebsiteVariation {
            slug: "kinetic-exhibition",
            name: "Kinetic Exhibition & VIP Collector Portal",
            artist_benchmark: "Pace Live / Hauser & Wirth Digital Exhibition Launch",
            typography_pairing: "Bricolage Grotesque (Contemporary Display) + Crimson Pro (Curatorial Essay)",
            palette: "Crisp Chalk #FAF9F6 | Obsidian #141417 | Ultramarine #1C3FD9 | Warm Gray #737278",
            signature_layout: "Horizontal gallery-wall rail of real framed works with snap-scroll, live 'Available / On Hold / Collected' status pills, password-gated VIP Advisor Room",
            best_for: "Collection drops, solo show launches, interior designer trade portals, and artists running social video ad traffic to a high-converting landing page",
            ascii_wireframe: r#"
+------------------------------------------------------------------------------+
|  ARTIST · CURRENT DROP: "SERIES VIII"     [ VIP Trade Login ]   [ Inquire ]  |
+------------------------------------------------------------------------------+
|  HORIZONTAL GALLERY WALL (DRAG / SCROLL ->)                                  |
|  +--------------------+  +--------------------+  +--------------------+      |
|  | [REAL WORK 01]     |  | [REAL WORK 02]     |  | [REAL WORK 03]     |  --> |
|  | 48x60" · AVAILABLE |  | 36x48" · ON HOLD   |  | 60x72" · AVAILABLE |      |
|  +--------------------+  +--------------------+  +--------------------+      |
+------------------------------------------------------------------------------+
|  FOR INTERIOR DESIGNERS & ADVISORS        |  FOR PRIVATE COLLECTORS          |
|  - 20% Trade Pricing                      |  - Direct Studio Crated Shipping |
|  - Free Digital Wall Mockup (24h)         |  - Certificate of Authenticity   |
|  [ Download 5-Work Trade Sheet ]          |  [ Request Private Viewing ]     |
+-------------------------------------------+----------------------------------+"#,
        },
    ]
}

pub fn video_variations() -> Vec<VideoVariation> {
    vec![
        VideoVariation {
            slug: "blender-3d-raking-light",
            dimension: "3D",
            name: "Blender 3D Raking-Light Gallery Inspection",
            local_skills: &["blender-film", "ffmpeg", "youtube-shorts-uploader"],
            real_assets_required: "1 high-res flat real scan/photo of artwork (+ optional real macro detail photo)",
            visual_concept: "Maps the artist's unaltered real scan onto a physically accurate 3D stretched canvas (real depth, gallery frame bevel) on a plaster wall; sweeps a warm raking directional sunbeam & shallow-DOF handheld camera across the canvas to evoke physical presence (RTX 3050 EEVEE Raytracing).",
            platform_fit: "Instagram Reels, YouTube Shorts, TikTok, Website Hero Loop (9:16 & 16:9)",
        },
        VideoVariation {
            slug: "blender-3d-architectural-scale",
            dimension: "3D",
            name: "Blender 3D Architectural Room Scale Staging",
            local_skills: &["blender-film", "blender-forest", "ffmpeg"],
            real_assets_required: "1–3 high-res real artwork scans with exact physical cm/inch dimensions",
            visual_concept: "Places the real artwork at exact 1:1 physical scale inside a minimalist 3D residential living room, limestone loft, or boutique hotel lobby with natural window light so interior designers & collectors immediately judge scale.",
            platform_fit: "LinkedIn (for Interior Designers/Advisors), Pinterest Video Pins, Private Viewing Room embeds",
        },
        VideoVariation {
            slug: "onetake-kinetic-exhibition",
            dimension: "2D",
            name: "OneTake Single-Shot Kinetic Exhibition Teaser",
            local_skills: &["onetake", "motion-doctrine", "ffmpeg"],
            real_assets_required: "3–6 real artwork photographs + 2 real macro texture crops",
            visual_concept: "Pure-function-of-time HTML composition with real shutter motion blur: camera glides from a real macro texture detail out to the full framed painting, carrying seamlessly into museum placard typography (Title, Medium, Dimensions) and room-matched subtle acoustics.",
            platform_fit: "Instagram Reels, YouTube Shorts, Exhibition Launch Announcements (15–25s)",
        },
        VideoVariation {
            slug: "motionmaxxing-collector-film",
            dimension: "2D+3D",
            name: "MotionMaxxing Anti-Slop Collection Launch Film",
            local_skills: &["motionmaxxing", "hyperframes", "motion-doctrine"],
            real_assets_required: "Real finished artwork photos + optional real 5–10s studio video clips",
            visual_concept: "Senior-motion-designer commercial pacing (measured GSAP curves, zero HUD chrome, zero cheesy stat slides): opens on real raw studio texture, holds stillness before revealing the hero piece, closes quietly on the Viewing Room URL.",
            platform_fit: "YouTube 16:9 Collection Launch, Press Kits, Paid Meta/Instagram Collector Ads (25–45s)",
        },
        VideoVariation {
            slug: "remotion-collector-story",
            dimension: "2D",
            name: "Remotion Narrated Studio Story & Provenance Short",
            local_skills: &["remotion-best-practices", "remotion-captions", "remotion-render", "youtube-shorts-uploader"],
            real_assets_required: "Real studio process clips + real photos of the piece (plus artist's real voiceover or clean text narration)",
            visual_concept: "Programmatic 9:16 vertical storytelling reel explaining the physical material, inspiration, and making of ONE specific piece using real footage, crisp editorial word-timed captions, and a direct Collector Inquiry CTA — ready for automated YouTube Shorts publishing.",
            platform_fit: "YouTube Shorts, Instagram Reels, TikTok (25–40s)",
        },
        VideoVariation {
            slug: "macro-split-process",
            dimension: "2D",
            name: "Split-Screen Macro Detail & Signature Reveal",
            local_skills: &["ffmpeg", "motion-broll", "pixel2motion"],
            real_assets_required: "Real studio work-in-progress video + real finished artwork photo (+ optional real artist signature scan for SVG draw-on)",
            visual_concept: "Tactile side-by-side / wipe comparison cutting cleanly between the artist's real hands/studio video and the finished real artwork, closing with a clean `pixel2motion` stroke-draw animation of the artist's actual handwritten signature.",
            platform_fit: "Instagram Reels, TikTok, Studio Newsletter GIF/MP4 Embeds (10–18s)",
        },
    ]
}

pub fn outreach_variations() -> Vec<OutreachVariation> {
    vec![
        OutreachVariation {
            slug: "connector-trade-program",
            name: "Connector-Multiplier Trade Partnership",
            target_audience: "Interior Designers & Residential Architects",
            hook_mechanism: "References a real aesthetic/project from their website + offers 15–25% trade pricing, custom commission sizing, and crated studio delivery.",
            anti_fodder_rule: "Never attach heavy PDFs on Email #1; never write 'Dear Sir/Madam'; stay under 110 words.",
        },
        OutreachVariation {
            slug: "free-wall-mockup",
            name: "Zero-Risk Digital Wall-Mockup Offer",
            target_audience: "Interior Designers, Hospitality Buyers & Luxury Stagers",
            hook_mechanism: "Offers to drop 2–3 real artworks to exact physical scale onto a phone photo of a blank wall they are currently styling — free, within 24 hours.",
            anti_fodder_rule: "Lead with usefulness to their active project rather than asking them to browse a shop.",
        },
        OutreachVariation {
            slug: "private-viewing-room",
            name: "Curated Private Viewing Room Invitation",
            target_audience: "Art Consultants, Corporate Curators & Past Inquiries",
            hook_mechanism: "Sends a clean link to a dedicated 5-piece Online Viewing Room page tailored to their region/palette (built via the Website Pillar).",
            anti_fodder_rule: "Curate 5 cohesive works max; never dump 60 unedited pieces.",
        },
        OutreachVariation {
            slug: "curatorial-alignment",
            name: "Quiet Institutional / Gallery Note",
            target_audience: "Independent Contemporary Galleries & Museum Curators",
            hook_mechanism: "Short peer-to-peer note referencing one specific recent exhibition in their program + link to the artist's Catalogue Raisonné / CV.",
            anti_fodder_rule: "Zero sales discounts, zero hype adjectives, zero bulk blasting.",
        },
        OutreachVariation {
            slug: "project-credits-snowball",
            name: "Project-Credits Mutual Network Chain",
            target_audience: "Architects, Luxury Builders, Stagers & Stylists credited on local homes",
            hook_mechanism: "References a real published local home/hotel project they collaborated on and introduces complementary physical scale artwork.",
            anti_fodder_rule: "Always record the shared project in CRM `connection_notes` for genuine local warmth.",
        },
        OutreachVariation {
            slug: "studio-letter-dispatch",
            name: "Intimate Studio Dispatch & Process Reel",
            target_audience: "Direct Collectors, Design Press & Lifestyle Editors",
            hook_mechanism: "Shares a real 20-second studio process clip or raking-light 3D gallery video alongside the story of the physical materials.",
            anti_fodder_rule: "Reads like a personal letter from a working studio, never a promotional newsletter blast.",
        },
    ]
}

pub fn handle_command(cmd: StudioCommand) -> Result<()> {
    match cmd {
        StudioCommand::Variations { pillar } => {
            let p = pillar.to_lowercase();
            let out = match p.as_str() {
                "website" => json!({ "website_variations": website_variations() }),
                "video" => json!({ "video_variations": video_variations() }),
                "outreach" => json!({ "outreach_variations": outreach_variations() }),
                _ => json!({
                    "protocol": "STRICT_REAL_ARTWORK_ONLY_V1 (Zero AI-generated artwork or artist likeness)",
                    "website_variations": website_variations(),
                    "video_variations": video_variations(),
                    "outreach_variations": outreach_variations(),
                }),
            };
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        StudioCommand::Wireframe { variation } => {
            let vars = website_variations();
            let Some(v) = vars.into_iter().find(|x| x.slug == variation.to_lowercase()) else {
                bail!("Unknown website variation '{variation}'. Valid slugs: white-cube-monograph, catalogue-raisonne, editorial-brutalist, atmospheric-salon, tactile-studio-notebook, kinetic-exhibition");
            };
            println!("# {} (`{}`)\n", v.name, v.slug);
            println!("- **Artist / Gallery Benchmark**: {}", v.artist_benchmark);
            println!("- **Anti-Slop Typography**: {}", v.typography_pairing);
            println!("- **Color Tokens**: {}", v.palette);
            println!("- **Signature Layout**: {}", v.signature_layout);
            println!("- **Best For**: {}", v.best_for);
            println!("\n## ASCII Layout Wireframe (Lock Structure Before Coding)\n```text{}\n```", v.ascii_wireframe);
        }
        StudioCommand::Blueprint {
            medium,
            mood,
            price_tier,
            city,
            json: as_json,
        } => {
            let m_low = format!("{} {}", medium.to_lowercase(), mood.to_lowercase());

            let web_slug = if m_low.contains("dark") || m_low.contains("moody") || m_low.contains("nocturn") || m_low.contains("chiaroscuro") {
                "atmospheric-salon"
            } else if m_low.contains("brutal") || m_low.contains("bold") || m_low.contains("urban") || m_low.contains("steel") {
                "editorial-brutalist"
            } else if m_low.contains("ceramic") || m_low.contains("textile") || m_low.contains("organic") || m_low.contains("clay") || m_low.contains("earth") {
                "tactile-studio-notebook"
            } else if m_low.contains("kinetic") || m_low.contains("pop") || m_low.contains("chromatic") || m_low.contains("drop") {
                "kinetic-exhibition"
            } else if price_tier == "gallery-bluechip" {
                "catalogue-raisonne"
            } else {
                "white-cube-monograph"
            };

            let web_list = website_variations();
            let web = web_list.into_iter().find(|v| v.slug == web_slug).unwrap();

            let vid_list = video_variations();
            let primary_3d = &vid_list[0]; // blender-3d-raking-light
            let secondary_3d = &vid_list[1]; // blender-3d-architectural-scale
            let primary_2d = if m_low.contains("ceramic") || m_low.contains("textile") || m_low.contains("studio") {
                &vid_list[5] // macro-split-process
            } else if m_low.contains("bold") || m_low.contains("kinetic") {
                &vid_list[3] // motionmaxxing-collector-film
            } else {
                &vid_list[2] // onetake-kinetic-exhibition
            };
            let story_short = &vid_list[4]; // remotion-collector-story

            let out_list = outreach_variations();
            let primary_outreach = if price_tier == "gallery-bluechip" {
                &out_list[2] // private-viewing-room
            } else {
                &out_list[1] // free-wall-mockup
            };
            let secondary_outreach = &out_list[0]; // connector-trade-program

            if as_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "artist_context": { "medium": medium, "mood": mood, "price_tier": price_tier, "city": city },
                        "authenticity_protocol": "STRICT_REAL_ARTWORK_ONLY_V1 — run `art-outreach assets audit` before shipping any website or video",
                        "pillar_1_website_and_portfolio": web,
                        "pillar_2_video_3d": [primary_3d, secondary_3d],
                        "pillar_2_video_2d": [primary_2d, story_short],
                        "pillar_3_outreach": [primary_outreach, secondary_outreach],
                    }))?
                );
                return Ok(());
            }

            println!("# Master Art Seller Blueprint — {} ({}, {})\n", medium, mood, city);
            println!("> **HARD AUTHENTICITY PROTOCOL**: Zero AI-generated images of artwork or the artist. Register real studio photos first with `art-outreach assets register --dir real_assets` and pass `art-outreach assets audit` before publishing.\n");
            println!("## 1. Website & Portfolio Architecture (Anti-Slop)");
            println!("- **Selected Variation**: {} (`{}`)", web.name, web.slug);
            println!("- **Benchmark**: {}", web.artist_benchmark);
            println!("- **Typography**: {}", web.typography_pairing);
            println!("- **Palette**: {}", web.palette);
            println!("```text{}\n```\n", web.ascii_wireframe);
            println!("## 2. Social Media 3D & 2D Video Pipeline (Real Assets Only)");
            println!("- **3D Hero Reel**: `{}` — {} (Skills: `{}`)", primary_3d.slug, primary_3d.name, primary_3d.local_skills.join(", "));
            println!("- **3D Scale Proof**: `{}` — {} (Skills: `{}`)", secondary_3d.slug, secondary_3d.name, secondary_3d.local_skills.join(", "));
            println!("- **2D Motion Cut**: `{}` — {} (Skills: `{}`)", primary_2d.slug, primary_2d.name, primary_2d.local_skills.join(", "));
            println!("- **Narrated Short**: `{}` — {} (Skills: `{}`)\n", story_short.slug, story_short.name, story_short.local_skills.join(", "));
            println!("## 3. High-Reach Outreach Strategy (Non-Fodder)");
            println!("- **Primary Angle**: {} (`{}`) — {}", primary_outreach.name, primary_outreach.slug, primary_outreach.hook_mechanism);
            println!("- **Secondary Angle**: {} (`{}`) — {}", secondary_outreach.name, secondary_outreach.slug, secondary_outreach.hook_mechanism);
            println!("- **Discovery Seed**: `art-outreach discover generate-queries --cities \"{city}\" --medium \"{medium}\" --style \"{mood}\"`\n");
            println!("## 4. Superpowers Verification Gates");
            println!("1. `art-outreach assets register --dir real_assets` (0 AI suspects)");
            println!("2. Approve ASCII wireframe before coding website");
            println!("3. `art-outreach assets audit --target ./website` (0 violations)");
            println!("4. `art-outreach deliverability <your-domain>` (score >= 90)");
            println!("5. Approve Markdown email batch + `--dry-run` before live outreach");
        }
    }
    Ok(())
}
