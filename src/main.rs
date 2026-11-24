mod assets;
mod crm;
mod deliverability;
mod discover;
mod enrich;
mod imap_client;
mod outreach;
mod studio;
mod verify;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "art-outreach",
    version,
    about = "Full-fledged Rust CLI for physical artwork sales: strict real-asset authenticity auditing, anti-slop website/portfolio ASCII wireframes & variations, 3D/2D video routing, connector lead discovery, async website enrichment, MX verification, and multi-inbox outreach."
)]
struct Cli {
    /// Path to SQLite CRM database
    #[arg(long, global = true, default_value = crm::DEFAULT_DB_REL)]
    db: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Strict Zero-AI-Art / Zero-AI-Artist asset registration, SHA-256 manifest, and build audit
    Assets {
        #[command(subcommand)]
        cmd: assets::AssetsCommand,
    },
    /// Anti-slop Website/Portfolio ASCII wireframes, 3D/2D video variations, and Master Launch Blueprints
    Studio {
        #[command(subcommand)]
        cmd: studio::StudioCommand,
    },
    /// Local SQLite CRM & pipeline commands
    Crm {
        #[command(subcommand)]
        cmd: crm::CrmCommand,
    },
    /// Multi-source lead discovery (Google Boolean queries, Search APIs, OpenStreetMap, public directories)
    Discover {
        #[command(subcommand)]
        cmd: discover::DiscoverCommand,
    },
    /// Async website enrichment & contact/social/hook extraction
    Enrich {
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long, default_value_t = 8)]
        concurrency: usize,
        #[arg(long, default_value_t = 4)]
        max_subpages: usize,
        #[arg(long, default_value = "")]
        category: String,
        #[arg(long)]
        force: bool,
    },
    /// Live DNS MX verification and 0-100 Connector-Multiplier lead scoring
    Verify {
        #[arg(long)]
        profile: Option<PathBuf>,
        #[arg(long)]
        skip_dns: bool,
        #[arg(long, default_value_t = 16)]
        concurrency: usize,
    },
    /// Audit your sending domain's MX, SPF, DKIM, and DMARC records before outreach
    Deliverability {
        /// Sending domain or email address (e.g. hello@yourstudio.com)
        domain: String,
        /// Additional custom DKIM selector(s) to check
        #[arg(long = "dkim-selector")]
        dkim_selectors: Vec<String>,
    },
    /// Multi-touch draft generation, IMAP drafts upload, rotated SMTP/API sending, and reply scanning
    Outreach {
        #[command(subcommand)]
        cmd: outreach::OutreachCommand,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(code) => code,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::from(1)
        }
    }
}

async fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Assets { cmd } => {
            let ok = assets::handle_command(cmd)?;
            if !ok {
                return Ok(ExitCode::from(2));
            }
        }
        Commands::Studio { cmd } => studio::handle_command(cmd)?,
        Commands::Crm { cmd } => crm::handle_command(&cli.db, cmd)?,
        Commands::Discover { cmd } => discover::handle_command(&cli.db, cmd).await?,
        Commands::Enrich {
            limit,
            concurrency,
            max_subpages,
            category,
            force,
        } => {
            enrich::run_enrich(&cli.db, limit, concurrency, max_subpages, &category, force).await?;
        }
        Commands::Verify {
            profile,
            skip_dns,
            concurrency,
        } => {
            verify::run_verify(&cli.db, profile.as_deref(), skip_dns, concurrency).await?;
        }
        Commands::Deliverability {
            domain,
            dkim_selectors,
        } => {
            let ok = deliverability::run_deliverability(&domain, &dkim_selectors).await?;
            if !ok {
                return Ok(ExitCode::from(2));
            }
        }
        Commands::Outreach { cmd } => outreach::handle_command(&cli.db, cmd).await?,
    }
    Ok(ExitCode::SUCCESS)
}
