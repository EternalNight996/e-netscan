#[cfg(feature = "async")]
use crate::async_scan;
#[cfg(feature = "os")]
use crate::os;
#[cfg(feature = "sync")]
use crate::sync_scan;

use crate::{
    frame::{Destination, ScanType},
    interface,
    traceroute::Tracert,
};
use e_utils::sys_utils::dns;
use ipnet::{self, IpNet};
use serde_derive::Deserialize;
use std::{
    any::Any,
    ffi::OsString,
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};
use structopt::{clap::arg_enum, StructOpt};

use super::dns::{DnsResult, DnsResultType, DnsResults};

arg_enum! {
    /// Script
    #[derive(Deserialize, Debug, StructOpt, Clone, PartialEq, Copy)]
    pub enum ScriptsRequired {
        None,
        Default,
        Custom,
    }
}
arg_enum! {
    /// scan type
    #[derive(Deserialize, Debug, StructOpt, Clone, PartialEq, Copy)]
    pub enum ScanModelType {
        Sync,
        Async,
        Os,
        Service,
        Dns,
        Traceroute,
        None
    }
}
arg_enum! {
    /// scan type
    #[derive(Deserialize, Debug, StructOpt, Clone, PartialEq, Copy)]
    pub enum ScanOrderType {
        None,
        Icmp,
        TcpConnect,
        Udp,
        Tcp,
        TcpSyn,
    }
}
/// Opts
#[derive(StructOpt, Debug)]
#[structopt(name = "", setting = structopt::clap::AppSettings::TrailingVarArg)]
#[allow(clippy::struct_excessive_bools)]
pub struct Opts {
    /// host list; example: "192.168.1.1", "192.168.1.0/24", "192.168.8-9.80-100", "baidu.com"
    #[structopt(short, long, use_delimiter = true)]
    pub ips: Vec<String>,

    /// port list; Example: 80,443,8080,100-1000.
    #[structopt(short, long, use_delimiter = true)]
    pub ports: Vec<String>,

    /// use it ip match network interface of hardware;
    #[structopt(long, default_value = "")]
    pub src_ip: String,

    /// The timeout in milliseconds before a port is assumed to be closed; default is 3_600_000ms
    #[structopt(short, long, default_value = "3600000")]
    pub timeout: u64,

    /// Waiting time after packet sending task is completed; default is 3000ms
    #[structopt(long, default_value = "3000")]
    pub wait_time: u64,

    /// Packet sending interval(0 for unlimited); default is 0
    #[structopt(short, long, default_value = "0")]
    pub rate: u64,

    /// send type; [ Icmp, TcpConnect, Udp, Tcp, TcpSyn ]; default: None;  
    #[structopt(short, long, possible_values = &ScanOrderType::variants(), case_insensitive = true, default_value = "none")]
    pub scan: ScanOrderType,

    /// scan type; [ Sync, Async, Os, Service, Dns, Traceroute ]; default: sync
    #[structopt(short, long, possible_values = &ScanModelType::variants(), case_insensitive = true, default_value = "none")]
    pub model: ScanModelType,

    /// scripts
    #[structopt(long, possible_values = &ScriptsRequired::variants(), case_insensitive = true, default_value = "default")]
    pub scripts: ScriptsRequired,

    /// no gui window
    #[structopt(long)]
    pub no_gui: bool,

    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[allow(dead_code)]
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,

    /// Extend command;
    /// example: e-libscanner --ips baidu.com 192.168.1.0/24
    ///         --model sync --scan icmp --no-gui -- -AS
    /// commads:
    /// -- -AS: ARP Spoofing,
    #[structopt(required = false, last = true)]
    pub command: Vec<String>,
}

#[cfg(not(tarpaulin_include))]
impl Opts {
    /// # Example
    /// ```
    /// let mut scanner = Opts::new(Some(&[
    /// "e-libscanner",
    /// "--ips",
    /// "192.168.80.0/21",
    /// "192.168.20-21.15-20",
    /// "baidu.com",
    /// "--model",
    /// "sync",
    /// "--scan",
    /// "Icmp",
    /// "--no-gui",
    /// "--",
    /// "-AS",
    /// ]))
    /// .init()?
    /// .downcast::<sync_scan::Scanner>()
    /// .unwrap();
    /// let rx = scanner.get_progress_receiver();
    /// // Run scan
    /// let handle = thread::spawn(move || scanner.scan(None));
    /// // Print progress
    /// while let Ok(socket_addr) = rx.lock().unwrap().recv() {
    /// println!("Check: {}", socket_addr);
    /// }
    /// let result = handle.join().unwrap();
    /// // Print results
    /// println!("Status: {:?}", result.scan_status);
    /// println!("UP Hosts:");
    /// let len = result.ips.len();
    /// for host in result.ips {
    /// println!("{:?}", host);
    /// }
    /// println!("Scan Time: {:?} count[ {} ]", result.scan_time, len);
    /// ```
    pub fn new<I>(args: Option<I>) -> Result<Self, String>
    where
        Self: Sized,
        I: IntoIterator,
        I::Item: Into<OsString> + Clone,
    {
        match args {
            Some(arg) => match Opts::from_iter_safe(arg) {
                Ok(opt) => Ok(opt),
                Err(e) => Err(String::from(e.to_string())),
            },
            None => Ok(Opts::from_args()),
        }
    }
}

impl Opts {
    /// init opts data
    pub fn init(&self) -> Result<Box<dyn Any>, String> {
        // if not set interface ip, then throgh tcp find interface ip;
        let src_ip = if !self.src_ip.is_empty() {
            match self.src_ip.parse::<IpAddr>() {
                Ok(ip) => ip,
                Err(e) => return Err(String::from(e.to_string())),
            }
        } else {
            interface::get_local_ipaddr()?
        };
        // parse ports
        let ports = parse_str_ports(&self.ports);
        // parse ips
        let ips = parse_ip_range(&self.ips).unwrap_or(vec![]);
        match self.model {
            ScanModelType::Sync => {
                // sync scan
                #[cfg(feature = "sync")]
                {
                    let mut scanner = sync_scan::Scanner::new(src_ip)?;
                    // set methods
                    if self.command.len() > 0 {
                        scanner.set_method(&&self.command)?;
                    }
                    for ip in ips {
                        // add scan target
                        scanner.add_destination(Destination::new(ip, ports.clone()));
                    }
                    // set scan rate
                    scanner.set_send_rate(Duration::from_millis(self.rate));
                    // set timeout
                    scanner.set_timeout(Duration::from_millis(self.timeout));
                    // set wating for time of during
                    scanner.set_wait_time(Duration::from_millis(self.wait_time));
                    // set scan type
                    if let Some(t) = parse_scan_type(&self.scan) {
                        scanner.set_scan_type(t);
                    } else if ports.len() > 0 {
                        scanner.set_scan_type(ScanType::TcpConnectScan);
                    } else {
                        scanner.set_scan_type(ScanType::IcmpPingScan);
                    }
                    Ok(Box::new(scanner))
                }
                #[cfg(not(feature = "sync"))]
                Ok(Box::new(()))
            }
            ScanModelType::Async => {
                // async scan
                #[cfg(feature = "async")]
                {
                    let mut scanner = async_scan::Scanner::new(src_ip)?;
                    for ip in ips {
                        // add scan target
                        scanner.add_destination(Destination::new(ip, ports.clone()));
                    }
                    // set scan rate
                    scanner.set_send_rate(Duration::from_millis(self.rate));
                    // set timeout
                    scanner.set_timeout(Duration::from_millis(self.timeout));
                    // set wating for time of during
                    scanner.set_wait_time(Duration::from_millis(self.wait_time));

                    // set scan type
                    if let Some(t) = parse_scan_type(&self.scan) {
                        scanner.set_scan_type(t);
                    } else if ports.len() > 0 {
                        scanner.set_scan_type(ScanType::TcpConnectScan);
                    } else {
                        scanner.set_scan_type(ScanType::IcmpPingScan);
                    }
                    Ok(Box::new(scanner))
                }
                #[cfg(not(feature = "async"))]
                Ok(Box::new(()))
            }
            ScanModelType::Os => {
                // init OS(osscan guess) data
                #[cfg(feature = "os")]
                {
                    let mut scanner = os::Scanner::new(src_ip)?;
                    // set methods
                    if self.command.len() > 0 {
                        scanner.set_method(&&self.command)?;
                    }
                    // set scan rate
                    scanner.set_send_rate(Duration::from_millis(self.rate));
                    // set timeout
                    scanner.set_timeout(Duration::from_millis(self.timeout));
                    // set wating for time of during
                    scanner.set_wait_time(Duration::from_millis(self.wait_time));
                    // set probe type: default full open
                    scanner.set_full_probe();
                    for ip in ips {
                        let probe_target = os::ProbeTarget {
                            ip_addr: ip,
                            open_tcp_ports: vec![80, 135, 554, 8000, 22],
                            closed_tcp_port: 443,
                            open_udp_port: 123,
                            closed_udp_port: 33455,
                        };
                        // add scan target
                        scanner.add_probe_target(probe_target);
                    }
                    Ok(Box::new(scanner))
                }
                #[cfg(not(feature = "os"))]
                Ok(Box::new(()))
            }
            ScanModelType::Service => {
                // scan service
                #[cfg(feature = "service")]
                {
                    let mut scanner = sync_scan::Scanner::new(src_ip)?;
                    // set scan rate
                    scanner.set_send_rate(Duration::from_millis(self.rate));
                    // set tiemout
                    scanner.set_timeout(Duration::from_millis(self.timeout));
                    // set scan watting for time
                    scanner.set_wait_time(Duration::from_millis(self.wait_time));
                    // set scan type
                    if let Some(t) = parse_scan_type(&self.scan) {
                        scanner.set_scan_type(t);
                    } else {
                        scanner.set_scan_type(ScanType::TcpSynScan);
                    }
                    for ip in ips {
                        // add scan target
                        scanner.add_destination(Destination::new(ip, ports.clone()));
                    }
                    Ok(Box::new(scanner))
                }
                #[cfg(not(feature = "service"))]
                Ok(Box::new(()))
            }
            ScanModelType::Dns => return Ok(Box::new(parse_dns(self.ips.clone()))),
            ScanModelType::Traceroute => {
                return Ok(Box::new(Tracert::new(
                    self.ips.clone(),
                    if self.src_ip.is_empty() {
                        None
                    } else {
                        match self.src_ip.parse::<IpAddr>() {
                            Ok(ip) => Some(ip),
                            Err(_) => None,
                        }
                    },
                )))
            }
            ScanModelType::None => Ok(Box::new(())),
        }
    }
}

/// parse dns and address
fn parse_dns(target: Vec<String>) -> DnsResults {
    target
        .into_iter()
        .map(|src| match src.parse::<IpAddr>() {
            Ok(ip) => match dns::lookup_addr(&ip) {
                Ok(dns_name) => DnsResult {
                    src,
                    result: DnsResultType::Host(dns_name),
                },
                Err(e) => DnsResult {
                    src,
                    result: DnsResultType::Error(e.to_string()),
                },
            },
            Err(_) => match dns::lookup_host(&src) {
                Ok(addr) => DnsResult {
                    src,
                    result: DnsResultType::Addr(addr),
                },
                Err(e) => DnsResult {
                    src,
                    result: DnsResultType::Error(e),
                },
            },
        })
        .collect::<Vec<DnsResult>>()
}

/// parse ip from string list
pub fn parse_ip_range(input: &Vec<String>) -> Result<Vec<IpAddr>, String> {
    let mut ips = vec![];
    for s in input {
        if s.contains('/') {
            // ipv4 parse example: 192.168.8.0/24 -> [192.168.8.1..192.168.8.254];
            // ipv6 parse fd00::/32 -> [fd00::..]
            match s.parse::<IpNet>() {
                Ok(ipnet) => ips.append(&mut ipnet.hosts().collect::<Vec<IpAddr>>()),
                Err(e) => return Err(e.to_string()),
            }
        } else if s.contains('-') {
            // ipv4 parse exmaple: 192.168.8-9.10-20 -> [192.168.8.10..192.168.8.20];
            // ipv6 parse example:
            let ipv4 = s.split('.').collect::<Vec<&str>>();
            if ipv4.len() == 4 {
                let mut range_ip = vec![];
                for ip in ipv4 {
                    let r = ip.split_once('-').unwrap_or((ip, ip));
                    range_ip.push((
                        r.0.parse::<u8>().unwrap_or(0),
                        r.1.parse::<u8>().unwrap_or(0),
                    ));
                }
                for v1 in range_ip[0].0..range_ip[0].1 + 1 {
                    for v2 in range_ip[1].0..range_ip[1].1 + 1 {
                        for v3 in range_ip[2].0..range_ip[2].1 + 1 {
                            for v4 in range_ip[3].0..range_ip[3].1 + 1 {
                                ips.push(IpAddr::V4(Ipv4Addr::new(v1, v2, v3, v4)))
                            }
                        }
                    }
                }
            } else {
                return Err(String::from("cannot parse range[-] of ipv4"));
            }
        } else {
            // ipv4 parse exmaple: baidu.com -> [110.242.68.66, 110.242.68.67]
            // ipv6 parse exmaple:
            match dns::lookup_host(s) {
                Ok(mut addrs) => ips.append(&mut addrs),
                Err(_) => {
                    // ipv4 parse example: 192.168.8.1
                    // ipv6 parse exmaple: fe80::ac47:a2d1:c566:2c6d
                    match s.parse::<IpAddr>() {
                        Ok(ipnet) => ips.push(ipnet),
                        Err(e) => return Err(e.to_string()),
                    }
                }
            }
        }
    }
    Ok(ips)
}

/// parse port from string
fn parse_str_ports(input: &Vec<String>) -> Vec<u16> {
    let mut ports = vec![];
    for s in input {
        if s.contains('-') {
            if let Some(r) = s.split_once('-') {
                if let Ok(start) = r.0.parse::<u16>() {
                    if let Ok(end) = r.1.parse::<u16>() {
                        if start < end {
                            ports.append(&mut (start..end).collect::<Vec<u16>>());
                        }
                    }
                };
            }
        } else {
            if let Ok(port) = s.parse::<u16>() {
                ports.push(port)
            }
        }
    }
    ports
}
/// parse scan type
fn parse_scan_type(scan_type: &ScanOrderType) -> Option<ScanType> {
    match scan_type {
        ScanOrderType::None => None,
        ScanOrderType::Icmp => Some(ScanType::IcmpPingScan),
        ScanOrderType::TcpConnect => Some(ScanType::TcpConnectScan),
        ScanOrderType::Udp => Some(ScanType::UdpPingScan),
        ScanOrderType::Tcp => Some(ScanType::TcpPingScan),
        ScanOrderType::TcpSyn => Some(ScanType::TcpSynScan),
    }
}
