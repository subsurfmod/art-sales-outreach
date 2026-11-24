# ICP Strategy, Connector Multipliers & Search Playbook

Selling physical artwork 1-to-1 to cold strangers is slow and expensive. Selling to **Art Multipliers & Connectors** — professionals who source physical artwork repeatedly for wealthy clients, hotels, offices, and luxury listings — multiplies reach by **10× to 50× per relationship**.

---

## Tier 1: High-Multiplier Buyer Verticals (Start Here)

| Vertical (`category`) | Why They Have High Reach / Connections | Typical Order Value | Best Offer Hook |
| :--- | :--- | :--- | :--- |
| **`art_consultant`** | Curate art for law firms, hospitals, luxury condos, banks, and hotels; maintain active artist registries. | $3,000–$50,000+ (multi-piece) | Ready-to-ship inventory + reliable commission lead times + 1-page registry PDF |
| **`interior_designer`** | Every residential/commercial project needs focal-wall art; hate sourcing generic mass-market prints. | $1,500–$12,000 per room (repeat!) | 15–25% trade discount + **free digital "art-on-their-wall" room mockup** |
| **`hospitality`** | Boutique hotels, luxury restaurants, spas, and hospitality design firms buy cohesive collections. | $5,000–$40,000+ | Cohesive multi-piece series matched to property palette + crated shipping |
| **`stager`** | Luxury home stagers & model-home designers need large statement art; homebuyers frequently buy the art with the house! | $1,500–$8,000 | Flexible consignment/loan + 15–20% referral commission when homebuyer purchases |
| **`gallery`** | Built-in collector email lists, opening nights, art fairs, and local press. | Ongoing representation | Short, respectful portfolio alignment note (never bulk-blast galleries!) |
| **`architect`** | Specify feature walls & sculptural niches during design phase; introduce artists directly to homeowners. | $3,000–$20,000+ | Site-specific custom sizing & architectural scale capability |

## Tier 2: Local & Media Amplifiers

| Vertical (`category`) | Multiplier Effect | Best Offer Hook |
| :--- | :--- | :--- |
| **`concept_store`** | Design showrooms, high-end framing shops, and lifestyle boutiques have walk-in affluent buyers. | Small curated capsule / window feature / local maker spotlight |
| **`press`** | Regional home/design magazines, architecture blogs, podcasts, and Substacks expose your work to thousands of warm collectors at once. | Story angle (studio process, unusual material, upcoming collection/show) |

---

## High-Yield Google Search Queries (Use with `search_web` or Search APIs)

Replace `{CITY}`, `{STYLE}`, and `{MEDIUM}` with the artist's target market. Always exclude aggregator noise (`-site:houzz.com -site:yelp.com -site:pinterest.com`).

### Interior Designers
- `"{CITY}" "interior design" studio ("projects" OR "portfolio") ("contact" OR "inquiries") -site:houzz.com -site:yelp.com`
- `"{CITY}" luxury residential "interior designer" ("curated" OR "art" OR "bespoke") contact`
- `"{CITY}" "interior architecture" firm "{STYLE}" portfolio email`
- `"ASID" OR "IIDA" "{CITY}" "interior design" studio website`

### Art Consultants & Corporate Curators
- `"{CITY}" ("art consultant" OR "art advisory" OR "art curation") ("submissions" OR "artists" OR "contact")`
- `("hospitality art" OR "corporate art") advisory "{CITY}" portfolio`
- `"art procurement" OR "art sourcing" studio "{CITY}" contact`

### Luxury Home Stagers & Developers
- `"{CITY}" ("luxury home staging" OR "luxury staging") portfolio contact`
- `"{CITY}" "model home" interior merchandising studio contact`
- `"{CITY}" boutique real estate developer luxury residences design`

### Boutique Hotels & Hospitality Design
- `"{CITY}" "boutique hotel" ("local art" OR "design" OR "interiors") -site:booking.com -site:expedia.com -site:tripadvisor.com`
- `"{CITY}" "hospitality design" firm ("hotels" OR "restaurants") projects contact`

### Independent Galleries & Custom Framers
- `"{CITY}" "contemporary art gallery" ("submissions" OR "artists" OR "representation") "{MEDIUM}"`
- `"{CITY}" "custom framing" gallery "local artists" contact`

### Finding "Connectors" via Mutual Networks (Warmth Multiplier)
1. **Cross-reference Instagram & LinkedIn** extracted by `art-outreach enrich`:
   - Follow the design studio / art advisor 24–48 hours before emailing.
   - Leave 1 short, specific comment on a recent room installation (never pitch in comments!).
   - When your email arrives the next day with the same name/studio handle, recognition jumps dramatically.
2. **Project Credits Chain (Snowball Technique)**:
   - When an interior designer posts a completed luxury home or boutique hotel, check the project credits: they list the **architect**, **builder**, **photographer**, **stylist**, and **real estate agent**. All 5 serve the exact same affluent homeowner circle — add them to your CRM with `connection_notes` referencing the shared project!
3. **Referral Loop After Every Sale or Inquiry**:
   - Even when a designer says "not for this project", reply warmly: *"No problem at all — appreciate the quick reply! If you know a fellow designer or stager in {CITY} working on {STYLE} spaces, I'd love an intro."*
