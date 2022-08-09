/// The `cfg_debug`
macro_rules! cfg_std {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "std")]
            #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
            $item
        )*
    }
}
/// The `cfg_random`
macro_rules! cfg_random {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "random")]
            #[cfg_attr(docsrs, doc(cfg(feature = "random")))]
            $item
        )*
    }
}
