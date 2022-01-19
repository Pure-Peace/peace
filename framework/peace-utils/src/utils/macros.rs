#[macro_export(local_inner_macros)]
macro_rules! write_lock {
    ($lock: expr) => {
        $lock.write().await
    };
}

#[macro_export(local_inner_macros)]
macro_rules! read_lock {
    ($lock: expr) => {
        $lock.read().await
    };
}
