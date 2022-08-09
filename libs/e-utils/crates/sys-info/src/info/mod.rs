mod mem;
mod monitor;
use mem::MemInfo;
use monitor::MonitorInfo;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Info {
    /// Operating system edition.
    pub(crate) edition: Option<String>,
    pub(crate) monitor: MonitorInfo,
    pub(crate) mem: MemInfo,
}
impl Info {
    pub fn new() -> Self {
        Self {
            edition: None,
            monitor: MonitorInfo::new(),
            mem: MemInfo::new(),
        }
    }
}
impl Info {
    pub fn get_monitor(&self) -> MonitorInfo {
        self.monitor.clone()
    }
    pub fn get_mem(&self) -> MemInfo {
        self.mem.clone()
    }
}
