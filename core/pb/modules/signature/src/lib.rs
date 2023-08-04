#![allow(dead_code)]
#![allow(non_snake_case)]

mod peace {
    use pb_base as base;

    pub mod services {
        pub mod signature {
            include!("../../../generated/peace.services.signature.rs");

            pub const SIGNATURE_DESCRIPTOR_SET: &[u8] = include_bytes!(
                "../../../generated/peace.services.signature.descriptor.bin"
            );
        }
    }
}

pub use peace::services::signature::*;
