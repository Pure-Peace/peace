#[macro_export]
macro_rules! get_space {
    () => {
        Space::get()
    };
}

#[macro_export]
macro_rules! write_space {
    () => {
        Space::write().await
    };
}

#[macro_export]
macro_rules! read_space {
    () => {
        Space::read().await
    };
}
