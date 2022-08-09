mod scanner;
mod receiver;

#[cfg(not(target_os="windows"))]
mod unix;
#[cfg(not(target_os="windows"))]
use unix::*;

#[cfg(target_os="windows")]
#[path = "./windows.rs"]
mod win;
#[cfg(target_os="windows")]
use win::*;

pub use scanner::*;
