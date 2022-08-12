use e_libscanner::dns::DnsResult;
use e_libscanner::frame::result::{HostInfo, PortInfo};
use e_libscanner::interface;
use e_libscanner::os::ProbeResult;
use e_libscanner::service::ScanServiceResult;
use e_libscanner::traceroute::TracertQueryResult;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::sleep;
use std::time::Duration;

// share static datas
pub(crate) static DATAS: Lazy<Datas> = Lazy::new(|| Datas::default());

#[derive(Debug, Clone)]
pub(crate) enum ScanResultType {
    Host(Vec<HostInfo>),
    Port(HashMap<IpAddr, Vec<PortInfo>>),
    Os(Vec<ProbeResult>),
    Service(Vec<ScanServiceResult>),
    Dns(Vec<DnsResult>),
    Tracert(Vec<TracertQueryResult>),
    None(Option<String>),
}
impl ScanResultType {
    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        match self {
            ScanResultType::Host(d) => d.len(),
            ScanResultType::Port(d) => d.len(),
            ScanResultType::Os(d) => d.len(),
            ScanResultType::Service(d) => d.len(),
            ScanResultType::Dns(d) => d.len(),
            ScanResultType::Tracert(d) => d.len(),
            ScanResultType::None(_) => 0,
        }
    }
}
impl Default for ScanResultType {
    fn default() -> Self {
        ScanResultType::None(None)
    }
}
pub(crate) type ScanHistory = Vec<(ScanResultType, i64)>;

#[derive(Default, Debug)]
pub(crate) struct Datas {
    pub(crate) scan: ScanData,
    pub(crate) sysinfo: SystemInfoData,
}
#[derive(Debug)]
pub(crate) struct ScanData {
    pub(crate) results: Arc<Mutex<ScanResultType>>,
    pub(crate) history: Arc<RwLock<ScanHistory>>,
    pub(crate) locked: Arc<Mutex<bool>>,
    /// timeout /ms
    pub(crate) timeout: u64,
    /// timeout count
    pub(crate) timeout_count: u64,
    /// stop tasks
    pub(crate) stop: Arc<Mutex<bool>>,
    /// finished time /ms
    pub(crate) finished_time: Arc<RwLock<f64>>,
}

impl Default for ScanData {
    fn default() -> Self {
        ScanData {
            results: Arc::new(Mutex::new(ScanResultType::default())),
            history: Arc::new(RwLock::new(ScanHistory::default())),
            locked: Arc::new(Mutex::new(false)),
            timeout: 300,
            timeout_count: 20,
            stop: Arc::new(Mutex::new(false)),
            finished_time: Arc::new(RwLock::new(0.0)),
        }
    }
}
impl ScanData {
    /// result to vec
    pub(crate) fn result_to_vec(&self) -> Result<Vec<String>, String> {
        match self.results.lock() {
            Ok(r) => Ok(match &*r {
                ScanResultType::Host(data) => {
                    data.iter().map(|x| x.to_string()).collect::<Vec<String>>()
                }
                ScanResultType::Port(data) => data
                    .iter()
                    .map(|x| {
                        format!(
                            "{}: [ {} ]",
                            x.0.to_string(),
                            x.1.iter().map(|x| x.to_string()).collect::<String>()
                        )
                    })
                    .collect::<Vec<String>>(),
                ScanResultType::Os(data) => {
                    data.iter().map(|x| x.display()).collect::<Vec<String>>()
                }
                ScanResultType::Service(data) => {
                    data.iter().map(|x| x.to_string()).collect::<Vec<String>>()
                }
                ScanResultType::Dns(data) => {
                    data.iter().map(|x| x.to_string()).collect::<Vec<String>>()
                }
                ScanResultType::Tracert(data) => {
                    data.iter().map(|x| x.to_string()).collect::<Vec<String>>()
                }
                ScanResultType::None(data) => match data {
                    Some(d) => vec![d.clone()],
                    None => return Err("None".to_owned()),
                },
            }),
            Err(e) => Err(e.to_string()),
        }
    }
    /// result to string
    pub(crate) fn result_to_string(&self) -> Result<String, String> {
        match self.results.lock() {
            Ok(r) => Ok(match &*r {
                ScanResultType::Host(data) => {
                    data.iter().map(|x| x.to_string()).collect::<String>()
                }
                ScanResultType::Port(data) => data
                    .iter()
                    .map(|x| {
                        format!(
                            "{}: [{}]\n",
                            x.0,
                            x.1.iter().map(|xx| xx.to_string()).collect::<String>()
                        )
                    })
                    .collect::<String>(),
                ScanResultType::Os(data) => data.iter().map(|x| x.to_string()).collect::<String>(),
                ScanResultType::Service(data) => {
                    data.iter().map(|x| x.to_string()).collect::<String>()
                }
                ScanResultType::Dns(data) => data.iter().map(|x| x.to_string()).collect::<String>(),
                ScanResultType::Tracert(data) => {
                    data.iter().map(|x| x.to_string()).collect::<String>()
                }
                ScanResultType::None(data) => match data {
                    Some(d) => d.clone(),
                    None => return Err(String::from("Scan result of type is None.")),
                },
            }),
            Err(e) => Err(e.to_string()),
        }
    }
    /// watting for scan of end
    pub(crate) fn waiting_for_lock(&self) {
        let mut n = 0;
        loop {
            if !*self.locked.lock().unwrap() || n > self.timeout_count {
                break;
            } else {
                sleep(Duration::from_millis(self.timeout));
                n += 1;
            }
        }
    }
    /// stop tasks
    pub(crate) fn set_stop(&self, value: bool) {
        *self.stop.lock().unwrap() = value;
    }
    /// set memory safety lock 
    pub(crate) fn set_locked(&self, value: bool) {
        *self.locked.lock().unwrap() = value;
    }
    /// set finished time
    pub(crate) fn set_finished_time(&self, value: f64) {
        *self.finished_time.write().unwrap() = value;
    }
    /// get finished time
    pub(crate) fn get_finished_time(&self) -> f64 {
        *self.finished_time.read().unwrap()
    }
    /// get history length
    pub(crate) fn get_history_len(&self) -> usize {
        if let Ok(r) = self.history.read() {
            r.len()
        } else {
            0
        }
    }
    /// get result length
    pub(crate) fn get_result_len(&self) -> usize {
        match self.results.lock() {
            Ok(r) => match &*r {
                ScanResultType::Host(data) => data.len(),
                ScanResultType::Port(data) => data.len(),
                ScanResultType::Os(data) => data.len(),
                ScanResultType::Service(data) => data.len(),
                ScanResultType::Dns(data) => data.len(),
                ScanResultType::Tracert(data) => data.len(),
                ScanResultType::None(data) => {
                    if data.is_some() {
                        1
                    } else {
                        0
                    }
                }
            },
            Err(_) => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SystemInfoData {
    pub(crate) iface: InterfaceData,
}
impl Default for SystemInfoData {
    fn default() -> Self {
        Self {
            iface: InterfaceData::new(),
        }
    }
}
/// network interface infomation
#[derive(Debug, Default, Clone)]
pub(crate) struct InterfaceData {
    #[allow(unused)]
    pub(crate) index: Option<u32>,
    pub(crate) local_addr: Option<IpAddr>,
    pub(crate) gateway: Option<interface::Gateway>,
}
impl InterfaceData {
    fn new() -> Self {
        let iface = get_interface();
        log::debug!("interface {:?}", iface);
        iface
    }
}

/// get network interface infomation
#[cfg(target_os = "windows")]
fn get_interface() -> InterfaceData {
    let local_addr = interface::get_local_ipaddr().unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)));
    let index = interface::get_interface_index_by_ip(local_addr);
    let gateway = interface::get_default_gateway()
        .and_then(|x| Ok(Some(x)))
        .unwrap_or(None);
    InterfaceData {
        index,
        local_addr: Some(local_addr),
        gateway,
    }
}
#[cfg(any(target_os = "linux", target_os = "unix"))]
fn get_interface() -> InterfaceData {
    InterfaceData::default()
}
#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "unix")))]
fn get_interface() -> InterfaceData {
    InterfaceData::default()
}
