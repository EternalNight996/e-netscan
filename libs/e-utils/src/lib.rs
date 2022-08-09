#[macro_use]
mod cfgs;
cfg_random! {
    pub mod random;
    pub use rand;
}

cfg_std! {
    #[macro_use]
    pub mod p_std;
}

#[cfg(feature = "sys_info")]
#[cfg_attr(docsrs, doc(cfg(feature = "sys_info")))]
pub use sys_info;
#[cfg(feature = "sys_utils")]
#[cfg_attr(docsrs, doc(cfg(feature = "sys_utils")))]
pub use sys_utils;

#[cfg(feature = "log")]
#[cfg_attr(docsrs, doc(cfg(feature = "log")))]
#[path = "./logger.rs"]
pub mod log;

#[cfg(feature = "base64")]
#[cfg_attr(docsrs, doc(cfg(feature = "base64")))]
pub mod base64;
