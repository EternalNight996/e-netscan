//! Log宏打印
//! # Example
//! ```no_run
//! fn main() {
//!     info!(12345; 2;3; 4;); 打印 -> [Console] 2021-12-15T14:50:26.443790100+08:00 - INFO -server_1379::... - [>] 😵 12345234 🐢⌛
//!     debug!(pure "{}123{}", "pudge", 321); 打印 -> Console] 2021-12-15T14:50:26.444484500+08:00 - DEBUG -server_1379::...- pudge123321
//!     error!("error code"); 打印 -> [Console] 2021-12-15T14:50:26.601590100+08:00 - ERROR -server_1379::service::db - [>] 😵 error code 🐢⌛
//!     error!(); 抛出异常 -> internal error: entered unreachable code
//! }
//! ```
//! 
//! 
#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! p_log {
    () => (panic!("internal error: entered unreachable code"));
    (pure $target:ident($fmt:expr)) => (log::$target!("{}", $fmt));
    (pure $target:ident($($arg:tt)*)) => {{log::$target!($($arg)*)}};
    ($target:ident($fmt:expr)) => (log::$target!("{}", $crate::rgb_format!($fmt)));
    ($target:ident($($arg:tt)*)) => {{log::$target!("{}", $crate::rgb_format!($($arg)*))}};
}
#[macro_export]
#[doc(hidden)]
macro_rules! debug {
    () => ($crate::p_log!());
    (pure $fmt:expr) => ($crate::p_log!(pure debug($fmt)));
    (pure $($arg:tt)*) => {{$crate::p_log!(pure debug($($arg)*))}};
    ($fmt:expr) => ($crate::p_log!(debug($fmt)));
    ($($args:tt);*) => {{$crate::p_log!(debug(concat!($($args),*)))}};
    ($($arg:tt)*) => {{$crate::p_log!(debug($($arg)*))}};
}
#[macro_export]
#[doc(hidden)]
macro_rules! info {
    () => ($crate::p_log!());
    (pure $fmt:expr) => ($crate::p_log!(pure info($fmt)));
    (pure $($arg:tt)*) => {{$crate::p_log!(pure info($($arg)*))}};
    ($fmt:expr) => ($crate::p_log!(info($fmt)));
    ($($args:tt);*) => {{$crate::p_log!(info(concat!($($args),*)))}};
    ($($arg:tt)*) => {{$crate::p_log!(info($($arg)*))}};
}
#[macro_export]
#[doc(hidden)]
macro_rules! error {
    () => ($crate::p_log!());
    (pure $fmt:expr) => ($crate::p_log!(pure error($fmt)));
    (pure $($arg:tt)*) => {{$crate::p_log!(pure error($($arg)*))}};
    ($fmt:expr) => ($crate::p_log!(error($fmt)));
    ($($args:tt);*) => {{$crate::p_log!(error(concat!($($args),*)))}};
    ($($arg:tt)*) => {{$crate::p_log!(error($($arg)*))}};
}