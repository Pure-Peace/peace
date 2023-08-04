#![allow(dead_code)]
#![allow(non_snake_case)]

mod peace {
    pub mod base {
        include!("../../../generated/peace.base.rs");

        pub const LOGS_DESCRIPTOR_SET: &[u8] =
            include_bytes!("../../../generated/peace.base.descriptor.bin");
    }
}

pub use peace::base::*;
