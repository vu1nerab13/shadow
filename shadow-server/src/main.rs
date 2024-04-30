use anyhow::Result as AppResult;
use shadow_server::network;

#[tokio::main]
async fn main() -> AppResult<()> {
    network::server::run().await?;

    Ok(())
}
