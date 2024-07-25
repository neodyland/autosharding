pub mod client;

pub mod pb {
    tonic::include_proto!("auto_sharding.v1");
}

pub use client::Client;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
