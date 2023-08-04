#![allow(dead_code)]
#![allow(non_snake_case)]

mod peace {
    use pb_base as base;

    pub mod services {
        pub mod logs {
            include!("../../../generated/peace.frame.logs.rs");

            pub const LOGS_DESCRIPTOR_SET: &[u8] = include_bytes!(
                "../../../generated/peace.frame.logs.descriptor.bin"
            );
        }
    }
}

pub use peace::services::logs::*;
