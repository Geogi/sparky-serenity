macro_rules! match_guild {
    (const $name:ident: &str = match { exylobby => $exylobby:expr, ytp => $ytp:expr, }) => {
        #[cfg(feature = "exylobby")]
        const $name: &str = $exylobby;
        #[cfg(feature = "ytp")]
        const $name: &str = $ytp;
    };
    (const $name:ident: $typ:ident = match { exylobby => $exylobby:expr, ytp => $ytp:expr, }) => {
        #[cfg(feature = "exylobby")]
        #[allow(clippy::unreadable_literal)]
        const $name: $typ = $typ($exylobby);
        #[cfg(feature = "ytp")]
        #[allow(clippy::unreadable_literal)]
        const $name: $typ = $typ($ytp);
    };
    (pub const $name:ident: $typ:ident = match { exylobby => $exylobby:expr, ytp => $ytp:expr, }) => {
        #[cfg(feature = "exylobby")]
        #[allow(clippy::unreadable_literal)]
        pub const $name: $typ = $typ($exylobby);
        #[cfg(feature = "ytp")]
        #[allow(clippy::unreadable_literal)]
        pub const $name: $typ = $typ($ytp);
    };
}

macro_rules! handle {
    ($event:literal for $ctx:ident, $arg:expr => {$($name:literal => $func:expr),*,}
     in $chan:expr) => {
        let inner = || -> $crate::error::AVoid {
            $(
            $func(&$ctx, &$arg).context($name)?;
            )*
            Ok(())
        };
        log_handler_err(
            &$ctx,
            $chan,
            inner().context(format!("`{}`", $event)),
        );
    };
}
