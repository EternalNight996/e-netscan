use std::collections::{HashMap, HashSet};
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

/// Status of scan task
#[derive(Clone, Debug, PartialEq)]
pub enum ScanStatus {
    Ready,
    Done,
    Timeout,
    Error,
}

/// Status of the scanned port
#[derive(Clone, Copy, Debug)]
pub enum PortStatus {
    Open,
    Closed,
    Filtered,
}
impl fmt::Display for PortStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PortStatus::Open => "Open",
                PortStatus::Closed => "Closed",
                PortStatus::Filtered => "Filtered",
            }
        )
    }
}
/// Information about the scanned host
#[derive(Clone, Copy, Debug)]
pub struct HostInfo {
    /// IP address of the host
    pub ip_addr: IpAddr,
    /// IP Time to Live (Hop Limit)
    pub ttl: u8,
}
impl fmt::Display for HostInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ip {} ttl {}] ", self.ip_addr, self.ttl)
    }
}

/// Information about the scanned port
#[derive(Clone, Debug)]
pub struct PortInfo {
    /// Port number
    pub port: u16,
    /// Port status
    pub status: PortStatus,
    // Port describe
    pub describe: String,
}
impl fmt::Display for PortInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} {} {}] ", self.port, self.status, self.describe)
    }
}

/// Result of port scan
#[derive(Clone, Debug)]
pub struct ScanResult {
    pub ips: Vec<HostInfo>,
    /// HashMap of scanned IP addresses and their respective port scan results.
    pub ip_with_port: HashMap<IpAddr, Vec<PortInfo>>,
    /// Time taken to scan
    pub scan_time: Duration,
    /// Status of the scan task
    pub scan_status: ScanStatus,
}

impl ScanResult {
    /// 0x1 host scan 0x2 port scan
    pub fn new() -> ScanResult {
        ScanResult {
            ips: vec![],
            ip_with_port: HashMap::new(),
            scan_time: Duration::from_millis(0),
            scan_status: ScanStatus::Ready,
        }
    }
    /// Returns IP addresses from the scan result
    pub fn get_hosts(&self) -> Vec<IpAddr> {
        self.ips
            .iter()
            .map(|info| info.ip_addr)
            .collect::<Vec<IpAddr>>()
    }
    /// Get open ports of the specified IP address from the scan results
    pub fn get_open_ports(&self, ip_addr: IpAddr) -> Vec<u16> {
        let mut open_ports: Vec<u16> = vec![];
        if let Some(ports) = self.ip_with_port.get(&ip_addr) {
            for port_info in ports {
                match port_info.status {
                    PortStatus::Open => {
                        open_ports.push(port_info.port);
                    }
                    _ => {}
                }
            }
        }
        open_ports
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ScanResults {
    pub result: ScanResult,
    pub ip_set: HashSet<IpAddr>,
    pub socket_set: HashSet<SocketAddr>,
}

impl ScanResults {
    /// 0x1 host scan 0x2 port scan
    pub fn new() -> ScanResults {
        ScanResults {
            result: ScanResult::new(),
            ip_set: HashSet::new(),
            socket_set: HashSet::new(),
        }
    }
}
