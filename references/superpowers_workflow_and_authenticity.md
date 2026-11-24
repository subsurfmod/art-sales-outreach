# Superpowers Engineering Discipline & Strict Authenticity Protocol

Inspired by **`obra/superpowers`** (Jesse Vincent's structured agentic workflow: Brainstorming → Spec → Planning → Isolated Execution → Verification) combined with a **Hard Zero-AI-Art Authenticity Gate**.

---

## 1. The Non-Negotiable Zero-AI-Artwork / Zero-AI-Artist Protocol

Collectors buy physical artwork because of **human authorship, provenance, and material truth**. Using an AI-generated painting, AI-outpainted canvas, or AI-generated headshot destroys an artist's credibility permanently.

### Hard Rules (Zero Exceptions)
1. **Never generate AI images of artwork or the artist.** Do not invoke `image-generator`, Midjourney, DALL-E, Flux, Stable Diffusion, ComfyUI, Adobe Firefly, or HeyGen avatars for any artwork visual, studio photo, or artist portrait.
2. **Never use stock photos as artwork stand-ins.** Do not link Unsplash, Pexels, Pixabay, or Picsum images in place of real paintings/sculptures.
3. **Register First, Build Second.** Before building any website, portfolio, PDF lookbook, or 3D/2D video, require the artist to place their real photographs/scans/studio clips into `art-outreach-workspace/real_assets/` and run:
   ```bash
   art-outreach assets register --dir art-outreach-workspace/real_assets
   ```
   The Rust CLI computes SHA-256 checksums and automatically inspects filenames + EXIF/PNG/XMP/C2PA bytes to reject suspected AI-generated media.
4. **Enforce Build Gate Before Delivery.** Run:
   ```bash
   art-outreach assets audit --target <website-or-video-dir>
   ```
   If exit code is non-zero (`2`), **STOP** and fix every violation before presenting or publishing the deliverable.
5. **Preserve Color & Aspect Fidelity.** When staging real artwork scans on a 3D Blender canvas or inside a 2D HTML/Remotion video, preserve exact aspect ratio (`W:H`) and truthful color grading so the physical piece matches the preview.

---

## 2. The 5-Phase Superpowers Execution Loop

Never "vibe-code" a generic artist website, generic social video, or spammy outreach blast. Always execute in order:

### Phase 1: Brainstorming & Differentiation Interview (Before Building Anything)
Ask focused questions (1 batch, concise) to capture what makes *this* artist unmistakable:
- Medium, physical scale range, surface texture, and price tier
- Real available assets (high-res scans, macro crops, studio clips, real artist portrait, CV/exhibitions)
- Target geography & ideal buyers (interior designers, art consultants, galleries, hospitality, luxury stagers, direct collectors)
- Generate a bespoke 4-pillar recommendation via:
  ```bash
  art-outreach studio blueprint --medium "..." --mood "..." --price-tier "..." --city "..."
  ```
- Present **2–3 distinct variation options** with clear trade-offs and let the artist pick before building.

### Phase 2: Spec & ASCII Wireframe Sign-Off
- For websites/portfolios: output the box-drawn **ASCII Wireframe** (`art-outreach studio wireframe <slug>`) + typography/palette tokens and confirm layout hierarchy before coding.
- For 3D/2D videos: write a shot-by-shot storyboard listing exact `real_assets/` filenames and durations before opening Blender/HTML/Remotion.
- For email outreach: audit sending domain DNS (`art-outreach deliverability`) and lock target verticals + offer angle before scraping/drafting.

### Phase 3: Isolated, Bite-Sized Execution
- Keep each pillar cleanly isolated under `art-outreach-workspace/` (`real_assets/`, `website/`, `videos/`, `data/`, `batches/`).
- Use parallel subagents only for genuinely independent tasks (e.g., running `search_web` queries across 3 cities simultaneously while auditing website HTML).

### Phase 4: Automated Verification Gates (Zero Self-Certification)
Never declare a pillar "done" without running its concrete gate:
- **Assets Gate**: `art-outreach assets audit --target <dir>` → `passed: true`, `violation_count: 0`
- **Website Gate**: Visual inspection of desktop + mobile renders + zero broken links + zero banned AI-slop fonts/patterns
- **Video Gate**: Inspect rendered keyframes/contact sheet via `read_file` + verify real artwork aspect ratio & audio loudness
- **Outreach Gate**: `art-outreach deliverability` score `>= 65` + MX verified + human review of `review-touch1.md` + `--dry-run` pass

### Phase 5: Continuous Feedback & Compounding Loop
- Track real email replies (`art-outreach outreach check-replies`), website viewing-room inquiries, and social video engagement; refine variations based on real collector responses rather than guesswork.
