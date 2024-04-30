use anyhow::Result as AppResult;
use flexi_logger::Logger;
use shadow_server::network;

#[tokio::main]
async fn main() -> AppResult<()> {
    Logger::try_with_str("trace")?.start()?;

    network::server::run().await?;

    Ok(())
}
