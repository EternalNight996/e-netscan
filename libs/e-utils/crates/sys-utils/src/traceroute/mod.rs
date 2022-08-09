extern crate pnet;
pub mod interface;
pub(crate) mod packet_builder;

use async_std::task::block_on;
use pnet::datalink::{self, channel};
use pnet::datalink::{MacAddr, NetworkInterface};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::icmp::{IcmpPacket, IcmpTypes};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::Packet;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;
use std::time::Duration;

use crate::dns;

use self::interface::{get_default_gateway, get_local_ipaddr};

/// Traceroute instance containing destination address and configurations
pub struct Traceroute {
    src_target: String,
    target: IpAddr,
    config: Config,
    done: bool,
}
impl Iterator for Traceroute {
    type Item = TracerouteHop;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done
            || self
                .config
                .channel
                .max_hops_reached(self.config.max_hops as u8)
        {
            return None;
        }

        let hop = self.calculate_next_hop();
        self.done = hop
            .query_result
            .iter()
            .filter(|ip| ip.addr.contains(&self.target.to_string()))
            .next()
            .is_some();
        Some(hop)
    }
}

impl Traceroute {
    /// Creates new instance of Traceroute
    pub fn new(target: String, iface_ip: Option<IpAddr>) -> Result<Self, String> {
        let destination_ip = target.parse::<IpAddr>().unwrap_or_else(|_x| {
            dns::lookup_host(&target).unwrap_or(vec![target
                .parse::<IpAddr>()
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))])[0]
        });
        let src_ip = if let Some(ip) = iface_ip {
            ip
        } else {
            get_local_ipaddr()?
        };
        if let Some(iface) = datalink::interfaces()
            .into_iter()
            .find(|x| x.ips.iter().find(|xx| xx.ip() == src_ip).is_some())
        {
            Ok(Traceroute {
                src_target: target,
                target: destination_ip,
                config: Config::default()
                    .with_port(33480)
                    .with_max_hops(30)
                    .with_first_ttl(1)
                    .with_interface(iface)
                    .with_number_of_queries(3)
                    .with_protocol(Protocol::ICMP)
                    .with_timeout(3000),
                done: false,
            })
        } else {
            Err(String::from("无法找到相匹配ip的网卡"))
        }
    }

    /// 打印基本信息
    pub fn get_info(&self) -> String {
        format!(
            "src_target[ {} ] parse_target[ {} ] gateway[ {} ]",
            self.src_target,
            self.target,
            get_default_gateway()
                .and_then(|x| Ok(x.ip_addr.to_string()))
                .unwrap_or("".to_owned())
        )
    }

    /// Returns a vector of traceroute hops
    pub fn perform_traceroute(&mut self) -> Vec<TracerouteHop> {
        let mut hops = Vec::<TracerouteHop>::new();
        for _ in 1..self.config.max_hops {
            if self.done {
                return hops;
            }
            match self.next() {
                Some(hop) => hops.push(hop),
                None => {}
            }
        }
        return hops;
    }

    /// Get next hop on the route. Increases TTL
    fn calculate_next_hop(&mut self) -> TracerouteHop {
        let mut query_results = Vec::<TracerouteQueryResult>::new();
        for _ in 0..self.config.number_of_queries {
            let result = self.get_next_query_result();
            if result.addr.len() == 0
                || query_results
                    .iter()
                    .filter(|query_result| query_result.addr == result.addr)
                    .next()
                    .is_none()
            {
                query_results.push(result)
            }
        }
        TracerouteHop {
            ttl: self.config.channel.increment_ttl(),
            query_result: query_results,
        }
    }

    /// Runs a query to the destination and returns RTT and IP of the router where
    /// time-to-live-exceeded. Doesn't increase TTL
    fn get_next_query_result(&mut self) -> TracerouteQueryResult {
        let now = std::time::SystemTime::now();
        self.config.channel.send_to(self.target);
        let hop_ip = self.config.channel.recv_timeout(Duration::from_secs(1));
        TracerouteQueryResult {
            rtt: now.elapsed().unwrap_or(Duration::from_millis(0)),
            addr: if hop_ip == "*" {
                vec![]
            } else {
                let dn = dns::lookup_addr(
                    &hop_ip
                        .parse::<IpAddr>()
                        .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
                )
                .unwrap_or("".to_owned());
                if dn == hop_ip {
                    vec![dn]
                } else {
                    vec![dn, hop_ip]
                }
            },
        }
    }
}

#[derive(PartialEq)]
/// Protocol to be used for traceroute
pub enum Protocol {
    /// UDP-based traceroute
    UDP,
    /// TCP-based traceroute
    TCP,
    /// ICMP-based traceroute
    ICMP,
}

pub(crate) struct Channel {
    interface: NetworkInterface,
    packet_builder: packet_builder::PacketBuilder,
    payload_offset: usize,
    port: u16,
    ttl: u8,
    seq: u16,
}

impl Default for Channel {
    fn default() -> Self {
        let available_interfaces = get_available_interfaces();

        let default_interface = available_interfaces
            .iter()
            .next()
            .expect("no interfaces available")
            .clone();

        Channel::new(default_interface, 33434, 1)
    }
}

impl Channel {
    pub fn new(network_interface: NetworkInterface, port: u16, ttl: u8) -> Self {
        let source_ip = network_interface
            .ips
            .iter()
            .filter(|i| i.is_ipv4())
            .next()
            .expect("couldn't get interface IP")
            .ip()
            .to_string();

        let source_ip = Ipv4Addr::from_str(source_ip.as_str()).expect("malformed source ip");
        let payload_offset = if cfg!(any(target_os = "macos", target_os = "ios"))
            && network_interface.is_up()
            && !network_interface.is_broadcast()
            && ((!network_interface.is_loopback() && network_interface.is_point_to_point())
                || network_interface.is_loopback())
        {
            if network_interface.is_loopback() {
                14
            } else {
                0
            }
        } else {
            0
        };

        Channel {
            interface: network_interface.clone(),
            packet_builder: packet_builder::PacketBuilder::new(
                Protocol::UDP,
                network_interface.mac.unwrap(),
                source_ip,
            ),
            payload_offset,
            port,
            ttl,
            seq: 0,
        }
    }

    /// Change protocol of packet_builder
    pub(crate) fn change_protocol(&mut self, new_protocol: Protocol) {
        self.packet_builder.protocol = new_protocol;
    }

    /// Increments current TTL
    pub(crate) fn increment_ttl(&mut self) -> u8 {
        self.ttl += 1;
        self.ttl - 1
    }

    /// Checks whether the current TTL exceeds maximum number of hops
    pub(crate) fn max_hops_reached(&self, max_hops: u8) -> bool {
        self.ttl > max_hops
    }

    /// Sends a packet
    pub(crate) fn send_to(&mut self, destination_ip: IpAddr) {
        match destination_ip {
            IpAddr::V4(dst_ip) => {
                let (mut tx, _) = match channel(&self.interface, Default::default()) {
                    Ok(pnet::datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
                    Ok(_) => panic!("libtraceroute: unhandled util type"),
                    Err(e) => panic!("libtraceroute: unable to create util: {}", e),
                };
                let buf = self
                    .packet_builder
                    .build_packet(dst_ip, self.ttl, self.port + self.seq);
                tx.send_to(&buf, None);
                if self.packet_builder.protocol != Protocol::TCP {
                    self.seq += 1;
                }
            }
            IpAddr::V6(_) => {}
        }
    }

    /// Waits for the expected ICMP packet for specified amount of time
    pub(crate) fn recv_timeout(&mut self, timeout: Duration) -> String {
        let processor =
            async_std::task::spawn(Self::recv(self.interface.clone(), self.payload_offset));
        let ip = block_on(async {
            match async_std::future::timeout(timeout, processor).await {
                Ok(ip) => ip,
                Err(_) => String::from("*"),
            }
        });
        ip
    }

    /// Waits for the expected ICMP packet to arrive until interrupted.
    async fn recv(interface: NetworkInterface, payload_offset: usize) -> String {
        loop {
            match process_incoming_packet(interface.clone(), payload_offset) {
                Ok(ip) => return ip,
                Err(_) => {}
            }
        }
    }
}

/// Returns the list of interfaces that are up, not loopback, not point-to-point,
/// and have an IPv4 address associated with them.
pub fn get_available_interfaces() -> Vec<NetworkInterface> {
    let all_interfaces = pnet::datalink::interfaces();

    let available_interfaces: Vec<NetworkInterface>;

    available_interfaces = if cfg!(target_family = "windows") {
        all_interfaces
            .into_iter()
            .filter(|e| {
                e.mac.is_some()
                    && e.mac.unwrap() != MacAddr::zero()
                    && e.ips
                        .iter()
                        .filter(|ip| ip.ip().to_string() != "0.0.0.0")
                        .next()
                        .is_some()
            })
            .collect()
    } else {
        all_interfaces
            .into_iter()
            .filter(|e| {
                e.is_up()
                    && !e.is_loopback()
                    && e.ips.iter().filter(|ip| ip.is_ipv4()).next().is_some()
                    && e.mac.is_some()
                    && e.mac.unwrap() != MacAddr::zero()
            })
            .collect()
    };

    available_interfaces
}

// TODO: add checks for ICMP code icmp[1] = 0 and icmp[1] = 3
/// Processes ICMP packets. Returns addresses of packets that conform to the following
/// Berkeley Packet filter formula: `icmp and (icmp[0] = 11) or (icmp[0] = 3)`, thus
/// accepting all ICMP packets that have information about the status of UDP packets used for
/// traceroute.
fn handle_icmp_packet(source: IpAddr, packet: &[u8]) -> Result<String, &'static str> {
    let icmp_packet = IcmpPacket::new(packet).expect("malformed ICMP packet");

    match icmp_packet.get_icmp_type() {
        IcmpTypes::TimeExceeded => Ok(source.to_string()),
        IcmpTypes::EchoReply => Ok(source.to_string()),
        IcmpTypes::DestinationUnreachable => Ok(source.to_string()),
        _ => Err("wrong packet"),
    }
}

/// Processes IPv4 packet and passes it on to transport layer packet handler.
fn handle_ipv4_packet(packet: &[u8]) -> Result<String, &'static str> {
    let header = Ipv4Packet::new(packet).expect("malformed IPv4 packet");

    let source = IpAddr::V4(header.get_source());
    let payload = header.payload();

    match header.get_next_level_protocol() {
        IpNextHeaderProtocols::Icmp => handle_icmp_packet(source, payload),
        _ => Err("wrong packet"),
    }
}

/// Processes ethernet frame and rejects all packets that are not IPv4.
fn handle_ethernet_frame(packet: &[u8]) -> Result<String, &'static str> {
    let ethernet = EthernetPacket::new(packet).expect("malformed Ethernet frame");
    match ethernet.get_ethertype() {
        EtherTypes::Ipv4 => return handle_ipv4_packet(ethernet.payload()),
        _ => Err("wrong packet"),
    }
}

/// Start capturing packets until until expected ICMP packet was received.
fn process_incoming_packet(
    interface: NetworkInterface,
    payload_offset: usize,
) -> Result<String, &'static str> {
    let (_, mut rx) = match channel(&interface, Default::default()) {
        Ok(pnet::datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("libtraceroute: unhandled util type"),
        Err(e) => panic!("libtraceroute: unable to create util: {}", e),
    };
    match rx.next() {
        Ok(packet) => {
            if payload_offset > 0 && packet.len() > payload_offset {
                return handle_ipv4_packet(&packet[payload_offset..]);
            }
            return handle_ethernet_frame(packet);
        }
        Err(e) => panic!("libtraceroute: unable to receive packet: {}", e),
    }
}

/// Traceroute configurations
pub struct Config {
    port: u16,
    max_hops: u32,
    number_of_queries: u32,
    ttl: u8,
    timeout: Duration,
    channel: Channel,
}

/// Single traceroute hop containing TTL and a vector of traceroute query results
pub struct TracerouteHop {
    /// Current Time-To-Live
    pub ttl: u8,
    /// Traceroute query results
    pub query_result: Vec<TracerouteQueryResult>,
}

/// Result of a single query execution - IP and RTT
pub struct TracerouteQueryResult {
    /// Round-Trip Time
    pub rtt: Duration,
    /// IP address of a remote node
    pub addr: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            port: 33434,
            max_hops: 30,
            number_of_queries: 3,
            ttl: 1,
            timeout: Duration::from_secs(1),
            channel: Default::default(),
        }
    }
}

impl Config {
    /// Builder: Port for traceroute. Will be incremented on every query (except for TCP-based traceroute)
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Builder: Maximum number of hops
    pub fn with_max_hops(mut self, max_hops: u32) -> Self {
        self.max_hops = max_hops;
        self
    }

    /// Builder: Number of queries to run per hop
    pub fn with_number_of_queries(mut self, number_of_queries: u32) -> Self {
        self.number_of_queries = number_of_queries;
        self
    }

    /// Builder: Protocol. Supported: UDP, TCP, ICMP
    pub fn with_protocol(mut self, protocol: Protocol) -> Self {
        self.channel.change_protocol(protocol);
        self
    }

    /// Builder: Interface that will be used for sending and receiving packets
    pub fn with_interface(mut self, network_interface: NetworkInterface) -> Self {
        self.channel = Channel::new(network_interface, self.port, self.ttl);
        self
    }

    /// Builder: First TTL to record
    pub fn with_first_ttl(mut self, first_ttl: u8) -> Self {
        self.ttl = first_ttl;
        self
    }

    /// Builder: Timeout per query
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Duration::from_millis(timeout);
        self
    }
}
