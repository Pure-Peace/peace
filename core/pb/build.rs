use std::{
    env,
    path::{Path, PathBuf},
};

const OUT_DIR: &str = "generated";

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
    configure()
        .file_descriptor_set_path(file("logs_descriptor.bin"))
        .compile(&["proto/logs.proto"], &["proto"])
        .unwrap();
}

#[cfg(feature = "peace_db")]
fn build_peace_db() {
    configure()
        .file_descriptor_set_path(file("peace_db_descriptor.bin"))
        .compile(&["proto/peace_db.proto"], &["proto"])
        .unwrap();
}

#[cfg(feature = "chat")]
fn build_chat() {
    configure()
        .file_descriptor_set_path(file("chat_descriptor.bin"))
        .compile(&["proto/chat.proto"], &["proto"])
        .unwrap();
}

#[cfg(feature = "bancho")]
fn build_bancho() {
    configure()
        .file_descriptor_set_path(file("bancho_descriptor.bin"))
        .compile(&["proto/bancho.proto"], &["proto"])
        .unwrap();
}

#[cfg(feature = "bancho_state")]
fn build_bancho_state() {
    configure()
        .file_descriptor_set_path(file("bancho_state_descriptor.bin"))
        .compile(&["proto/bancho_state.proto"], &["proto"])
        .unwrap();
}

fn out_dir() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(OUT_DIR);
    if !dir.exists() {
        std::fs::create_dir_all(dir.clone()).unwrap();
    }
    dir
}

fn configure() -> tonic_build::Builder {
    tonic_build::configure().out_dir(out_dir())
}

fn file(path: impl AsRef<Path>) -> PathBuf {
    out_dir().join(path)
}
