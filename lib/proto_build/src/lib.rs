use std::path::PathBuf;

pub const PROTO_EXT: &str = "proto";
pub const DESCRIPTOR_EXT: &str = "descriptor.bin";

#[derive(Debug, Clone)]
pub struct StructAttr<'a> {
    pub attr: &'a str,
    pub structs: &'a [&'a str],
}

impl<'a> StructAttr<'a> {
    pub fn new(attr: &'a str, structs: &'a [&'a str]) -> Self {
        Self { attr, structs }
    }
}

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

    pub fn descriptor_output(
        &self,
        pkg: &str,
    ) -> Result<PathBuf, std::io::Error> {
        Ok(self
            .out_dir()?
            .join(format!("{}.{pkg}.{DESCRIPTOR_EXT}", self.namespace)))
    }

    pub fn proto_path(&self, pkg: &str) -> PathBuf {
        PathBuf::new()
            .join(&self.proto_root)
            .join(&self.namespace)
            .join(format!("{}.{PROTO_EXT}", pkg.replace('.', "/")))
    }

    pub fn build(&self, pkg: &str) -> Result<(), std::io::Error> {
        self.configure()?
            .file_descriptor_set_path(self.descriptor_output(pkg)?)
            .compile(&[self.proto_path(pkg)], &[&self.proto_root])
    }

    pub fn build_with_attrs(
        &self,
        pkg: &str,
        struct_attrs: &[StructAttr],
    ) -> Result<(), std::io::Error> {
        self.configure_with_attrs(struct_attrs)?
            .file_descriptor_set_path(self.descriptor_output(pkg)?)
            .compile(&[self.proto_path(pkg)], &[&self.proto_root])
    }

    pub fn out_dir(&self) -> Result<PathBuf, std::io::Error> {
        let path = PathBuf::from(&self.out_dir);
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        Ok(path)
    }

    pub fn configure(&self) -> Result<tonic_build::Builder, std::io::Error> {
        Ok(tonic_build::configure().out_dir(self.out_dir()?))
    }

    pub fn configure_with_attrs(
        &self,
        struct_attrs: &[StructAttr],
    ) -> Result<tonic_build::Builder, std::io::Error> {
        let mut cfg = self.configure()?;

        for StructAttr { attr, structs } in struct_attrs {
            for s in structs.iter() {
                cfg = cfg.type_attribute(s, attr);
            }
        }

        Ok(cfg)
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
