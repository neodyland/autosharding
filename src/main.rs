mod sys;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Autosharding system v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
} // Oh, hi. えも
// やあ