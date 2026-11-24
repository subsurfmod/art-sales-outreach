---
name: art-preview-studio
description: >-
  Live visual Preview Studio for the Art Seller Assistant. Generates and serves
  an interactive localhost preview hub (`art-outreach preview`) so artists can
  visually inspect and compare all 6 anti-slop website/portfolio archetypes,
  ASCII wireframes, 60"-hang View-on-Wall scale mockups, 3D/2D video
  storyboards, and rendered outreach email sequences BEFORE coding, rendering,
  or sending anything. Also captures headless desktop/mobile visual screenshots
  and video contact sheets for instant review. Use whenever the user wants to
  preview a website, portfolio, email batch, or campaign before going live.
---

# Art Preview Studio (See Everything Before You Ship)

Never ask an artist to approve blind code, blind email blasts, or blind video renders. This skill provides **instant, zero-dependency local visual previews** of every deliverable.

---

## 1. Interactive Local Preview Studio (Pure-Rust HTTP Server)

Refresh and launch the interactive 4-tab Preview Studio (Website Archetypes + ASCII Wireframes + 60" Wall Scale + Video Storyboards + Email Previews + Market Research):

```bash
ART_CLI=/home/slave/.agents/skills/art-sales-outreach/bin/art-outreach

# Regenerate preview HTML from latest artist_profile.json, real_assets/manifest.json & CRM drafts
"$ART_CLI" preview generate --workspace art-outreach-workspace

# Serve locally on http://127.0.0.1:4173/ (pure Rust stdlib HTTP server, zero Node/Python needed)
"$ART_CLI" preview serve --dir art-outreach-workspace/preview --port 4173
```

You can also serve any built full website directory (`art-outreach-workspace/website`) with the exact same command:
```bash
"$ART_CLI" preview serve --dir art-outreach-workspace/website --port 4174
```

What the artist can inspect interactively in the browser:
- **Tab 1 — Website & Portfolio Switcher**: Click live across all 6 anti-slop archetypes (`White-Cube Monograph`, `Catalogue Raisonné`, `Editorial Brutalist`, `Atmospheric Salon`, `Tactile Studio Notebook`, `Kinetic Exhibition`) populated with their **real registered artwork scans** — plus toggle **Visual Prototype**, **60"-Hang View-on-Wall Scale**, and **ASCII Wireframe Spec**.
- **Tab 2 — 3D & 2D Social Video Storyboards**: Inspect all 6 video concepts, aspect ratios, required real assets, local skill pipelines, and play any rendered `.mp4` preview clips in `videos/`.
- **Tab 3 — Outreach Email Previews**: Inspect exact plain-text rendered emails, subject lines, word lengths, and live drafts queued in SQLite CRM.
- **Tab 4 — Market Research & Toolchain Readiness**: Verify real-asset SHA-256 registration count, local Blender/FFmpeg/YouTube Shorts OAuth readiness, and generated Boolean queries.

---

## 2. Automated Visual Screenshot Verification (Desktop + Mobile)

When previewing a built website for the user inside chat (so they see it immediately without even opening a browser), capture desktop (`1440×900`) and mobile (`390×844`) PNGs against the running local preview server using hardware-accelerated Chromium (respecting RTX 3050 GPU flags — never SwiftShader), then inspect both PNGs with `read_file` to check typography, spacing, contrast, and real artwork aspect fidelity:

1. Start `art-outreach preview serve --dir <target> --port 4173` as a background daemon (`IsDaemon: true`).
2. Capture screenshots with Playwright/Chromium or `chrome-headless-shell` (`--screenshot=... --window-size=1440,900` and `390,844`).
3. Run `art-outreach assets audit --target <target>` to guarantee `0` AI/stock violations.
4. Present the preview URL (`http://127.0.0.1:4173`) and inspected visual summary to the artist for sign-off.

---

## 3. Email & Dry-Run Outreach Previews

Before uploading IMAP drafts or sending live emails:
1. Generate the human-readable Markdown batch (`art-outreach outreach generate-drafts ... --review-out batches/review-touch1.md`).
2. Refresh the Preview Studio (`art-outreach preview generate`) so the live drafts appear in Tab 3.
3. Run `art-outreach outreach send ... --dry-run` to preview exact per-mailbox quota usage and recipient count with zero emails dispatched.
