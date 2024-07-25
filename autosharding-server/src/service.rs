use crate::{pb, sys::ShardManager};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use pb::{
    auto_sharding_service_server::AutoShardingService, AcquireRequest, AcquireResponse,
    ReleaseRequest, ReleaseResponse,
};

pub struct ShardService {
    pub manager: Mutex<ShardManager>,
}

#[tonic::async_trait]
impl AutoShardingService for ShardService {
    async fn acquire(
        &self,
        _: Request<AcquireRequest>,
    ) -> Result<Response<AcquireResponse>, Status> {
        Ok(Response::new(AcquireResponse {
            shard_id: self.manager.lock().await.acquire().await?,
        }))
    }

    async fn release(
        &self,
        request: Request<ReleaseRequest>,
    ) -> Result<Response<ReleaseResponse>, Status> {
        self.manager
            .lock()
            .await
            .release(request.get_ref().shard_id)
            .await?;
        Ok(Response::new(ReleaseResponse {}))
    }
}
