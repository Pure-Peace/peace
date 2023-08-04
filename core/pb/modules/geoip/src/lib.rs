#![allow(dead_code)]
#![allow(non_snake_case)]

mod peace {
    use pb_base as base;

    pub mod services {
        pub mod geoip {
            include!("../../../generated/peace.services.geoip.rs");

            pub const GEOIP_DESCRIPTOR_SET: &[u8] = include_bytes!(
                "../../../generated/peace.services.geoip.descriptor.bin"
            );
        }
    }
}

pub use peace::services::geoip::*;
