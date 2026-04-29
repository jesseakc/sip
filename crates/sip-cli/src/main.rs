use clap::Parser;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "sip-cli")]
#[command(about = "SIP admin and developer CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Run database migrations
    Migrate,
    /// Seed demo data
    Seed,
    /// Export tenant data
    Export,
    /// Reindex search vectors
    Reindex,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = sip_observability::init_tracing();
    let cli = Cli::parse();

    match cli.command {
        Commands::Migrate => {
            info!("Running migrations...");
        }
        Commands::Seed => {
            info!("Seeding demo data...");
        }
        Commands::Export => {
            info!("Exporting tenant data...");
        }
        Commands::Reindex => {
            info!("Reindexing search vectors...");
        }
    }

    Ok(())
}
