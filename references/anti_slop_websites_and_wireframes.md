# Anti-Slop Artist Website Design & ASCII Wireframe Methodology

Distilled from **Anthropic's `frontend-design` plugin**, **`anthropics/claude-cookbooks/coding/prompting_for_frontend_aesthetics.ipynb`**, and **ASCII Wireframe-First AI layout constraints** (`bareminimum.design` / `waiframe`).

---

## 1. The Anti-Slop Ban List (Never Ship Common AI Fodder)

AI coding tools suffer from *distributional convergence* — defaulting to the same generic SaaS look for every site. For a physical fine-art website, **explicitly ban all of the following**:

- **Banned Default Fonts**: `Inter`, `Roboto`, `Arial`, `Open Sans`, `Helvetica Neue` system defaults.
- **Banned SaaS Clichés**: Purple-to-blue gradients, uniform rounded `border-radius: 16px` white cards with soft drop-shadows, "Meet [Artist]" hero kickers, 3-column icon feature grids, fake collector testimonials, or generic "Add to Cart" buttons on unique fine art originals.
- **Banned Fake Imagery**: Zero Unsplash/Pexels stock photos, zero placeholder abstract blobs pretending to be paintings, zero AI-generated artwork or artist portraits (enforced technically by `art-outreach assets audit`).

---

## 2. ASCII Wireframe First (Mandatory Structural Lock)

Before writing a single line of HTML, CSS, or JS:
1. Run `art-outreach studio wireframe <variation>` to generate a strict box-drawn **ASCII layout wireframe**.
2. Customize the ASCII wireframe to match the specific artist's series count, aspect ratios (portrait vs. panoramic canvases vs. 3D sculptures), and navigation (max 4–6 items).
3. Get user sign-off on the ASCII wireframe before implementing code.

Why? Plain-text ASCII wireframes act as rigid spatial constraints — locking negative space, asymmetry, placard alignment, and hierarchy without distracting color noise or layout hallucination.

---

## 3. Six Distinct Website Archetypes (Never Repeat the Same Template)

Choose the archetype that grows organically from the artist's actual medium, palette, and market tier — never force a minimalist quiet painter into a loud brutalist grid, or a bold sculptural artist into a timid template:

| Slug | Aesthetic Direction | Display + Technical Font Pairing | Ground & Ink Tokens | Structural Signature |
| :--- | :--- | :--- | :--- | :--- |
| `white-cube-monograph` | Museum stillness, massive breathing room | `Cormorant Garamond` + `IBM Plex Mono` | Plaster `#F7F5F0` / Graphite `#181816` / Ochre `#9A6B3F` | Single full-scale hero work, 60"-hang View-on-Wall simulator, discreet Collector Inquiry drawer |
| `catalogue-raisonne` | Scholarly archive & provenance authority | `Newsreader` + `JetBrains Mono` | Archival `#F3F1EC` / Carbon `#111111` / Cobalt `#1E4E8C` | Filterable oeuvre ledger (Year/Medium/Status) + sticky deep-zoom plate & provenance viewer |
| `editorial-brutalist` | Exhibition poster & stark architectural grid | `Syne` + `Space Mono` | Newsprint `#F2EFE9` / Pitch `#0D0D0C` / Vermilion `#D9381E` | Hairline 1px grid rules, oversized masthead, raw exhibition ledger, direct studio video |
| `atmospheric-salon` | Dark-wall nocturnal private viewing room | `Instrument Serif` + `DM Mono` | Velvet `#111214` / Bone `#ECE7DD` / Brass `#C8A25A` | Illuminated real artwork on dark museum walls, slow chaptered literary essays |
| `tactile-studio-notebook` | Craft, pigment recipes, process-led storytelling | `Fraunces` + `JetBrains Mono` | Raw Linen `#EFEAE0` / Walnut `#261E18` / Terracotta `#B85233` | Side-by-side finished real work + macro detail crop + real studio notes & studio visit booking |
| `kinetic-exhibition` | Contemporary gallery launch & VIP drop portal | `Bricolage Grotesque` + `Crimson Pro` | Chalk `#FAF9F6` / Obsidian `#141417` / Ultramarine `#1C3FD9` | Horizontal snap-scroll gallery wall, live availability status, password-gated Advisor Trade Room |

---

## 4. High-Impact Motion & Atmospheric Craft Rules

- **Orchestrated Entrance Over Micro-Clutter**: Spend motion on 1–2 intentional moments (e.g., slow museum curtain fade-in of the real hero painting, smooth View-on-Wall scale transition, or subtle horizontal gallery rail glide) instead of jittery hover bounces on every button.
- **Let the Physical Artwork Hold the Color**: Derive the website's single accent color directly from an actual pigment present in the artist's real photographs — keep surrounding surfaces neutral archival plaster, raw linen, or museum charcoal so the physical artwork always dominates.
- **Responsive Mobile Viewing**: Collectors and interior designers often open links on iPhone/iPad from Instagram or email; ensure full-bleed tap-to-zoom on real artwork photos and a 1-tap "Inquire About This Work" drawer on mobile.
