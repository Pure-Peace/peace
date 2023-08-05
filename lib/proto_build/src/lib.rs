use std::path::PathBuf;

pub const PROTO_EXT: &str = "proto";
pub const DESCRIPTOR_EXT: &str = "descriptor.bin";

#[derive(Debug, Clone)]
pub struct ProtoBuilder {
    pub proto_root: String,
    pub namespace: String,
    pub out_dir: String,
}

impl ProtoBuilder {
    pub fn new<S: Into<String>>(
        proto_root: S,
        namespace: S,
        out_dir: S,
    ) -> Self {
        Self {
            proto_root: proto_root.into(),
            namespace: namespace.into(),
            out_dir: out_dir.into(),
        }
    }

    pub fn descriptor_output(&self, pkg: &str) -> PathBuf {
        self.out_dir()
            .join(format!("{}.{pkg}.{DESCRIPTOR_EXT}", self.namespace))
    }

    pub fn proto_path(&self, pkg: &str) -> PathBuf {
        PathBuf::new()
            .join(&self.proto_root)
            .join(&self.namespace)
            .join(format!("{}.{PROTO_EXT}", pkg.replace('.', "/")))
    }

    pub fn build(&self, pkg: &str, type_attrs: Option<&[(&str, &[&str])]>) {
        self.configure(type_attrs)
            .file_descriptor_set_path(self.descriptor_output(pkg))
            .compile(&[self.proto_path(pkg)], &[&self.proto_root])
            .unwrap();
    }

    pub fn out_dir(&self) -> PathBuf {
        let path = PathBuf::from(&self.out_dir);
        if !path.exists() {
            std::fs::create_dir_all(&path).unwrap();
        }
        path
    }

    pub fn configure(
        &self,
        type_attrs: Option<&[(&str, &[&str])]>,
    ) -> tonic_build::Builder {
        let mut cfg = tonic_build::configure().out_dir(self.out_dir());

        if let Some(type_attrs) = type_attrs {
            for (attr, paths) in type_attrs {
                for path in paths.iter() {
                    cfg = cfg.type_attribute(path, attr);
                }
            }
        }

        cfg
    }
}

#[macro_export]
macro_rules! define_attr {
    ($const: ident, $($attr: pat$(,)*)*) => {
        pub const $const: &str = concat!("#[derive(", $(stringify!($attr,)),*, ")]");
    };
}

pub mod preset_attr {
    define_attr!(SERDE, serde::Deserialize, serde::Serialize);
}
