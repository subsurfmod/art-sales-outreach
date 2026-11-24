# Local Installed Skills Orchestration (3D, 2D, Video & Social Publishing)

This skill orchestrates the **actual video, 3D, motion, and publishing skills already installed on this machine** (including skills sourced from `zhuyansen/awesome-claude-video-skills`) — filtered through our **STRICT ZERO-AI-ARTWORK / ZERO-AI-ARTIST AUTHENTICITY PROTOCOL**.

---

## 1. Strict Authenticity Routing Table

### APPROVED Skills (Real Artist Assets Only)

| Role | Installed Skill | Exact Path | How to Use Safely for Real Physical Artwork |
| :--- | :--- | :--- | :--- |
| **3D Gallery & Scale Staging** | `blender-film` | `/home/slave/.agents/skills/blender-film/SKILL.md` | Build 3D minimalist gallery walls, architectural living rooms, or sculpture plinths in Blender 5.2 (RTX 3050 EEVEE Raytracing via `run_blender_rtx3050.sh`). Apply the artist's **unaltered real high-res scan** as the canvas image texture (with subtle bump/roughness from the real photo — NEVER AI-regenerate the painting!). Sweep raking directional light & handheld shallow-DOF camera. |
| **3D Atmospheric Environments** | `blender-forest` | `/home/slave/.agents/skills/blender-forest/SKILL.md` | Stage outdoor sculptures or serene sunlit architectural spaces looking onto nature around the real artwork/3D scan. |
| **2D Single-Take Exhibition Reels** | `onetake` | `/home/slave/Projects/video-skills/.agents/skills/onetake/SKILL.md` | Pure-function-of-time HTML motion with real shutter motion blur: glide from a real macro brushwork photo into the full framed work + clean gallery placard typography + room acoustics. |
| **Anti-Slop Launch Films** | `motionmaxxing` | `/home/slave/Projects/video-skills/.agents/skills/motionmaxxing/SKILL.md` | High-craft GSAP commercial films for new collection drops & solo exhibitions using measured pro easing curves, zero HUD clutter, and **only real artist photos/clips** (explicitly disable any AI image generation step!). |
| **Continuous Camera Motion Law** | `motion-doctrine` | `/home/slave/Projects/video-skills/.agents/skills/motion-doctrine/SKILL.md` | Load before composing any multi-scene HTML/HyperFrames reel so cuts feel like one continuous gallery walkthrough rather than a slideshow. |
| **HTML-to-MP4 Motion Renderer** | `hyperframes` / `hyperframes-cli` | `/home/slave/Projects/video-skills/.agents/skills/hyperframes/SKILL.md` | Render code-driven 2D exhibition reels, kinetic typography title cards, and Viewing Room promo videos with real embedded artist photography. |
| **Programmatic React Video & Captions** | `remotion-best-practices`, `remotion-captions`, `remotion-render` | `/home/slave/Projects/video-skills/.agents/skills/remotion-best-practices/SKILL.md` | Build 9:16 vertical studio story Shorts combining real studio process clips, real macro crops, word-timed editorial captions, and collector inquiry CTAs. |
| **Real Signature / Monogram Animation** | `pixel2motion` | `/home/slave/Projects/video-skills/.agents/skills/pixel2motion/SKILL.md` | Vectorize the artist's **real handwritten signature or studio chop/monogram** into a smooth SVG draw-on end card (never alter or trace the artwork itself!). |
| **Media Cutting, Grading & Packaging** | `ffmpeg` | `/home/slave/Projects/video-skills/.agents/skills/ffmpeg/SKILL.md` | Trim real studio clips, create split-screen process/final reveals, normalize audio, add subtle film grain, and export crisp `9:16` (Reels/Shorts/TikTok) and `16:9` (YouTube/Website) MP4s (`h264_nvenc`). |
| **Automated YouTube Shorts Publishing** | `youtube-shorts-uploader` | `/home/slave/Projects/video-skills/.agents/skills/youtube-shorts-uploader/SKILL.md` | Inspect final vertical MP4, generate art-collector SEO metadata (`youtube-metadata.json`), dry-run, and upload directly to the artist's YouTube channel. |

### PROHIBITED For Artwork or Artist Likeness (Zero-AI-Art Protocol)

NEVER invoke these tools/skills to generate images or video of the artwork or the artist:
- `image-generator` subagent (NEVER fabricate paintings, sculptures, studio photos, or artist portraits!)
- `heygen-avatar` / `heygen-video` (NEVER create an AI avatar clone of the artist!)
- `vox-director` / `paper-cut` / `huashu-art-motion` AI frame-generation steps (NEVER feed artwork into AI image generators or AI outpainters!)

---

## 2. Six Distinct Social Media Video Production Recipes

Run `art-outreach studio variations --pillar video` to pick a format tailored to the artist:

1. **`blender-3d-raking-light` (3D — 12–18s)**: Real flat scan mapped onto 3D stretched linen canvas (`W × H × 4cm` depth + optional oak float frame). Animated warm sunlight rakes slowly across the gallery wall while a 50mm shallow-DOF handheld camera pans from edge depth to full composition.
2. **`blender-3d-architectural-scale` (3D — 15–20s)**: Exact-scale real artwork mounted at 60" eye level above a 3D travertine console or linen sofa inside a sunlit architectural interior. Slow dolly pull-back reveals true physical scale for interior designers & collectors.
3. **`onetake-kinetic-exhibition` (2D — 15–25s)**: Continuous one-take HTML/CSS/JS composition starting inside a 300% zoom on a real macro texture crop, pulling back with real shutter blur to reveal the full work, carrying smoothly into Title / Medium / Dimensions placard + Viewing Room URL.
4. **`motionmaxxing-collector-film` (2D+3D — 25–45s)**: Editorial collection launch film cutting real studio process video with 3D gallery shots and stillness-before-climax hero reveals — audited against MotionMaxxing's anti-slop "Never Ship" list.
5. **`remotion-collector-story` (2D — 25–40s)**: 9:16 vertical narrated Short (using the artist's real studio voice memo or clean quiet text captions) walking through the story and materials of one real piece, then published via `youtube-shorts-uploader`.
6. **`macro-split-process` (2D — 10–18s)**: `ffmpeg` vertical split-screen or wipe comparing real raw work-in-progress footage with the finished framed real piece, closing with a `pixel2motion` animated draw-on of the artist's real signature.

---

## 3. Mandatory Pre-Publish Video Audit

Before exporting or uploading ANY 3D or 2D social video:
1. Run `art-outreach assets audit --target <video-project-dir>` — verify every single artwork texture, photo, and studio clip traces back to `real_assets/manifest.json`.
2. Verify the artwork colors, aspect ratio, and cropping have NOT been distorted, stretched, or over-filtered (collectors must receive a physical object that matches what they saw in the video!).
3. Export both **9:16 (`1080×1920`)** for Instagram Reels / YouTube Shorts / TikTok and **16:9 (`1920×1080`)** for Website / YouTube / LinkedIn where appropriate.
