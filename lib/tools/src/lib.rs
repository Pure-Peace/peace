#[cfg(feature = "async_collections")]
pub mod async_collections;

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

#[macro_export]
/// Display code execution time
///
/// Examples:
/// ```rust,ignore
/// tools::time_spent!(
///     @log=info;
///     "hello".to_string();
///     println!("hello")
/// );
///
/// // or
/// tools::time_spent!(
///     @log=println;
///     println!("hello")
/// );
///
/// // or
/// tools::time_spent!(
///     @log=info;
///     "this is a tag" => println!("hello"),
///     "this is a tag1" => { let s = "".to_string(); }
/// );
///
/// // or
/// tools::time_spent!(
///     @log=info;
///     "tag for each" => @each(
///         println!("hello"),
///         println!("hello1")
///     )
/// );
///
/// ```
macro_rules! time_spent {
    (@log=$printer: tt; $($code: expr$(,)*)*) => {
        $(
            $crate::time_spent!(@log=$printer; stringify!($code) => $code);
        )*
    };
    (@log=$printer: tt; $tag: expr => @each($($code: expr$(,)*)*)) => {
        $(
            $crate::time_spent!(@log=$printer; $tag => $code);
        )*
    };
    (@log=$printer: tt; $($tag: expr => $code: expr$(,)*)*) => {
        $(
            let __time_start = std::time::Instant::now();
            $code;
            let __time_end = __time_start.elapsed();
            $printer!(concat!("\"", $tag, "\" in {:?}"), __time_end);
        )*
    };
}
