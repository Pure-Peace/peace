fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "logs")]
    build_peace_logs();

    #[cfg(feature = "db")]
    build_peace_db();

    #[cfg(feature = "bancho")]
    build_bancho();

    Ok(())
}

#[cfg(feature = "logs")]
fn build_peace_logs() {
    tonic_build::configure()
        .compile(&["proto/frame/logs.proto"], &["proto"])
        .unwrap();
}

#[cfg(feature = "db")]
fn build_peace_db() {
    tonic_build::compile_protos("proto/services/db.proto").unwrap();
}

#[cfg(feature = "bancho")]
fn build_bancho() {
    tonic_build::compile_protos("proto/services/bancho.proto").unwrap();
}
