#[cfg(feature = "grpc")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("protobuf/peace_logs.proto")?;
    Ok(())
}

#[cfg(not(feature = "grpc"))]
fn main() { }
