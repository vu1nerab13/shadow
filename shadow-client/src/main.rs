#![windows_subsystem = "windows"]

use anyhow::Result as AppResult;
use clap::Parser;
use flexi_logger::Logger;
use shadow_client::{network, AppArgs};

#[tokio::main]
async fn main() -> AppResult<()> {
    // Parse arguments
    let args = AppArgs::parse();

    #[cfg(debug_assertions)]
    Logger::try_with_str(args.verbose)?.start()?;

    // Server config
    let server_cfg = network::Config::new(args.server_addr.parse()?);
    network::run(server_cfg).await?;

    Ok(())
}
