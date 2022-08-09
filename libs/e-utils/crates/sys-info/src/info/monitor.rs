use winapi::um::winuser::{
    GetSystemMetrics, SM_CXFULLSCREEN, SM_CXSCREEN, SM_CXVIRTUALSCREEN, SM_CYFULLSCREEN,
    SM_CYSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

// #[repr(C)]
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MonitorInfo {
    pub xscreen: i32,
    pub yscreen: i32,
    pub cy_fullscreen: i32,
    pub cx_fullscreen: i32,
    pub cxvirtual_screen: i32,
    pub cyvirtual_screen: i32,
    pub xvirtual_screen: i32,
    pub yvirtual_screen: i32,
}

impl MonitorInfo {
    pub fn new() -> Self {
        Self {
            xscreen: unsafe { GetSystemMetrics(SM_CXSCREEN) },
            yscreen: unsafe { GetSystemMetrics(SM_CYSCREEN) },
            cx_fullscreen: unsafe { GetSystemMetrics(SM_CXFULLSCREEN) },
            cy_fullscreen: unsafe { GetSystemMetrics(SM_CYFULLSCREEN) },
            cxvirtual_screen: unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) },
            cyvirtual_screen: unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) },
            xvirtual_screen: unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) },
            yvirtual_screen: unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) },
        }
    }
}

// impl MonitorInfo {
//     pub fn new() -> Self {
//         get_monitor().unwrap_or(MonitorInfo::default())
//     }
// }
// extern "C" {
//     #[cfg(target_os = "windows")]
//     fn get_monitor_info() -> MonitorInfo;
// }

// /// 获取内存信息
// pub fn get_monitor() -> Result<MonitorInfo, std::io::Error> {
//     #[cfg(target_os = "windows")]
//     {
//         Ok(unsafe { get_monitor_info() })
//     }
// }
