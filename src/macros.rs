macro_rules! match_id {
    (const $name:ident: $typ:ident = match { exylobby => $exylobby:expr, ytp => $ytp:expr, }) => {
        #[cfg(feature = "exylobby")]
        const $name: $typ = $typ($exylobby);
        #[cfg(feature = "ytp")]
        const $name: $typ = $typ($ytp);
    };
    (pub const $name:ident: $typ:ident = match { exylobby => $exylobby:expr, ytp => $ytp:expr, }) => {
        #[cfg(feature = "exylobby")]
        pub const $name: $typ = $typ($exylobby);
        #[cfg(feature = "ytp")]
        pub const $name: $typ = $typ($ytp);
    };
}
