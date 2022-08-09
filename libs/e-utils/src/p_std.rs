//! utils的标准库
//! 😵 hello tuils 🐢
//!

/// # Example
/// ``` no_run
/// fn main() {
/// output!("hello world");
/// 输出 [>] 😵 hello world 🐢
///
/// output!(1;2;34; 5);
/// 输出 [>] 😵 hello world 🐢
///
/// let list = [1,2,34,5];
/// 输出 [>] 😵 12345 🐢
///
/// output!("{:#?}",list);
/// 输出 [>] 😵 [
/// 1,
/// 2,
/// 34,
/// 5,
/// ] 🐢
/// }
/// 
/// output!(rgb[Some((0,255,0)), None] "打印自动义RGB");
/// 打印自动义RGB
/// 
/// ```
#[macro_export]
#[doc(hidden)]
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
macro_rules! output {
    () => {print!("\n")};
    (rgb[$rgb_f:expr, $rgb_b:expr] $fmt:expr) => {::std::eprintln!("{}", $crate::rgb_format!(rgb[$rgb_f, $rgb_b] $fmt))};
    (pure $fmt:expr) => {$crate::rgb_format!(pure $fmt)};
    (pure $($arg:tt)*) => {{$crate::rgb_format!(pure $($arg)*)}};
    ($fmt:expr) => {::std::eprintln!("{}", $crate::rgb_format!($fmt))};
    ($($args:tt);*) => {{::std::eprintln!("{}", $crate::rgb_format!(::std::concat!($($args),*)))}};
    ($($args:tt)*) => {{::std::eprintln!("{}", $crate::rgb_format!($($args)*))}};
} 
