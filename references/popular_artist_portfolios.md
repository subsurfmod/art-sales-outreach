# Contemporary Artist Portfolio Architecture (Popular Artist Benchmarks)

Modeled on the real structural standards used by leading contemporary artists, blue-chip galleries (David Zwirner, Pace, Hauser & Wirth), and institutional archives (Gerhard Richter, Olafur Eliasson, Agnes Martin).

---

## 1. Core Site Map (Keep Top Nav to 4–6 Items Max)

1. **Selected Works** — Grouped strictly by **Series, Year, or Medium** (never one unstructured dump of 80 images). Clicking any work opens a dedicated **Plate View** with high-res zoom + real macro detail crops + real installation photo.
2. **Viewing Room (OVR)** — Dedicated curated rooms (`viewing-room/<series-slug>`) showcasing 5–8 cohesive available works with a short curatorial note, exact physical dimensions, **View-on-Wall scale reference** (standard 60" eye-level museum hang), and a 1-click downloadable PDF checklist.
3. **Catalogue / Archive** — Standardized scholarly record per work:
   - *Title* (italicized), Year
   - Medium & substrate (e.g., *Oil, cold wax, and crushed limestone on Belgian linen*)
   - Exact dimensions: `H × W × D` in **both inches and centimeters**
   - Status (`Available`, `On Hold`, `Private Collection`, `Institutional Collection`)
   - Provenance & Exhibition History
4. **Exhibitions & Academic CV** — Strictly chronological (newest first), single-column editorial layout with a **Download PDF CV** button:
   - Lives & Works (City)
   - Education (if applicable)
   - Solo Exhibitions
   - Selected Group Exhibitions
   - Public, Corporate & Notable Collections
   - Residencies, Grants & Awards
   - Selected Press & Bibliography
5. **Studio / Practice** — Concise, plain-spoken artist statement (150–250 words, zero pretentious "artspeak" jargon) + **real** studio process photos/clips + optional studio visit scheduling.
6. **Inquire / Trade** — Dedicated collector & interior-designer acquisition page (trade discount terms, custom commission process, crated worldwide shipping, Certificate of Authenticity).

---

## 2. Collector Inquiry Flow (Why "Add to Cart" Hurts Original Fine Art Sales)

- **Originals ($1,500+)**: Use a contextual slide-out **"Inquire About *[Artwork Title]*"** drawer right beside the artwork placard (pre-filling the artwork ID & title!) instead of a cold e-commerce shopping cart. Ask only for: *Name, Email, Role (Private Collector / Interior Designer / Art Advisor / Press), and optional Wall Photo upload for a free scale mockup*.
- **Limited Prints / Small Works (< $500)**: Can optionally sit in a clean separate **Editions** section with transparent pricing and direct checkout, clearly separated from unique studio originals so institutional pricing integrity stays intact.
- **Gallery Representation Hygiene**: If the artist is represented by a gallery in a specific territory, include a clear, respectful line on the Contact page routing primary inquiries for that territory to the gallery.

---

## 3. Portfolio Build Checklist (Zero-Slop Verification)

- [ ] Every artwork image, detail crop, studio clip, and artist portrait is registered in `real_assets/manifest.json` (`art-outreach assets register`).
- [ ] Zero AI-generated images or external stock photos (`art-outreach assets audit --target ./website` exits `0`).
- [ ] Every work displays exact physical dimensions in both inches and cm + medium + year.
- [ ] At least one **Private Viewing Room** page exists to link directly from high-score outreach emails!
- [ ] Real macro detail crops are paired with full-frame shots so collectors can inspect surface texture.
- [ ] Academic CV & downloadable Trade Catalog PDF link are accessible within 1 click.
