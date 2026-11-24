# Deliverability, Scaling Reach Safely & Compliance

**True Reach = Emails Sent × Inbox Placement Rate × Reply Rate.**
Blasting 500 emails from an unconfigured Gmail lands 90% in Spam and gets your account suspended. Properly authenticated multi-inbox outreach scales cleanly to **100–300+ verified leads/day** with 95%+ Primary Inbox placement.

---

## 1. Domain Strategy (Protect Your Main Website Domain)

- **Under 30 emails/day**: Your main studio custom domain (`hello@yourartstudio.com` on Google Workspace, Zoho Mail, Fastmail, or Microsoft 365) is fine once SPF/DKIM/DMARC pass.
- **Scaling to 60–200+ emails/day**: Buy 1–2 lookalike secondary domains (e.g., `yourartstudio.co`, `yournameart.com`, ~$10/year) and redirect their root URL (`301`) to your main portfolio site. Create 2 mailboxes per domain and add them to `senders.json` — `art-outreach outreach send` automatically **round-robins** across all enabled mailboxes while enforcing each mailbox's `daily_limit`!

### Scaling Math (Safe Multi-Inbox Capacity)
| Setup | Mailboxes | Safe Warmed Daily Limit / Inbox | Total Daily Reach | Monthly Reach (22 business days) |
| :--- | :---: | :---: | :---: | :---: |
| Starter (1 studio inbox) | 1 | 25–35 / day | 25–35 / day | ~600–750 leads/mo |
| Growth (1 secondary domain) | 2 | 40 / day | 80 / day | ~1,750 leads/mo |
| Studio Scale (2 secondary domains) | 4 | 45 / day | 180 / day | ~3,960 leads/mo |

---

## 2. Mandatory DNS Records (Audit with `art-outreach deliverability`)

Run before launching any campaign:
```bash
art-outreach deliverability hello@yourartstudio.com
```

1. **SPF** (TXT record on `@` — MUST have only ONE `v=spf1` record):
   - Google Workspace: `v=spf1 include:_spf.google.com ~all`
   - Microsoft 365: `v=spf1 include:spf.protection.outlook.com ~all`
   - Zoho Mail: `v=spf1 include:zoho.com ~all`
   - If also sending via Resend/Brevo, combine includes in the single SPF record.
2. **DKIM** (TXT/CNAME record generated inside your email provider's Admin Console → Domain Authentication).
3. **DMARC** (TXT record on `_dmarc`):
   - `v=DMARC1; p=none; rua=mailto:dmarc@yourartstudio.com; adkim=r; aspf=r`
   - After 2–4 weeks with clean reports, tighten to `p=quarantine`.

---

## 3. 4-Week Mailbox Warmup Schedule

Never jump from 0 to 50 emails/day on a brand-new mailbox:
- **Week 1**: `daily_limit: 10–15` (prioritize highest-score leads so you get real replies early — replies build massive sender reputation!)
- **Week 2**: `daily_limit: 20–25`
- **Week 3**: `daily_limit: 30–40`
- **Week 4+**: `daily_limit: 40–50` per mailbox (cap at 50/mailbox; scale horizontally with extra mailboxes rather than hammering one inbox).

---

## 4. Technical Anti-Spam Guardrails (Built Into `art-outreach`)

- **Plain-Text First**: Zero tracking pixels, zero heavy HTML tables, zero cold attachments.
- **Live MX Verification**: Never send to domains without valid MX (`art-outreach verify` blocks them by default). Keep bounce rate **< 2%**.
- **Human-Like Pacing**: `art-outreach outreach send` waits a randomized `--min-delay 45` to `--max-delay 120` seconds between emails instead of firing bursts.
- **Reply & Bounce Auto-Halt**: Always run `art-outreach outreach check-replies` before generating Touch 2/3 so anyone who replied, bounced, or opted out is immediately stopped and suppressed.

---

## 5. Email Sending Providers Supported Out of the Box

1. **Gmail / Google Workspace SMTP + IMAP** (easiest): Enable 2-Step Verification → create an **App Password** → set `export ART_SMTP_PASSWORD_1="..."`. Supports both direct sending (`send`) and native Drafts inspection (`upload-imap-drafts`) + reply sync (`check-replies`).
2. **Outlook / Zoho / Fastmail / Custom cPanel SMTP**: Standard port `587` (STARTTLS) or `465` (SSL).
3. **Resend API** (`type: "resend"`, free tier 100 emails/day, 3,000/month) or **Brevo API** (`type: "brevo"`, free tier 300 emails/day): Set `RESEND_API_KEY` or `BREVO_API_KEY` and verify your domain's DKIM/SPF in their dashboard.

---

## 6. Legal & Ethical Compliance Checklist (CAN-SPAM / GDPR B2B / CASL)

- **Target Relevant Business Contacts Only**: Only reach out to public business/studio contacts who genuinely source, curate, display, or cover artwork (legitimate B2B interest).
- **Truthful Identity & Subject**: Use your real artist/studio name, real reply-to address, and non-deceptive subject lines.
- **Clear Opt-Out**: Every Touch 1 includes a frictionless opt-out line (`opt_out_line` in `artist_profile.json`).
- **Honor Opt-Outs Permanently**: Use `art-outreach crm suppress --email ...` (and `art-outreach outreach check-replies`) so opted-out or bounced addresses are permanently blocked across all future campaigns.
- **Physical Studio Address (US CAN-SPAM)**: For commercial US campaigns, include your studio city/mailing address or PO Box in your `signature` field in `artist_profile.json`.
