use std::sync::Arc;

use tokio::sync::Semaphore;
use tonic::{Code, Status};

use crate::prelude::*;

pub enum ShardStatus {
    Starting,
    Stopped,
}

pub struct Shard {
    id: u32,
    status: ShardStatus,
}

pub struct ShardManager {
    pub bucket: Arc<Semaphore>,
    pub shards: Vec<Shard>,
}

impl ShardManager {
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
            shards,
        }
    }

    pub async fn acquire(&mut self) -> Result<u32, Status> {
        let mut shard = None;

        for maybe_stopped_shard in self.shards.iter_mut() {
            if let ShardStatus::Stopped = maybe_stopped_shard.status {
                shard.replace(maybe_stopped_shard);
                break;
            }
        }

        let shard = shard
            .ok_or_else(|| Status::new(Code::ResourceExhausted, "All shards are acquired."))?;

        // シャードの接続処理の数が、Discordから告げられた`max_concurrency`の数を
        // 超えないようにする。そのためにセマフォを使う。これはタスクで５秒後に解放される。
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
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            drop(permit);
        });

        shard.status = ShardStatus::Starting;

        Ok(shard.id)
    }

    pub async fn release(&mut self, shard_id: u32) -> Result<(), Status> {
        let shard = self
            .shards
            .get_mut(shard_id as usize)
            .ok_or_else(|| Status::new(Code::NotFound, "Requested shard id not exists."))?;

        shard.status = ShardStatus::Stopped;

        Ok(())
    }
}
