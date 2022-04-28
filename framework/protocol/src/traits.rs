use parking_lot::RwLock;

use crate::types::{Readable, Writeable};

pub trait LockedSingleton<T> {
    fn get() -> &'static RwLock<T>;

    fn write() -> Writeable<T> {
        Self::get().write()
    }

    fn read() -> Readable<T> {
        Self::get().read()
    }
}
