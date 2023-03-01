use std::{env, path::PathBuf};

const OUT_DIR: &str = "generated";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build("logs");
    build("peace_db");
    build("chat");
    build("bancho");
    build("bancho_state");

    Ok(())
}

fn descriptor(pkg: &str) -> PathBuf {
    out_dir().join(format!("{}_descriptor.bin", pkg))
}

fn proto(pkg: &str) -> String {
    format!("proto/{}.proto", pkg)
}

fn build(pkg: &str) {
    configure()
        .file_descriptor_set_path(descriptor(pkg))
        .compile(&[proto(pkg)], &["proto"])
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
