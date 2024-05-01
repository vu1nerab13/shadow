use anyhow::Result as AppResult;
use flexi_logger::Logger;
use shadow_client::network;

#[tokio::main]
async fn main() -> AppResult<()> {
    Logger::try_with_str("trace")?.start()?;

    network::run().await?;

    Ok(())
}
