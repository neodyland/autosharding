use crate::pb::{auto_sharding_service_client::AutoShardingServiceClient, AcquireRequest};
use crate::Result;

use tonic::transport::Channel;
use tonic::Request;

pub struct Client {
    client: AutoShardingServiceClient<Channel>,
}

impl Client {
    pub async fn connect(addr: String) -> Result<Self> {
        let client = AutoShardingServiceClient::connect(addr).await?;
        Ok(Self { client })
    }

    pub async fn get_shard_id(&mut self) -> Result<u32> {
        let request = Request::new(AcquireRequest {});
        let response = self.client.acquire(request).await?;
        let data = response.get_ref();
        Ok(data.shard_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_shard() {
        let mut client = Client::connect("http://localhost:1919".to_string())
            .await
            .unwrap();
        let shard_id = client.get_shard_id().await.unwrap();
        println!("{}", shard_id);
    }

    #[tokio::test]
    async fn get_many_shards() {
        let mut client = Client::connect("http://localhost:1919".to_string())
            .await
            .unwrap();

        for _ in 0..1000 {
            println!("{}", client.get_shard_id().await.unwrap());
        }
    }
}
