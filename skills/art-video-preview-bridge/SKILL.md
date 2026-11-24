---
name: art-video-preview-bridge
description: >-
  3D & 2D Artwork Video Pre-Visualization Bridge for the Art Seller Assistant.
  Connects all preview-capable local video skills (Blender 3D viewport/clay
  snaps, HyperFrames live preview, Remotion Studio & stills, OneTake seekable
  HTML contact sheets, FFmpeg 3x3 visual storyboard grids, and YouTube Shorts
  Uploader dry-run inspection) strictly under the Zero-AI-Artwork /
  Zero-AI-Artist Authenticity Gate. Use whenever previewing, storyboarding, or
  QA-checking a 3D or 2D artwork promo video before full GPU rendering or
  social media publishing.
---

# Art Video Pre-Visualization Bridge (3D & 2D Preview Pack)

Full 3D and 4K60 renders take time. Always **preview first** with fast low-cost snapshots, interactive browser scrubbers, and 3×3 visual contact sheets so the artist approves lighting, framing, color fidelity, and typography before rendering or publishing.

> **HARD AUTHENTICITY GATE**: Every artwork texture, photo, and studio video clip MUST be registered in `real_assets/manifest.json` and pass `art-outreach assets audit --target <video-dir>`. Never use AI image generators or AI artist avatars.

---

## Preview Tools Packed & Routed From This Machine

| Pipeline | Local Skill Path | Instant Preview Command / Method | What You Verify Before Full Render |
| :--- | :--- | :--- | :--- |
| **3D Blender Gallery & Room Scale** | `/home/slave/.agents/skills/blender-film/SKILL.md` | Fast **white-clay / low-sample viewport snapshot** via live bridge (`snap()` at 16–32 samples on RTX 3050 EEVEE via `/home/slave/.agents/skills/remmino/scripts/run_blender_rtx3050.sh`) | Exact physical canvas aspect ratio (`W:H`), 60" hang height, raking light angle, zero color washout on the real painting |
| **2D Single-Take Kinetic Reels** | `/home/slave/Projects/video-skills/.agents/skills/onetake/SKILL.md` | Open single HTML composition via `art-outreach preview serve` + scrub `window.__seek(t)` + run OneTake contact-sheet oracle | Smooth camera carry from macro crop to full frame, zero slideshow cadence, legible museum placards |
| **HTML / GSAP Commercial Reels** | `/home/slave/Projects/video-skills/.agents/skills/hyperframes-cli/SKILL.md` & `motionmaxxing` | `hyperframes preview` (interactive browser timeline) + `motionmaxxing` visual gate (`look.py` / `lint.mjs`) | Zero HUD clutter, pro easing curves, truthful artwork framing |
| **React Programmatic Vertical Shorts** | `/home/slave/Projects/video-skills/.agents/skills/remotion-best-practices/SKILL.md` | `npx remotion studio` (live browser scrubber) or instant single-frame PNG via `npx remotion still <CompId> out.png --frame=45` | Safe caption margins (`9:16` Shorts/Reels UI clear zones), word-timing alignment, crisp real footage |
| **Any Rendered MP4 Contact Sheet** | `/home/slave/Projects/video-skills/.agents/skills/ffmpeg/SKILL.md` | Generate a 3×3 visual storyboard grid PNG in 1 second (see command below) and inspect directly with `read_file`! | End-to-end visual pacing, aspect ratio, color fidelity, zero black frames |
| **YouTube Shorts Publish Preview** | `/home/slave/Projects/video-skills/.agents/skills/youtube-shorts-uploader/SKILL.md` | `yt_shorts.py inspect <mp4>` + `yt_shorts.py upload <mp4> --dry-run` | Title, SEO description, tags, Shorts eligibility (`9:16`, `<=180s`), duplicate check before live upload |

---

## 1-Second FFmpeg 3×3 Video Storyboard Contact Sheet

Run this on ANY draft 3D or 2D video before sharing with the artist or uploading to social media, then visually inspect `preview-grid.png` using `read_file`:

```bash
ffmpeg -y -i art-outreach-workspace/videos/draft.mp4 \
  -vf "fps=9/$(ffprobe -v error -show_entries format=duration -of default=nw=1:nk=1 art-outreach-workspace/videos/draft.mp4),scale=360:-1,tile=3x3" \
  -frames:v 1 art-outreach-workspace/videos/preview-grid.png
```

## Fast Low-Res Draft Export (Before Final 1080p/4K)

When iterating on lighting, pacing, or captions, export a 540p draft first (`h264_nvenc`), refresh `art-outreach preview generate`, and play it in Tab 2 of the **Art Preview Studio** (`http://127.0.0.1:4173`) so the artist approves before the final render!
