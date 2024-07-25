use std::sync::Arc;

use crate::pb;
use tokio::sync::{Mutex, Semaphore};
use tonic::{Code, Request, Response, Status};

use pb::{
    auto_sharding_service_server::AutoShardingService, AcquireRequest, AcquireResponse,
    ReleaseRequest, ReleaseResponse,
};

use crate::prelude::*;

pub enum ShardStatus {
    Starting,
    Stopped,
}

pub struct Shard {
    id: u32,
    status: ShardStatus,
}

pub struct ShardService {
    pub bucket: Arc<Semaphore>,
    pub shards: Mutex<Vec<Shard>>,
}

impl ShardService {
    pub fn new(shard_count: usize, max_concurrency: usize) -> Self {
        let mut shards = Vec::with_capacity(shard_count);

        for shard_id in 0..shard_count {
            shards.push(Shard {
                id: shard_id as _,
                status: ShardStatus::Stopped,
            })
        }

        Self {
            bucket: Arc::new(Semaphore::new(max_concurrency)),
            shards: Mutex::new(shards),
        }
    }
}

#[tonic::async_trait]
impl AutoShardingService for ShardService {
    async fn acquire(
        &self,
        _: Request<AcquireRequest>,
    ) -> Result<Response<AcquireResponse>, Status> {
        let mut shard = None;
        let mut shards = self.shards.lock().await;

        for maybe_stopped_shard in shards.iter_mut() {
            if let ShardStatus::Stopped = maybe_stopped_shard.status {
                shard.replace(maybe_stopped_shard);
            }
        }

        let shard = shard
            .ok_or_else(|| Status::new(Code::ResourceExhausted, "All shards are acquired."))?;

        // Discordから提示された５秒の間に接続可能なシャードの数を超えないように待機する。
        let permit = Arc::clone(&self.bucket)
            .acquire_owned()
            .await
            .map_err(|_| {
                Status::new(
                    Code::Internal,
                    "Failed to wait for rate limit of max currency.",
                )
            })?;
        tokio::spawn(async {
            let _ = tokio::time::sleep(std::time::Duration::from_secs(5));
            drop(permit);
        });

        shard.status = ShardStatus::Starting;

        Ok(Response::new(AcquireResponse { shard_id: shard.id }))
    }

    async fn release(
        &self,
        request: Request<ReleaseRequest>,
    ) -> Result<Response<ReleaseResponse>, Status> {
        let mut shards = self.shards.lock().await;
        let shard_id = request.get_ref().shard_id;
        let shard = shards
            .get_mut(shard_id as usize)
            .ok_or_else(|| Status::new(Code::NotFound, "Requested shard id not exists."))?;

        shard.status = ShardStatus::Stopped;

        Ok(Response::new(ReleaseResponse {}))
    }
}
