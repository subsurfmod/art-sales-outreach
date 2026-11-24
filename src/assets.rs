//! Real-Asset Authenticity Gate (Zero AI-Generated Artwork / Artist Protocol).
//!
//! Enforces that EVERY artwork image, detail crop, studio clip, and artist portrait
//! used in websites, portfolios, outreach lookbooks, and 3D/2D social videos is a
//! verified authentic artist-supplied asset — NEVER AI-generated artwork or an AI
//! likeness of the artist.

use anyhow::{bail, Context, Result};
use clap::Subcommand;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

pub const DEFAULT_MANIFEST_REL: &str = "art-outreach-workspace/real_assets/manifest.json";

const MEDIA_EXTS: &[&str] = &[
    "jpg", "jpeg", "png", "webp", "tif", "tiff", "avif", "mp4", "mov", "webm", "m4v",
];
const CODE_EXTS: &[&str] = &[
    "html", "htm", "css", "js", "mjs", "cjs", "ts", "tsx", "jsx", "json", "py", "md",
];

/// Binary / metadata / filename signatures strongly associated with AI image generators.
const AI_METADATA_MARKERS: &[&str] = &[
    "midjourney",
    "dall-e",
    "dall·e",
    "openai_image",
    "stable diffusion",
    "stablediffusion",
    "automatic1111",
    "comfyui",
    "invokeai",
    "novelai",
    "adobe firefly",
    "c2pa.actions",
    "trainedalgorithmicmedia",
    "digitalSourceType/trainedAlgorithmicMedia",
    "imagen",
    "black forest labs",
    "flux.1",
    "ideogram",
    "leonardo.ai",
    "heygen",
    "runwayml",
    "klingai",
    "luma dream machine",
];

const AI_FILENAME_MARKERS: &[&str] = &[
    "midjourney", "dalle", "dall-e", "stablediffusion", "comfyui", "ai_gen", "aigen",
    "ai-generated", "generated_art", "fake_art", "placeholder_art", "flux_", "heygen",
];

const DISALLOWED_EXTERNAL_MEDIA_HOSTS: &[&str] = &[
    "unsplash.com",
    "images.unsplash.com",
    "pexels.com",
    "pixabay.com",
    "oaidalleapiprodscus.blob.core.windows.net",
    "cdn.midjourney.com",
    "replicate.delivery",
    "fal.media",
    "picsum.photos",
    "via.placeholder.com",
    "placehold.co",
];

static MEDIA_REF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)["'(=]\s*([a-zA-Z0-9_./:@\-]+\.(?:jpe?g|png|webp|avif|gif|mp4|mov|webm|tiff?))(?:\?[^"' )]*)?["' )]"#).unwrap()
});

#[derive(Subcommand, Debug)]
pub enum AssetsCommand {
    /// Register real artist-supplied artwork/studio/portrait files into real_assets/manifest.json with SHA-256 & AI-marker screening
    Register {
        /// Folder containing real artist-supplied photos/scans/videos
        #[arg(long)]
        dir: PathBuf,
        /// Output manifest path
        #[arg(long, default_value = DEFAULT_MANIFEST_REL)]
        manifest: PathBuf,
        /// Default asset kind (artwork, detail, studio, artist_portrait, installation, video)
        #[arg(long, default_value = "artwork")]
        kind: String,
    },
    /// Audit a website, portfolio, or 3D/2D video project to guarantee 100% real artist assets (exits non-zero on violation)
    Audit {
        /// Path to the generated website or video project directory to audit
        #[arg(long)]
        target: PathBuf,
        /// Path to verified real_assets/manifest.json
        #[arg(long, default_value = DEFAULT_MANIFEST_REL)]
        manifest: PathBuf,
    },
    /// Print a clean portfolio catalog JSON/summary of registered authentic artworks
    Catalog {
        #[arg(long, default_value = DEFAULT_MANIFEST_REL)]
        manifest: PathBuf,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetEntry {
    pub id: String,
    pub filename: String,
    pub rel_path: String,
    pub kind: String,
    pub sha256: String,
    pub bytes: u64,
    pub title: String,
    pub year: String,
    pub medium: String,
    pub dimensions_in: String,
    pub dimensions_cm: String,
    pub series: String,
    pub price: String,
    pub availability: String,
    pub provenance: String,
    pub verified_real: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetManifest {
    pub protocol: String,
    pub updated_at: String,
    pub assets: Vec<AssetEntry>,
}

/// Pure-Rust SHA-256 (FIPS 180-4) — zero extra dependencies.
pub fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    h.iter().map(|v| format!("{v:08x}")).collect()
}

/// Inspect filename + file header/trailer metadata bytes for AI generation provenance markers.
pub fn detect_ai_markers(path: &Path, bytes: &[u8]) -> Vec<String> {
    let mut hits = Vec::new();
    let fname = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    for m in AI_FILENAME_MARKERS {
        if fname.contains(m) {
            hits.push(format!("filename:{m}"));
        }
    }

    // Scan first 128 KB and last 32 KB for EXIF / PNG tEXt / XMP / C2PA AI software signatures
    let head = &bytes[..bytes.len().min(131_072)];
    let tail_start = bytes.len().saturating_sub(32_768);
    let tail = if tail_start > 0 { &bytes[tail_start..] } else { &[] };
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(head).to_lowercase(),
        String::from_utf8_lossy(tail).to_lowercase()
    );
    for marker in AI_METADATA_MARKERS {
        if combined.contains(marker) {
            hits.push(format!("metadata:{marker}"));
        }
    }
    // Classic Stable Diffusion PNG tEXt chunk signature: "negative prompt:" + "steps:"
    if combined.contains("negative prompt:") && combined.contains("steps:") {
        hits.push("metadata:stable_diffusion_parameters".to_string());
    }
    hits
}

fn collect_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let p = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') || matches!(name_str.as_ref(), "node_modules" | "target" | ".git") {
            continue;
        }
        if p.is_dir() {
            collect_files(&p, out)?;
        } else if p.is_file() {
            out.push(p);
        }
    }
    Ok(())
}

fn ext_lower(p: &Path) -> String {
    p.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}

pub fn handle_command(cmd: AssetsCommand) -> Result<bool> {
    match cmd {
        AssetsCommand::Register {
            dir,
            manifest,
            kind,
        } => {
            if !dir.exists() {
                bail!("Assets directory does not exist: {}", dir.display());
            }
            let existing: BTreeMap<String, AssetEntry> = if manifest.exists() {
                let m: AssetManifest = serde_json::from_str(&fs::read_to_string(&manifest)?)?;
                m.assets.into_iter().map(|a| (a.sha256.clone(), a)).collect()
            } else {
                BTreeMap::new()
            };

            let mut files = Vec::new();
            collect_files(&dir, &mut files)?;
            files.sort();

            let mut registered = Vec::new();
            let mut rejected = Vec::new();
            let mut seq = existing.len();

            for path in files {
                let ext = ext_lower(&path);
                if !MEDIA_EXTS.contains(&ext.as_str()) {
                    continue;
                }
                let bytes = fs::read(&path)
                    .with_context(|| format!("Failed to read {}", path.display()))?;
                let ai_hits = detect_ai_markers(&path, &bytes);
                if !ai_hits.is_empty() {
                    rejected.push(json!({
                        "file": path.display().to_string(),
                        "reason": "AI-generated image/video marker detected (STRICT ZERO-AI-ART PROTOCOL)",
                        "markers": ai_hits,
                    }));
                    continue;
                }

                let digest = sha256_hex(&bytes);
                let rel = path
                    .strip_prefix(&dir)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                let fname = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("asset")
                    .to_string();
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled")
                    .replace(['_', '-'], " ");

                let inferred_kind = if matches!(ext.as_str(), "mp4" | "mov" | "webm" | "m4v") {
                    "video".to_string()
                } else if fname.to_lowercase().contains("portrait") || fname.to_lowercase().contains("artist") {
                    "artist_portrait".to_string()
                } else if fname.to_lowercase().contains("detail") || fname.to_lowercase().contains("macro") {
                    "detail".to_string()
                } else if fname.to_lowercase().contains("studio") {
                    "studio".to_string()
                } else if fname.to_lowercase().contains("install") || fname.to_lowercase().contains("wall") {
                    "installation".to_string()
                } else {
                    kind.clone()
                };

                let entry = if let Some(prev) = existing.get(&digest) {
                    let mut cloned = prev.clone();
                    cloned.filename = fname;
                    cloned.rel_path = rel;
                    cloned
                } else {
                    seq += 1;
                    AssetEntry {
                        id: format!("ART-{seq:03}"),
                        filename: fname,
                        rel_path: rel,
                        kind: inferred_kind,
                        sha256: digest,
                        bytes: bytes.len() as u64,
                        title: stem,
                        year: String::new(),
                        medium: String::new(),
                        dimensions_in: String::new(),
                        dimensions_cm: String::new(),
                        series: String::new(),
                        price: "Price on Request".to_string(),
                        availability: "available".to_string(),
                        provenance: "Artist Studio".to_string(),
                        verified_real: true,
                    }
                };
                registered.push(entry);
            }

            if let Some(parent) = manifest.parent() {
                fs::create_dir_all(parent)?;
            }
            let out_manifest = AssetManifest {
                protocol: "STRICT_REAL_ARTWORK_ONLY_V1".to_string(),
                updated_at: crate::crm::utc_now(),
                assets: registered,
            };
            fs::write(&manifest, serde_json::to_string_pretty(&out_manifest)?)?;

            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "manifest": manifest.display().to_string(),
                    "registered_real_assets": out_manifest.assets.len(),
                    "rejected_ai_suspects": rejected.len(),
                    "rejected": rejected,
                }))?
            );
            Ok(rejected.is_empty())
        }
        AssetsCommand::Audit { target, manifest } => {
            if !target.exists() {
                bail!("Audit target does not exist: {}", target.display());
            }
            let m: AssetManifest = serde_json::from_str(
                &fs::read_to_string(&manifest)
                    .with_context(|| format!("Missing real_assets manifest at {}. Register artist photos first with `art-outreach assets register`.", manifest.display()))?,
            )?;
            let valid_hashes: HashSet<String> = m.assets.iter().map(|a| a.sha256.clone()).collect();
            let valid_names: HashSet<String> = m
                .assets
                .iter()
                .map(|a| a.filename.to_lowercase())
                .collect();

            let mut files = Vec::new();
            collect_files(&target, &mut files)?;

            let mut violations = Vec::new();
            let mut verified_media_files = 0usize;
            let mut scanned_code_files = 0usize;

            for path in files {
                let ext = ext_lower(&path);
                if MEDIA_EXTS.contains(&ext.as_str()) {
                    let bytes = fs::read(&path)?;
                    let ai_hits = detect_ai_markers(&path, &bytes);
                    if !ai_hits.is_empty() {
                        violations.push(json!({
                            "type": "AI_GENERATED_MEDIA_DETECTED",
                            "file": path.display().to_string(),
                            "markers": ai_hits,
                        }));
                        continue;
                    }
                    let digest = sha256_hex(&bytes);
                    if !valid_hashes.contains(&digest) {
                        violations.push(json!({
                            "type": "UNREGISTERED_MEDIA_FILE",
                            "file": path.display().to_string(),
                            "hint": "Every artwork/artist image or video MUST be registered in real_assets/manifest.json.",
                        }));
                    } else {
                        verified_media_files += 1;
                    }
                } else if CODE_EXTS.contains(&ext.as_str()) {
                    scanned_code_files += 1;
                    let text = fs::read_to_string(&path).unwrap_or_default();
                    let lower = text.to_lowercase();
                    for host in DISALLOWED_EXTERNAL_MEDIA_HOSTS {
                        if lower.contains(host) {
                            violations.push(json!({
                                "type": "DISALLOWED_STOCK_OR_AI_HOST",
                                "file": path.display().to_string(),
                                "host": host,
                            }));
                        }
                    }
                    for cap in MEDIA_REF_RE.captures_iter(&text) {
                        let raw_ref = cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        let fname = raw_ref.rsplit('/').next().unwrap_or(raw_ref).to_lowercase();
                        // Ignore obvious UI icons/favicons/paper grain textures
                        if fname.contains("favicon")
                            || fname.contains("grain")
                            || fname.contains("icon")
                            || fname.contains("og-card")
                        {
                            continue;
                        }
                        if !valid_names.is_empty() && !valid_names.contains(&fname) {
                            violations.push(json!({
                                "type": "UNREGISTERED_MEDIA_REFERENCE",
                                "file": path.display().to_string(),
                                "reference": raw_ref,
                            }));
                        }
                    }
                }
            }

            let passed = violations.is_empty();
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "protocol": "STRICT_REAL_ARTWORK_ONLY_V1",
                    "target": target.display().to_string(),
                    "passed": passed,
                    "verified_media_files": verified_media_files,
                    "scanned_code_files": scanned_code_files,
                    "violation_count": violations.len(),
                    "violations": violations,
                }))?
            );
            Ok(passed)
        }
        AssetsCommand::Catalog { manifest } => {
            let m: AssetManifest = serde_json::from_str(&fs::read_to_string(&manifest)?)?;
            println!("{}", serde_json::to_string_pretty(&m)?);
            Ok(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn flags_ai_markers_and_allows_real_camera_bytes() {
        let fake_sd = b"\x89PNG\r\n\x1a\n...tEXtparameters\x00oil painting\nNegative prompt: blurry\nSteps: 30, Sampler: Euler";
        let hits = detect_ai_markers(Path::new("painting_01.png"), fake_sd);
        assert!(!hits.is_empty());

        let fake_mj = b"\xff\xd8\xff\xe0...Software: Midjourney v6...";
        let hits2 = detect_ai_markers(Path::new("studio.jpg"), fake_mj);
        assert!(!hits2.is_empty());

        let real_camera = b"\xff\xd8\xff\xe1...FUJIFILM X-T5...2026:11:12 14:22:01...";
        let hits3 = detect_ai_markers(Path::new("DSC_4821_canvas_linen.jpg"), real_camera);
        assert!(hits3.is_empty());
    }
}
