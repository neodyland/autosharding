fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/auto_sharding/v1/auto_sharding.proto")?;
    Ok(())
}
