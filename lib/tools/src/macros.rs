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
macro_rules! main_startup_info {
    ($($banner: expr)?) => {
        println!("{}", $crate::constants::PEACE_BANNER);
        $(println!("\n{}", $banner))?;
        println!(
            "\n>> PEACE PROJECT @ 2023\n>> Running {} v{}\n  | authors: {}\n  | git repository: {}\n  | docs website: {}\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_AUTHORS"),
            env!("CARGO_PKG_REPOSITORY"),
            env!("CARGO_PKG_HOMEPAGE")
        );
    };
}

#[macro_export]
macro_rules! framework_info {
    ($($banner: expr)?) => {
        $(println!("\n{}", $banner))?;
        println!(
            ">> Runs on framework {} v{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        );
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

#[macro_export]
/// Lazy init or handle
///
/// Examples:
/// ```rust,ignore
/// let mut messages = None::<Vec<T>>;
/// lazy_init!(messages => messages.push(msg.content.clone()), vec![msg.content.clone()]);
///
/// // equals to:
/// match messages {
///     Some(ref mut messages) => messages.push(msg.content.clone()),
///     None => messages = Some(vec![msg.content.clone()]),
/// }
/// ```
macro_rules! lazy_init {
    ($option: ident => $handle: expr, $init: expr) => {
        match $option {
            Some(ref mut $option) => {
                $handle;
            },
            None => $option = Some($init),
        };
    };
    ($option:ident, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? => $handle: expr, $init: expr) => {
        match $option {
            $( $pattern )|+ $( if $guard )? => {
                $handle;
            },
            None => $option = Some($init),
        };
    };
}
