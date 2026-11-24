# art-sales-outreach

A full-fledged, **Rust-powered Art Seller Assistant** & **AI Agent Skill Suite** for selling **physical artwork** end-to-end — featuring an effortless **First-Time Setup Wizard**, a built-in **Interactive Local Preview Studio**, strict **Zero-AI-Artwork / Zero-AI-Artist** authenticity auditing, anti-slop artist websites & museum-grade portfolios, 3D & 2D social media artwork video orchestration, and high-deliverability multi-inbox customer outreach.

> **STRICT AUTHENTICITY PROTOCOL**: Zero AI-generated images of the artwork or the artist. Built-in Rust SHA-256 asset registry & build auditor (`art-outreach assets register` / `audit`) screens filenames, EXIF/PNG/XMP/C2PA metadata, and external stock/AI hosts so every website, viewing room, and 3D/2D video showcases 100% authentic artist-supplied photography and studio footage.

---

## Packed Skill Suite (`SKILL.md` + `skills/`)

This repository ships as a complete skill bundle — install all 4 skills together:

1. **`art-sales-outreach`** ([`SKILL.md`](./SKILL.md)) — Master Art Seller Assistant orchestrating all pillars & the Rust CLI (`art-outreach`).
2. **`art-first-time-setup`** ([`skills/art-first-time-setup/SKILL.md`](./skills/art-first-time-setup/SKILL.md)) — Effortless guided onboarding wizard covering portfolio/real artwork intake, social media & email integrations, website/video/outreach options, and automated initial market research.
3. **`art-preview-studio`** ([`skills/art-preview-studio/SKILL.md`](./skills/art-preview-studio/SKILL.md)) — Built-in pure-Rust localhost Preview Studio (`http://127.0.0.1:4173`) to compare all 6 anti-slop website archetypes live, toggle 60" View-on-Wall scale & ASCII wireframes, inspect 3D/2D video storyboards, and review email pitches before shipping.
4. **`art-video-preview-bridge`** ([`skills/art-video-preview-bridge/SKILL.md`](./skills/art-video-preview-bridge/SKILL.md)) — Pre-visualization bridge for Blender 3D clay/viewport snaps, HyperFrames preview, Remotion Studio/stills, OneTake seekable contact sheets, FFmpeg 3×3 storyboard grids, and YouTube Shorts `--dry-run`.

---

## Quick Start

### 1. Build the Single Native Rust Binary

```bash
git clone https://github.com/subsurfmod/art-sales-outreach.git
cd art-sales-outreach
cargo build --release
# Binary: ./target/release/art-outreach
```

### 2. Install All Packed Skills (Antigravity / Claude Code / Codex)

```bash
mkdir -p ~/.agents/skills/art-sales-outreach/bin
cp -r SKILL.md README.md Cargo.toml Cargo.lock src templates references skills ~/.agents/skills/art-sales-outreach/
cp target/release/art-outreach ~/.agents/skills/art-sales-outreach/bin/art-outreach
cp -r skills/art-first-time-setup skills/art-preview-studio skills/art-video-preview-bridge ~/.agents/skills/
```

### 3. Run Effortless First-Time Setup & Open Live Preview Studio

```bash
# Interactive setup wizard (portfolio/artwork intake, socials, email backend, initial market research):
./target/release/art-outreach setup

# Open the Interactive Local Preview Studio to see all 6 website variations,
# 60" wall scale, 3D/2D video storyboards, email previews & market research:
./target/release/art-outreach preview serve --dir art-outreach-workspace/preview --port 4173
```

---

## CLI Command Tree

```text
art-outreach [--db <path>]
├── setup           [--workspace] [--from-json] [--defaults] [--assets-dir] [--run-osm]
├── preview         generate | serve [--dir] [--port 4173]
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

GNU General Public License v3.0 — see [LICENSE](./LICENSE).
