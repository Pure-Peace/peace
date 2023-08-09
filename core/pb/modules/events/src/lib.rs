#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

mod peace {
    use pb_base as base;

    pub mod services {
        pub mod events {
            include!("../../../generated/peace.services.events.rs");

            pub const EVENTS_DESCRIPTOR_SET: &[u8] = include_bytes!(
                "../../../generated/peace.services.events.descriptor.bin"
            );
        }
    }
}

pub use peace::services::events::*;
