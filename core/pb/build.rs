use std::{env, path::PathBuf};

const OUT_DIR: &str = "generated";

macro_rules! define_attr {
    ($const: ident, $($attr: pat$(,)*)*) => {
        const $const: &str = concat!("#[derive(", $(stringify!($attr,)),*, ")]");
    };
}

define_attr!(SERDE, serde::Deserialize, serde::Serialize);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build("logs", None);
    build("peace_db", None);
    build("chat", None);
    build("bancho", None);
    build(
        "bancho_state",
        Some(&[(
            SERDE,
            &["UserData", "ConnectionInfo", "GetAllSessionsResponse"],
        )]),
    );

    Ok(())
}

fn descriptor(pkg: &str) -> PathBuf {
    out_dir().join(format!("{}_descriptor.bin", pkg))
}

fn proto(pkg: &str) -> String {
    format!("proto/{}.proto", pkg)
}

fn build(pkg: &str, type_attrs: Option<&[(&str, &[&str])]>) {
    configure(type_attrs)
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

fn configure(type_attrs: Option<&[(&str, &[&str])]>) -> tonic_build::Builder {
    let mut cfg = tonic_build::configure().out_dir(out_dir());

    if let Some(type_attrs) = type_attrs {
        for (attr, paths) in type_attrs {
            for path in paths.iter() {
                cfg = cfg.type_attribute(path, attr);
            }
        }
    }

    cfg
}
