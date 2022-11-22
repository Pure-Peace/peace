#[macro_export]
macro_rules! pkg_metadata {
    () => {
        {
            const NAME: &str = env!("CARGO_PKG_NAME");
            const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
            const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
            const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");
            const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
            const VERSION: &str = env!("CARGO_PKG_VERSION");
            format!(
                ">> PACKAGE METADATA <<\n\n  - NAME: {}\n  - AUTHORS: {}\n  - DESCRIPTION: {}\n  - HOMEPAGE: {}\n  - REPOSITORY: {}\n  - VERSION: v{}",
                NAME,
                AUTHORS,
                DESCRIPTION,
                HOMEPAGE,
                REPOSITORY,
                VERSION
            )
        }
    };
}

#[macro_export]
macro_rules! listening {
    ($addr: expr) => {
        format!("\n\n{}\n\n>> listening on {}", $crate::pkg_metadata!(), $addr)
    };
}
