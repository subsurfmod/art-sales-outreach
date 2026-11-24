# art-sales-outreach

A full-fledged, **Rust-powered Art Seller Assistant** & **AI Agent Skill (`SKILL.md`)** for selling **physical artwork** end-to-end — combining strict real-artwork authenticity verification, anti-slop artist websites & museum-grade portfolios, 3D & 2D social media artwork videos, and high-deliverability multi-inbox customer outreach.

> **STRICT AUTHENTICITY PROTOCOL**: Zero AI-generated images of the artwork or the artist. Built-in Rust SHA-256 asset registry & build auditor (`art-outreach assets register` / `audit`) screens filenames, EXIF/PNG/XMP/C2PA metadata, and external stock/AI hosts so every website, viewing room, and 3D/2D video showcases 100% authentic artist-supplied photography and studio footage.

---

## Four Integrated Pillars

1. **Strict Real-Asset Authenticity Gate (`art-outreach assets`)**
   - SHA-256 manifest ledger (`real_assets/manifest.json`) for every real painting scan, macro crop, installation shot, studio clip, and artist portrait.
   - Automatic AI-marker & stock-host screening (`Midjourney`, `DALL-E`, `Stable Diffusion`, `ComfyUI`, `Flux`, `C2PA`, `Unsplash`, `Pexels`, etc.).
   - Hard build gate (`art-outreach assets audit --target <dir>`) that fails CI/publishing if any website or video project references an unregistered or AI-generated image.
2. **Anti-Slop Artist Websites & Popular-Artist Portfolios (`art-outreach studio`)**
   - Built on **Anthropic's `frontend-design` / `prompting_for_frontend_aesthetics`**, **ASCII wireframe-first layout constraints**, **`obra/superpowers`** spec-and-verify discipline, and museum-grade contemporary artist portfolio standards (Online Viewing Rooms, Catalogue Raisonné, Academic CV, 60"-hang View-on-Wall scale preview, contextual Collector Inquiry drawers).
   - **6 radically distinct website archetypes** (`white-cube-monograph`, `catalogue-raisonne`, `editorial-brutalist`, `atmospheric-salon`, `tactile-studio-notebook`, `kinetic-exhibition`) — zero generic Inter/purple-gradient SaaS fodder.
3. **3D & 2D Social Media Artwork Video Pipeline**
   - **6 distinct video formats** orchestrating local 3D & 2D video skills (`blender-film`, `blender-forest`, `onetake`, `motionmaxxing`, `motion-doctrine`, `hyperframes`, `remotion-*`, `pixel2motion`, `ffmpeg`, `youtube-shorts-uploader` — inspired by `zhuyansen/awesome-claude-video-skills`).
   - Stages the artist's unaltered real scan on 3D stretched linen canvases in architectural rooms with raking light, or builds single-take 2D kinetic exhibition reels and narrated studio Shorts.
4. **Connector-Multiplier Lead Engine & Multi-Inbox Rust Email Outreach (`art-outreach crm|discover|enrich|verify|deliverability|outreach`)**
   - Embedded SQLite CRM (`rusqlite` bundled) + permanent suppression list.
   - Multi-source discovery: Google Boolean query generator, Serper/Brave/Google CSE APIs, free OpenStreetMap Overpass local discovery, and robots-respecting directory crawling.
   - Concurrent `tokio` + `reqwest` website enricher (emails, JSON-LD decision-makers, Instagram/LinkedIn/Pinterest, trade/submission signals, aesthetic hooks).
   - Concurrent `hickory-resolver` live MX verification + 0–100 Connector-Multiplier scoring + SPF/DKIM/DMARC sender domain auditor.
   - Multi-touch personalized sequences across rotated **SMTP** (`lettre` rustls), pure-Rust **TLS IMAP Drafts** upload & reply/bounce/opt-out scanning, or **Resend / Brevo** HTTP APIs.

---

## Quick Start

### 1. Build the Single Native Rust Binary

```bash
git clone https://github.com/subsurfmod/art-sales-outreach.git
cd art-sales-outreach
cargo build --release
# Binary: ./target/release/art-outreach
```

### 2. Install as an Agent Skill (Antigravity / Claude Code / Codex)

```bash
mkdir -p ~/.agents/skills/art-sales-outreach/bin
cp -r SKILL.md Cargo.toml src templates references ~/.agents/skills/art-sales-outreach/
cp target/release/art-outreach ~/.agents/skills/art-sales-outreach/bin/art-outreach
```

### 3. Example Workflow

```bash
# 1. Register artist's REAL artwork photos/videos (screens out AI-generated suspects)
art-outreach assets register --dir art-outreach-workspace/real_assets

# 2. Generate a bespoke 4-pillar Master Blueprint & ASCII Website Wireframe
art-outreach studio blueprint --medium "oil on linen" --mood "warm minimalist" --price-tier "mid-career" --city "Austin"
art-outreach studio wireframe white-cube-monograph

# 3. Audit your built website or 3D/2D video project before publishing
art-outreach assets audit --target art-outreach-workspace/website

# 4. Run high-reach connector lead discovery, enrichment, MX verification & outreach
art-outreach crm init
art-outreach discover osm-local --city "Austin, Texas" --limit 150
art-outreach enrich --limit 100 --concurrency 12
art-outreach verify --profile artist_profile.json
art-outreach deliverability hello@yourartstudio.com
art-outreach outreach generate-drafts --profile artist_profile.json --campaign "q1-connectors" --review-out batches/review-touch1.md
art-outreach outreach send --senders senders.json --campaign "q1-connectors" --dry-run
```

---

## CLI Command Tree

```text
art-outreach [--db <path>]
├── assets          register | audit | catalog
├── studio          variations | wireframe | blueprint
├── crm             init | import-urls | import-csv | stats | list | export | suppress | mark-status | next-followups
├── discover        generate-queries | search-api | osm-local | crawl-directory
├── enrich          --limit --concurrency --max-subpages --category --force
├── verify          --profile --skip-dns --concurrency
├── deliverability  <domain-or-email> [--dkim-selector ...]
└── outreach        generate-drafts | upload-imap-drafts | send | check-replies
```

## Inspiration & Methodology Credits

- **Agentic Engineering Discipline**: [`obra/superpowers`](https://github.com/obra/superpowers)
- **Anti-Slop Frontend Aesthetics**: Anthropic `frontend-design` & [`claude-cookbooks/coding/prompting_for_frontend_aesthetics.ipynb`](https://github.com/anthropics/claude-cookbooks/blob/main/coding/prompting_for_frontend_aesthetics.ipynb)
- **ASCII Wireframe-First Planning**: AI ASCII wireframe constraint workflows (`bareminimum.design` / `waiframe`)
- **3D & 2D Video Skill Ecosystem**: [`zhuyansen/awesome-claude-video-skills`](https://github.com/zhuyansen/awesome-claude-video-skills)

## License

MIT
