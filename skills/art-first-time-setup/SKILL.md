---
name: art-first-time-setup
description: >-
  Ultra-easy guided First-Time Setup wizard for the Art Seller Assistant.
  Walks the artist through portfolio & REAL artwork intake (Zero-AI-Art
  protocol), social media & email integrations, website/video/outreach
  variation choices, and automated initial market research, then generates an
  interactive Live Preview Studio before building or publishing anything.
  Use whenever a new artist wants to set up their art sales workspace,
  onboard their portfolio/social accounts, run initial market research, or
  explore all available options easily.
---

# Art Seller First-Time Setup Wizard

Makes onboarding effortless by offering clear multiple-choice options instead of blank forms, scaffolding the entire workspace in one Rust command (`art-outreach setup`), running initial market research automatically, and immediately generating a live visual preview.

> **STRICT ZERO-AI-ART PROTOCOL**: Never generate AI images of the artwork or the artist. Always collect or scaffold a `real_assets/` folder for authentic photographs, flat scans, and real studio clips.

---

## Effortless 3-Step Agent Onboarding Flow

When a user triggers first-time setup, use **clickable multiple-choice `ask_question` rounds** (keep it fast — 2 compact question rounds max, giving plenty of clear options!):

### Round 1 — Portfolio, Artwork & Style Options
Ask with `ask_question`:
1. **How would you like to bring in your artwork & portfolio?**
   - `(Recommended) Point to a local folder of my REAL artwork photos/scans/studio clips`
   - `Create organized starter folders (01_artworks, 02_macro_details, 03_studio_clips, 04_artist_portrait) so I can drop files in`
   - `I have an existing website / Instagram / Behance URL for my bio & CV + I will add local real photos`
2. **Which market tier & pricing structure fits your physical work?**
   - `(Recommended) Mid-Career ($1,500–$12,000 — Interior Designers, Art Consultants, Galleries & Collectors)`
   - `Emerging ($250–$2,000 — Direct Collectors, Local Shops, Concept Stores & Press)`
   - `Gallery / Blue-Chip ($10,000+ — Private Viewing Rooms, Catalogue Raisonné, Institutional Advisors)`
3. **Which anti-slop Website & Portfolio archetype feels closest to your work?** *(Remind them they will preview ALL 6 live right after setup!)*
   - `(Recommended) White-Cube Archival Monograph (Agnes Martin / Zwirner quiet stillness + 60" View-on-Wall)`
   - `Catalogue Raisonné Index (Gerhard Richter filterable oeuvre ledger + deep-zoom provenance)`
   - `Editorial Brutalist Manifesto (Jenny Holzer high-contrast exhibition poster grid)`
   - `Atmospheric Nocturnal Salon (Rothko / Turrell dark-wall luminous private viewing room)`
   - `Tactile Studio Notebook (Olafur Eliasson craft, material notes & studio process archive)`
   - `Kinetic Exhibition & VIP Drop Portal (Pace / Hauser & Wirth horizontal gallery rail + Advisor login)`

### Round 2 — Social Media, Video Formats, Email & Market Research (Multi-Select!)
Ask with `ask_question` (`is_multi_select: true` where appropriate):
1. **Which social media & publishing channels do you want integrated?** *(Multi-select)*
   - `Instagram (Reels 9:16 + Feed 4:5 + Collector DM/Bio funnel)`
   - `YouTube Shorts (automated upload via youtube-shorts-uploader)`
   - `Pinterest (Video Pins — top discovery engine for Interior Designers)`
   - `LinkedIn (3D architectural scale reels for Art Consultants & Hospitality)`
   - `TikTok (9:16 studio process & reveal reels)`
   - `Studio Newsletter / Substack`
2. **Which 3D & 2D artwork video formats do you want prepared (using ONLY your real photos/clips)?** *(Multi-select)*
   - `3D Blender Raking-Light Gallery Inspection (real scan on 3D stretched canvas)`
   - `3D Blender Architectural Room Scale Staging (1:1 scale in loft / hotel lobby)`
   - `2D OneTake Single-Shot Kinetic Exhibition Teaser (macro detail -> full work)`
   - `2D+3D MotionMaxxing Anti-Slop Collection Launch Film`
   - `2D Remotion 9:16 Narrated Studio Story Short`
   - `2D FFmpeg Split-Screen Process + Real Signature Reveal (pixel2motion)`
3. **How do you want to send outreach emails?**
   - `(Recommended) Gmail / Outlook Native Drafts (uploads directly to your Drafts folder — zero risk, review & click Send)`
   - `Automated Throttled SMTP (Gmail / Google Workspace / Zoho / Fastmail App Password)`
   - `Resend API (100 emails/day free tier) or Brevo API (300 emails/day free tier)`
   - `Local Markdown Review Sheets only for now (set up sending later)`
4. **Which high-multiplier buyer verticals should initial market research target first?** *(Multi-select)*
   - `Interior Designers & Luxury Residential Studios`
   - `Art Consultants & Corporate Art Advisors`
   - `Boutique Hotels & Hospitality Design Firms`
   - `Luxury Home Stagers & Model-Home Designers`
   - `Independent Contemporary Galleries`
   - `Residential & Commercial Architects`
   - `Design Showrooms, Concept Stores & Custom Framers`
   - `Design Magazines, Art Blogs & Architecture Substacks`

*(Also ask in plain text for: Artist Name, City, Medium/Style 1-liner, Target Cities, and optional folder path of real artwork photos!)*

---

## Execute One-Shot Rust Setup & Initial Market Research

1. Save the user's answers to `art-outreach-workspace/setup_answers.json` matching `SetupConfig`.
2. Run the unified Rust setup engine (creates folders, registers real assets with SHA-256 + AI screening, writes `artist_profile.json`, `integrations.json`, `senders.json`, initializes SQLite CRM, generates Google Boolean market research queries + `research_brief.md`, and builds the **Live Preview Studio**):
   ```bash
   ART_CLI=/home/slave/.agents/skills/art-sales-outreach/bin/art-outreach
   "$ART_CLI" setup --from-json art-outreach-workspace/setup_answers.json \
     --workspace art-outreach-workspace --run-osm
   ```
   *(Or for terminal-only interactive use: `"$ART_CLI" setup`.)*
3. **Run Live Market Research Kickoff**:
   - Read `art-outreach-workspace/market_research/search_queries.json`.
   - Run 4–6 of the highest-priority queries in parallel with `search_web` to pull an immediate first batch of real studios/consultants/galleries in the artist's target cities, import them via `art-outreach crm import-urls`, and enrich/score them!
4. **Hand Off Immediately to Preview (`art-preview-studio`)**:
   - Start or print the command for the **Live Preview Studio** (`art-outreach preview serve --dir art-outreach-workspace/preview --port 4173`) so the artist can visually inspect all 6 website archetypes, 60" wall scale, 3D/2D video storyboards, email pitches, and market research in one browser tab before committing!
