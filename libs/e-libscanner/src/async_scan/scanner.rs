use super::scan_target;
use crate::data::id::{DEFAULT_HOSTS_CONCURRENCY, DEFAULT_PORTS_CONCURRENCY, DEFAULT_SRC_PORT};
use crate::frame::result::ScanResult;
use crate::frame::{result::ScanStatus, Destination, ScanSetting, ScanType};
use crate::interface;
use std::collections::HashSet;
use std::net::{IpAddr, SocketAddr};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Host Scanner
#[derive(Clone, Debug)]
pub struct Scanner {
    /// Index of network interface
    pub if_index: u32,
    /// Name of network interface
    pub if_name: String,
    /// MAC address of network interface
    pub src_mac: [u8; 6],
    /// MAC address of default gateway(or scan target host)
    pub dst_mac: [u8; 6],
    /// Source IP address
    pub src_ip: IpAddr,
    /// Source port
    pub src_port: u16,
    /// Destinations
    pub destinations: Vec<Destination>,
    /// Scan Type
    pub scan_type: ScanType,
    /// Timeout setting for entire scan task
    pub timeout: Duration,
    /// Waiting time after packet sending task is completed
    pub wait_time: Duration,
    /// Packet sending interval(0 for unlimited)
    pub send_rate: Duration,
    /// Scan Result
    pub scan_result: ScanResult,
    /// Sender for progress messaging
    pub tx: Arc<Mutex<Sender<SocketAddr>>>,
    /// Receiver for progress messaging
    pub rx: Arc<Mutex<Receiver<SocketAddr>>>,
}

impl Scanner {
    /// Create new HostScanner with source IP address
    /// Initialized with default value based on the specified IP address
    pub fn new(src_ip: IpAddr) -> Result<Scanner, String> {
        let mut if_index: u32 = 0;
        let mut if_name: String = String::new();
        let mut src_mac: pnet_datalink::MacAddr = pnet_datalink::MacAddr::zero();
        for iface in pnet_datalink::interfaces() {
            for ip in iface.ips {
                if ip.ip() == src_ip {
                    if_index = iface.index;
                    if_name = iface.name;
                    src_mac = iface.mac.unwrap_or(pnet_datalink::MacAddr::zero());
                    break;
                }
            }
        }
        if if_index == 0 || if_name.is_empty() || src_mac == pnet_datalink::MacAddr::zero() {
            return Err(String::from(
                "Failed to create Scanner. Network Interface not found.",
            ));
        }
        let (tx, rx) = channel();
        let scanner = Scanner {
            if_index,
            if_name,
            src_mac: src_mac.octets(),
            dst_mac: interface::get_default_gateway_macaddr(),
            src_ip,
            src_port: DEFAULT_SRC_PORT,
            destinations: vec![],
            scan_type: ScanType::IcmpPingScan,
            timeout: Duration::from_millis(300_000),
            wait_time: Duration::from_millis(200),
            send_rate: Duration::from_millis(0),
            scan_result: ScanResult::new(),
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
        };
        Ok(scanner)
    }
    /// get scan count
    pub fn len(&self) -> usize {
        let mut len = 0;
        for dst in self.destinations.iter() {
            if dst.dst_ports.len() > 0 {
                len += dst.dst_ports.len();
            } else {
                len += 1;
            }
        }
        len
    }
    /// Set source IP address
    pub fn set_src_ip(&mut self, src_ip: IpAddr) {
        self.src_ip = src_ip;
    }
    /// Get source IP address
    pub fn get_src_ip(&self) -> IpAddr {
        self.src_ip.clone()
    }
    /// Add Destination
    pub fn add_destination(&mut self, dst: Destination) {
        self.destinations.push(dst);
    }
    /// Set Destinations
    pub fn set_destinations(&mut self, dst: Vec<Destination>) {
        self.destinations = dst;
    }
    /// Get Destinations
    pub fn get_destinations(&self) -> Vec<Destination> {
        self.destinations.clone()
    }
    /// Set ScanType
    pub fn set_scan_type(&mut self, scan_type: ScanType) {
        self.scan_type = scan_type;
    }
    /// Get ScanType
    pub fn get_scan_type(&self) -> ScanType {
        self.scan_type.clone()
    }
    /// Set timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
    /// Get timeout
    pub fn get_timeout(&self) -> Duration {
        self.timeout.clone()
    }
    /// Set wait time
    pub fn set_wait_time(&mut self, wait_time: Duration) {
        self.wait_time = wait_time;
    }
    /// Get wait time
    pub fn get_wait_time(&self) -> Duration {
        self.wait_time.clone()
    }
    /// Set send rate
    pub fn set_send_rate(&mut self, send_rate: Duration) {
        self.send_rate = send_rate;
    }
    /// Get send rate
    pub fn get_send_rate(&self) -> Duration {
        self.send_rate.clone()
    }
    /// Get scan result
    pub fn get_scan_result(&self) -> ScanResult {
        self.scan_result.clone()
    }
    /// Get progress receiver
    pub fn get_progress_receiver(&self) -> Arc<Mutex<Receiver<SocketAddr>>> {
        self.rx.clone()
    }
    /// Run Scan
    pub async fn run_scan(&mut self, pstop: Option<Arc<Mutex<bool>>>) {
        let mut ip_set: HashSet<IpAddr> = HashSet::new();
        for dst in self.destinations.clone() {
            ip_set.insert(dst.dst_ip);
        }
        let scan_setting: ScanSetting = ScanSetting {
            if_index: self.if_index.clone(),
            src_mac: pnet_datalink::MacAddr::from(self.src_mac),
            dst_mac: pnet_datalink::MacAddr::from(self.dst_mac),
            src_ip: self.src_ip.clone(),
            src_port: self.src_port.clone(),
            destinations: self.destinations.clone(),
            ip_set,
            timeout: self.timeout.clone(),
            wait_time: self.wait_time.clone(),
            send_rate: self.send_rate.clone(),
            scan_type: self.scan_type.clone(),
            hosts_concurrency: DEFAULT_HOSTS_CONCURRENCY,
            ports_concurrency: DEFAULT_PORTS_CONCURRENCY,
        };
        let start_time = Instant::now();
        let mut result: ScanResult = scan_target(scan_setting, &self.tx, pstop).await;
        result.scan_time = Instant::now().duration_since(start_time);
        if result.scan_time > self.timeout {
            result.scan_status = ScanStatus::Timeout;
        } else {
            result.scan_status = ScanStatus::Done;
        }
        self.scan_result = result;
    }
    /// Run Sync scan and return result
    pub async fn scan(&mut self, pstop: Option<Arc<Mutex<bool>>>) -> ScanResult {
        self.run_scan(pstop).await;
        self.scan_result.clone()
    }
}
