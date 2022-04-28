#[macro_export]
macro_rules! impl_locked_singleton {
    ($t:ty => $obj:ty) => {
        use parking_lot::RwLock;
        impl crate::traits::LockedSingleton<$t> for $obj {
            fn get() -> &'static RwLock<$t> {
                static INS: Lazy<RwLock<$t>> = Lazy::new(|| RwLock::new(<$t>::new()));
                &INS
            }
        }
    };
}
