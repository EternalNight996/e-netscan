use std::io::Error;
/// 内存信息
#[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemInfo {
    /// Total physical memory.
    pub total: u64,
    pub free: u64,
    pub avail: u64,
    pub buffers: u64,
    pub cached: u64,
    /// Total swap memory.
    pub swap_total: u64,
    pub swap_free: u64,
}
impl MemInfo {
    pub fn new() -> Self {
        get_mem().unwrap_or(MemInfo::default())
    }
}
extern "C" {
    #[cfg(target_os = "windows")]
    fn get_mem_info() -> MemInfo;
}

/// 获取内存信息
pub fn get_mem() -> Result<MemInfo, Error> {
    #[cfg(target_os = "windows")]
    {
        Ok(unsafe { get_mem_info() })
    }
}
