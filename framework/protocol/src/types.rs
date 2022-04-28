use parking_lot::{RwLockReadGuard, RwLockWriteGuard};

pub type Readable<T> = RwLockReadGuard<'static, T>;
pub type Writeable<T> = RwLockWriteGuard<'static, T>;
