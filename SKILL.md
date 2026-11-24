---
name: art-sales-outreach
description: >-
  Full-fledged Rust-powered Art Seller Assistant for physical artwork:
  effortless first-time setup wizard (portfolio/artwork intake, social media
  integrations, initial market research), interactive local Live Preview Studio
  (preview all 6 website archetypes, 60" wall scale, 3D/2D video storyboards,
  and email sequences before shipping), strict Zero-AI-Artwork/Artist
  authenticity auditing, anti-slop artist websites & museum-grade portfolios,
  3D (Blender) & 2D (OneTake/MotionMaxxing/Remotion/HyperFrames/FFmpeg) social
  media video advertising + YouTube Shorts uploading, and high-deliverability
  connector-multiplier lead discovery, website enrichment, MX verification, and
  multi-inbox email outreach. Use whenever the user wants to set up an art
  business/workspace, sell physical artwork, build or preview an artist website
  or portfolio, create/preview 3D/2D artwork promo videos for social media,
  find art buyers/connectors, or run personalized art sales email campaigns.
---

# Art Seller Assistant & Outreach Engine (Rust Edition)

A complete, museum-grade operating system for selling **physical artwork** — powered by a single native Rust CLI (`art-outreach`), featuring an effortless **First-Time Setup Wizard**, an interactive **Live Preview Studio**, packed preview sub-skills, and strict **anti-slop differentiation** so every artist gets unique websites, portfolios, social videos, and outreach campaigns instead of generic AI fodder.

> **NON-NEGOTIABLE AUTHENTICITY PROTOCOL (ZERO AI ART / ZERO AI ARTIST)**
> NEVER generate AI images or AI video frames of the artwork or the artist (no `image-generator`, Midjourney, DALL-E, Flux, Stable Diffusion, HeyGen avatars, or Unsplash/Pexels stock art). Every artwork photo, macro detail, studio clip, and artist portrait MUST be an authentic file supplied by the artist, registered with `art-outreach assets register`, and verified with `art-outreach assets audit` before shipping. See [superpowers_workflow_and_authenticity.md](./references/superpowers_workflow_and_authenticity.md).

## Packed Companion Skills (Included in `./skills/`)

- **`art-first-time-setup`** ([skills/art-first-time-setup/SKILL.md](./skills/art-first-time-setup/SKILL.md)) — Effortless multi-choice onboarding wizard (portfolio/real artwork intake, social media & email integrations, initial market research).
- **`art-preview-studio`** ([skills/art-preview-studio/SKILL.md](./skills/art-preview-studio/SKILL.md)) — Live localhost Preview Studio (`art-outreach preview serve`), desktop/mobile screenshot capture, and pre-send email dry-run inspection.
- **`art-video-preview-bridge`** ([skills/art-video-preview-bridge/SKILL.md](./skills/art-video-preview-bridge/SKILL.md)) — Pre-visualization bridge for Blender 3D clay/viewport snaps, HyperFrames preview, Remotion Studio/stills, OneTake seekable contact sheets, FFmpeg 3×3 storyboards, and YouTube Shorts `--dry-run`.

## Binary & Workspace Setup

The compiled Rust release binary lives at `./bin/art-outreach` (build anytime from source with `cargo build --release`):

```bash
ART_CLI=/home/slave/.agents/skills/art-sales-outreach/bin/art-outreach
# If rebuilding from source:
cargo build --release --manifest-path /home/slave/.agents/skills/art-sales-outreach/Cargo.toml
```

---

## Step 0: Effortless First-Time Setup & Instant Live Preview (Start Here!)

If `art-outreach-workspace/artist_profile.json` does not exist yet, activate **`art-first-time-setup`** (walk the user through clickable multiple-choice options for their portfolio/real artwork folder, social channels, email backend, and target markets) and run:

```bash
# Interactive terminal wizard:
"$ART_CLI" setup

# Or one-shot from agent-collected JSON answers (+ optional live local OpenStreetMap market scan):
"$ART_CLI" setup --from-json art-outreach-workspace/setup_answers.json --run-osm
```

This single command automatically scaffolds:
- `real_assets/` (`01_artworks/`, `02_macro_details/`, `03_studio_clips/`, `04_artist_portrait/`, `05_installations/`) + SHA-256 `manifest.json`
- `artist_profile.json`, `integrations.json` (auto-detecting local Blender, FFmpeg, YouTube Shorts OAuth), and `senders.json`
- Embedded SQLite CRM (`data/art_leads.db`)
- Initial Market Research (`market_research/search_queries.json` + `research_brief.md`)
- **Interactive Local Preview Studio (`preview/index.html`)**

### Immediately Preview Before Building or Sending (`art-preview-studio`)

```bash
"$ART_CLI" preview generate --workspace art-outreach-workspace
"$ART_CLI" preview serve --dir art-outreach-workspace/preview --port 4173
# Open http://127.0.0.1:4173/ to compare all 6 website archetypes live,
# toggle 60" View-on-Wall scale & ASCII wireframes, inspect 3D/2D video
# storyboards, preview email outreach copy, and review market research!
```

---

## Pillar 1: Anti-Slop Artist Website & Popular-Artist Portfolio

Synthesizes **Anthropic's `frontend-design` / `prompting_for_frontend_aesthetics`**, **ASCII wireframe-first constraints**, and **contemporary artist portfolio benchmarks** (Online Viewing Rooms, Catalogue Raisonné, Academic CV, Collector Inquiry Drawer).

1. **Preview & Lock ASCII Wireframe Before Coding**:
   Read [anti_slop_websites_and_wireframes.md](./references/anti_slop_websites_and_wireframes.md) and [popular_artist_portfolios.md](./references/popular_artist_portfolios.md):
   ```bash
   "$ART_CLI" studio wireframe white-cube-monograph
   # All 6 distinct variations:
   # white-cube-monograph | catalogue-raisonne | editorial-brutalist |
   # atmospheric-salon | tactile-studio-notebook | kinetic-exhibition
   ```
2. **Build Using Exclusively Registered Real Assets** (ban Inter/Roboto/Arial defaults, purple gradients, stock photos, and "Add to Cart" on museum originals). Include a dedicated **Private Viewing Room (`/viewing-room/...`)** to link from outreach emails.
3. **Mandatory Authenticity Audit + Live Local Preview**:
   ```bash
   "$ART_CLI" assets audit --target art-outreach-workspace/website
   "$ART_CLI" preview serve --dir art-outreach-workspace/website --port 4174
   ```

---

## Pillar 2: Social Media Advertising — 3D & 2D Artwork Videos

Orchestrates the **real 3D, 2D, motion, and publishing skills installed on this computer** — using **exclusively verified real artist scans/photos/clips**. Read [installed_skills_orchestration.md](./references/installed_skills_orchestration.md) and activate **`art-video-preview-bridge`** to pre-visualize before full renders.

- **3D Blender Gallery & Scale Reels** (`blender-3d-raking-light`, `blender-3d-architectural-scale`): Activate `blender-film` / `blender-forest` (RTX 3050 EEVEE Raytracing). Map the unaltered real scan onto a real-proportion 3D stretched linen canvas; preview via fast viewport `snap()` before full render.
- **2D Kinetic Exhibition & Launch Films** (`onetake-kinetic-exhibition`, `motionmaxxing-collector-film`): Activate `onetake` or `motionmaxxing` + `motion-doctrine` + `hyperframes` (strictly disabling any AI image-generation step!). Preview via `hyperframes preview` / `window.__seek(t)`.
- **Narrated Studio Story Shorts & Splits** (`remotion-collector-story`, `macro-split-process`): Activate `remotion-best-practices` / `remotion-captions` / `ffmpeg` / `pixel2motion` (artist's real signature only). Preview via `npx remotion studio` / `remotion still` / FFmpeg 3×3 storyboard PNG.
- **Audit & Publish**:
  1. Run `"$ART_CLI" assets audit --target art-outreach-workspace/videos/<project>`.
  2. Preview 3×3 storyboard PNG via `read_file` & publish vertical Shorts with `youtube-shorts-uploader` (`inspect` → `--dry-run` → `upload`).

---

## Pillar 3: High-Reach Connector Lead Discovery & Multi-Inbox Rust Outreach

Targets **High-Multiplier Connectors** (Interior Designers, Art Consultants, Hospitality Buyers, Luxury Stagers, Galleries, Architects, Concept Stores, Press) with 6 distinct non-fodder outreach angles (`"$ART_CLI" studio variations --pillar outreach`).

1. **Discover Leads (Multi-Source, CAPTCHA-Free)**:
   ```bash
   # Run generated queries via Antigravity's search_web tool, or use free OpenStreetMap / Search APIs / directory crawler:
   "$ART_CLI" discover osm-local --city "Austin, Texas" --limit 150
   "$ART_CLI" crm import-urls --file art-outreach-workspace/urls.txt --category interior_designer --location "Austin"
   ```
2. **Async Website Enrichment + Live MX Verification & 0–100 Scoring**:
   ```bash
   "$ART_CLI" enrich --limit 100 --concurrency 12 --max-subpages 4
   "$ART_CLI" verify --profile art-outreach-workspace/artist_profile.json
   "$ART_CLI" crm list --mx-valid --min-score 65 --limit 30
   ```
3. **Sender Deliverability Pre-Flight (SPF / DKIM / DMARC / MX)**:
   ```bash
   "$ART_CLI" deliverability hello@yourartstudio.com
   ```
4. **Draft Multi-Touch Outreach + Visual Review Gate**:
   ```bash
   "$ART_CLI" outreach generate-drafts \
     --profile art-outreach-workspace/artist_profile.json \
     --campaign "q1-connectors" --step 1 --min-score 65 --limit 25 \
     --review-out art-outreach-workspace/batches/review-touch1.md
   "$ART_CLI" preview generate --workspace art-outreach-workspace
   ```
5. **Dispatch Safely & Run Reply/Follow-Up Loop**:
   ```bash
   # Option A: Upload straight into Gmail/Outlook Drafts folder via pure-Rust TLS IMAP
   "$ART_CLI" outreach upload-imap-drafts --senders art-outreach-workspace/senders.json --campaign "q1-connectors"

   # Option B: Rotated multi-inbox SMTP or Resend/Brevo API (always --dry-run first!)
   "$ART_CLI" outreach send --senders art-outreach-workspace/senders.json --campaign "q1-connectors" --dry-run
   "$ART_CLI" outreach send --senders art-outreach-workspace/senders.json --campaign "q1-connectors"

   # Always sync replies/bounces/opt-outs BEFORE drafting Touch 2/3 follow-ups
   "$ART_CLI" outreach check-replies --senders art-outreach-workspace/senders.json --days 14
   "$ART_CLI" crm next-followups
   ```

---

## Reference Library (Read On Demand)

- [superpowers_workflow_and_authenticity.md](./references/superpowers_workflow_and_authenticity.md) — Superpowers 5-phase engineering discipline + Hard Zero-AI-Art / Zero-AI-Artist Protocol
- [anti_slop_websites_and_wireframes.md](./references/anti_slop_websites_and_wireframes.md) — Anthropic `frontend-design` anti-slop rules, ASCII wireframe planning, and 6 distinct artist site archetypes
- [popular_artist_portfolios.md](./references/popular_artist_portfolios.md) — Online Viewing Rooms, Catalogue Raisonné, Academic CV, and Collector Inquiry workflows
- [installed_skills_orchestration.md](./references/installed_skills_orchestration.md) — Orchestrating local Blender 3D, OneTake, MotionMaxxing, Remotion, HyperFrames, FFmpeg, and YouTube Shorts Uploader safely with real assets
- [icp_and_search_queries.md](./references/icp_and_search_queries.md) — High-multiplier buyer verticals, project-credits snowball method, and Google Boolean queries
- [email_playbooks.md](./references/email_playbooks.md) — High-reply art outreach copy, the free wall-mockup CTA, and reply-to-sale closing
- [deliverability_and_scaling.md](./references/deliverability_and_scaling.md) — SPF/DKIM/DMARC setup, 4-week warmup ramp, multi-inbox scaling math, and CAN-SPAM / GDPR compliance
