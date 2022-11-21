#[cfg(test)]
mod tests;

mod io;
mod packets;

pub use io::*;
pub use packets::*;

pub mod prelude {
    pub use crate::{build, data, traits::writing::*, PacketId, PacketBuilder};
}
