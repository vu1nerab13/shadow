#![windows_subsystem = "windows"]

use anyhow::Result as AppResult;
use flexi_logger::Logger;
use shadow_client::network;

#[tokio::main]
async fn main() -> AppResult<()> {
    #[cfg(debug_assertions)]
    Logger::try_with_str("trace")?.start()?;

    // Server config
    let server_cfg = network::Config::new("192.168.5.5:1244".parse()?);
    network::run(server_cfg).await?;

    Ok(())
}
