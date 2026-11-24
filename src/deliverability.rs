use crate::verify::build_resolver;
use anyhow::Result;
use hickory_resolver::TokioAsyncResolver;
use serde_json::json;
use std::collections::HashSet;

const COMMON_DKIM_SELECTORS: &[&str] = &[
    "google", "selector1", "selector2", "default", "mail", "dkim",
    "resend", "k1", "s1", "s2", "zmail", "fm1", "protonmail", "pm",
];

async fn query_txt(resolver: &TokioAsyncResolver, name: &str) -> Vec<String> {
    match resolver.txt_lookup(name).await {
        Ok(lookup) => lookup
            .iter()
            .map(|r| {
                r.txt_data()
                    .iter()
                    .map(|b| String::from_utf8_lossy(b).into_owned())
                    .collect::<String>()
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}

pub async fn run_deliverability(domain_or_email: &str, extra_selectors: &[String]) -> Result<bool> {
    let raw = domain_or_email.trim().to_lowercase();
    let domain = raw.split_once('@').map(|(_, d)| d).unwrap_or(&raw);
    let resolver = build_resolver();

    let mut score = 0i64;
    let mut warnings: Vec<String> = Vec::new();
    let mut recommendations: Vec<String> = Vec::new();

    // 1. MX
    let mx: Vec<String> = match resolver.mx_lookup(domain).await {
        Ok(lookup) => lookup
            .iter()
            .map(|r| r.exchange().to_utf8().trim_end_matches('.').to_string())
            .collect(),
        Err(_) => Vec::new(),
    };
    if !mx.is_empty() {
        score += 25;
    } else {
        warnings.push("No MX records found — replies to this domain may bounce.".into());
    }

    // 2. SPF
    let root_txt = query_txt(&resolver, domain).await;
    let spf_records: Vec<String> = root_txt
        .into_iter()
        .filter(|t| t.to_lowercase().starts_with("v=spf1"))
        .collect();
    let spf_val = if spf_records.len() == 1 {
        score += 25;
        if spf_records[0].contains("+all") {
            warnings.push("SPF uses '+all' (insecure). Change to '~all' or '-all'.".into());
            score -= 10;
        }
        json!(spf_records[0])
    } else if spf_records.len() > 1 {
        warnings.push("Multiple SPF records found — RFC 7208 invalidates SPF when >1 exist. Merge into one TXT record.".into());
        json!(spf_records)
    } else {
        warnings.push("Missing SPF record — Gmail/Outlook will likely flag cold outreach as spam.".into());
        recommendations.push(format!(
            r#"Add TXT record on {domain}: "v=spf1 include:_spf.google.com ~all" (adjust include for your mail provider)."#
        ));
        json!(null)
    };

    // 3. DMARC
    let dmarc_txt = query_txt(&resolver, &format!("_dmarc.{domain}")).await;
    let dmarc_records: Vec<String> = dmarc_txt
        .into_iter()
        .filter(|t| t.to_lowercase().starts_with("v=dmarc1"))
        .collect();
    let dmarc_val = if let Some(first) = dmarc_records.first() {
        score += 25;
        json!(first)
    } else {
        warnings.push("Missing DMARC record — required by Gmail & Yahoo bulk sender rules.".into());
        recommendations.push(format!(
            r#"Add TXT record on _dmarc.{domain}: "v=DMARC1; p=none; rua=mailto:dmarc@{domain}; adkim=r; aspf=r""#
        ));
        json!(null)
    };

    // 4. DKIM
    let mut seen = HashSet::new();
    let mut selectors = Vec::new();
    for s in extra_selectors
        .iter()
        .map(|s| s.as_str())
        .chain(COMMON_DKIM_SELECTORS.iter().copied())
    {
        if seen.insert(s.to_string()) {
            selectors.push(s.to_string());
        }
    }
    let mut dkim_found = Vec::new();
    for sel in &selectors {
        let txts = query_txt(&resolver, &format!("{sel}._domainkey.{domain}")).await;
        if txts
            .iter()
            .any(|t| t.to_lowercase().contains("v=dkim1") || t.to_lowercase().contains("p="))
        {
            dkim_found.push(sel.clone());
        }
    }
    if !dkim_found.is_empty() {
        score += 25;
    } else {
        warnings.push("No common DKIM selector detected (or custom selector used). Verify DKIM is enabled in your email host.".into());
        recommendations.push(
            "Enable DKIM signing in Google Workspace / Microsoft 365 / Zoho / Resend / Brevo DNS settings.".into(),
        );
    }

    if matches!(
        domain,
        "gmail.com" | "yahoo.com" | "outlook.com" | "hotmail.com" | "icloud.com"
    ) {
        warnings.push(
            "Using a free consumer mailbox limits safe daily outreach to ~15-25/day and prevents custom DMARC alignment. For higher reach, use a dedicated studio domain (e.g. yourname-studio.com).".into(),
        );
    }

    let readiness = if score >= 90 {
        "EXCELLENT"
    } else if score >= 65 {
        "FAIR"
    } else {
        "NEEDS_FIXES_BEFORE_SENDING"
    };

    let report = json!({
        "domain": domain,
        "mx": mx,
        "spf": spf_val,
        "dmarc": dmarc_val,
        "dkim_found": dkim_found,
        "score": score,
        "readiness": readiness,
        "warnings": warnings,
        "recommendations": recommendations,
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(score >= 65)
}
