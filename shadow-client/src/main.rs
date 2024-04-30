use anyhow::Result as AppResult;
use shadow_client::network;

#[tokio::main]
async fn main() -> AppResult<()> {
    network::server::run().await?;

    Ok(())
}
