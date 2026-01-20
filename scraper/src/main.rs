use clap::Parser;
use dotenv::dotenv;
use std::path::PathBuf;
use tracing::{info, error};
use tracing_subscriber;

mod gcp;
mod models;
mod transformer;
mod storage;
mod error;

use crate::gcp::GcpClient;
use crate::transformer::DataTransformer;
use crate::storage::StorageManager;

#[derive(Parser, Debug)]
#[command(name = "GCP IAM Scraper")]
#[command(about = "Scrape GCP IAM roles and permissions", long_about = None)]
struct Args {
    /// Output directory for JSON files
    #[arg(short, long, default_value = "./data")]
    output_dir: PathBuf,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Parse CLI arguments
    let args = Args::parse();

    // Initialize logging
    initialize_logging(&args.log_level)?;

    info!("Starting GCP IAM data collection...");
    let start = std::time::Instant::now();

    // Run scraper
    match run_scraper(&args.output_dir).await {
        Ok(_) => {
            let duration = start.elapsed();
            info!("✓ Data collection completed successfully!");
            info!("  - Duration: {:.2}s", duration.as_secs_f64());
            Ok(())
        }
        Err(e) => {
            error!("Fatal error during data collection: {}", e);
            Err(e.into())
        }
    }
}

async fn run_scraper(output_dir: &std::path::Path) -> Result<(), error::ScraperError> {
    // Step 1: Initialize GCP client
    info!("Step 1/4: Initializing GCP client...");
    let client = GcpClient::new().await?;
    info!("✓ GCP client initialized");

    // Step 2: Fetch data from GCP
    info!("Step 2/4: Fetching roles and permissions from GCP IAM API...");
    let raw_data = client.fetch_all_data().await?;
    info!(
        "✓ Fetched {} roles and {} permissions",
        raw_data.roles.len(),
        raw_data.permissions.len()
    );

    // Step 3: Transform data
    info!("Step 3/4: Transforming data to optimized schema...");
    let transformer = DataTransformer::new();
    let dataset = transformer.transform(raw_data)?;
    info!(
        "✓ Transformed {} roles and {} permissions",
        dataset.metadata.total_roles, dataset.metadata.total_permissions
    );

    // Step 4: Store data
    info!("Step 4/4: Saving data to disk...");
    let storage = StorageManager::new(output_dir);
    storage.save(&dataset).await?;
    info!("✓ Data saved successfully");

    // Print summary
    info!("Summary:");
    info!("  - Total roles: {}", dataset.metadata.total_roles);
    info!("  - Total permissions: {}", dataset.metadata.total_permissions);
    info!("  - Last updated: {}", dataset.metadata.last_updated);

    Ok(())
}

fn initialize_logging(level: &str) -> Result<(), Box<dyn std::error::Error>> {
    let level = match level.to_lowercase().as_str() {
        "trace" => tracing::Level::TRACE,
        "debug" => tracing::Level::DEBUG,
        "info" => tracing::Level::INFO,
        "warn" => tracing::Level::WARN,
        "error" => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(false)
        .init();

    Ok(())
}
