#![allow(dead_code)]
#![allow(non_snake_case)]

mod peace {
    pub mod services {
        use pb_bancho_state as bancho_state;

        pub mod bancho {
            include!("../../../generated/peace.services.bancho.rs");

            pub const BANCHO_DESCRIPTOR_SET: &[u8] = include_bytes!(
                "../../../generated/peace.services.bancho.descriptor.bin"
            );
        }
    }
}

pub use peace::services::bancho::*;
