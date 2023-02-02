use std::{
    env,
    path::{Path, PathBuf},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "logs")]
    build_peace_logs();

    #[cfg(feature = "peace_db")]
    build_peace_db();

    #[cfg(feature = "chat")]
    build_chat();

    #[cfg(feature = "bancho")]
    build_bancho();

    #[cfg(feature = "bancho_state")]
    build_bancho_state();

    Ok(())
}

#[cfg(feature = "logs")]
fn build_peace_logs() {
    tonic_build::configure()
        .file_descriptor_set_path(with_out_dir("logs_descriptor.bin"))
        .compile(&["proto/frame/logs.proto"], &["proto"])
        .unwrap();
}

#[cfg(feature = "peace_db")]
fn build_peace_db() {
    tonic_build::configure()
        .file_descriptor_set_path(with_out_dir("peace_db_descriptor.bin"))
        .compile(&["proto/services/peace_db.proto"], &["proto"])
        .unwrap();
}

#[cfg(feature = "chat")]
fn build_chat() {
    tonic_build::configure()
        .file_descriptor_set_path(with_out_dir("chat_descriptor.bin"))
        .compile(&["proto/services/chat.proto"], &["proto"])
        .unwrap();
}

#[cfg(feature = "bancho")]
fn build_bancho() {
    tonic_build::configure()
        .file_descriptor_set_path(with_out_dir("bancho_descriptor.bin"))
        .compile(&["proto/services/bancho.proto"], &["proto"])
        .unwrap();
}

#[cfg(feature = "bancho_state")]
fn build_bancho_state() {
    tonic_build::configure()
        .file_descriptor_set_path(with_out_dir("bancho_state_descriptor.bin"))
        .compile(&["proto/services/bancho_state.proto"], &["proto"])
        .unwrap();
}

fn with_out_dir(path: impl AsRef<Path>) -> PathBuf {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    out_dir.join(path)
}
