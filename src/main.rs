use std::env;
mod service;
mod sys;

use sys::ShardManager;
use tonic::transport::Server;

pub mod prelude {
    pub use anyhow::{Context as _, Error, Result};
}

use service::ShardService;

pub mod pb {
    tonic::include_proto!("auto_sharding.v1");
}

use pb::auto_sharding_service_server::AutoShardingServiceServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    let shard_count = env::var("SHARD_COUNT")?;

    tracing::info!("Autosharding system v{}", env!("CARGO_PKG_VERSION"));

    let shard_service = ShardService {
        manager: tokio::sync::Mutex::new(ShardManager::new(shard_count.parse::<usize>()?, 5)),
    };

    Server::builder()
        .add_service(AutoShardingServiceServer::new(shard_service))
        .serve(env::var("ADDRESS")?.parse()?)
        .await?;
    // pb::auto_sharding_service_server::AutoShardingServiceServer::new

    Ok(())
}
