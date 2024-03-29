macro_rules! match_env {
    (const $name:ident: &str = match { prod => $prod:expr, test => $test:expr, }) => {
        #[cfg(feature = "prod")]
        const $name: &str = $prod;
        #[cfg(feature = "test")]
        const $name: &str = $test;
    };
    (const $name:ident: $typ:ident = match { prod => $prod:expr, test => $test:expr, }) => {
        #[cfg(feature = "prod")]
        #[allow(clippy::unreadable_literal)]
        const $name: $typ = $typ($prod);
        #[cfg(feature = "test")]
        #[allow(clippy::unreadable_literal)]
        const $name: $typ = $typ($test);
    };
    (pub const $name:ident: $typ:ident = match { prod => $prod:expr, test => $test:expr, }) => {
        #[cfg(feature = "prod")]
        #[allow(clippy::unreadable_literal)]
        pub const $name: $typ = $typ($prod);
        #[cfg(feature = "test")]
        #[allow(clippy::unreadable_literal)]
        pub const $name: $typ = $typ($test);
    };
}

macro_rules! handle {
    ($event:literal for $ctx:ident, $arg:expr => {$($name:literal => $func:expr),*$(,)?}) => {
        let inner = || -> crate::error::AVoid {
            $(
            $func(&$ctx, &$arg).context($name)?;
            )*
            Ok(())
        };
        log_handler_err(
            &$ctx,
            inner().context(format!("`{}`", $event)),
        );
    };
}
