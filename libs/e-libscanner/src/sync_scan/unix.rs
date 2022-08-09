use super::receiver;
use crate::{
    data::DATA,
    frame::{
        result::{PortInfo, PortStatus, ScanResult, ScanResults},
        ScanSetting, ScanType,
    },
    packet,
};
use pnet_packet::Packet;
use rayon::prelude::*;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::{
    net::{IpAddr, SocketAddr},
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

fn build_icmpv4_echo_packet() -> Vec<u8> {
    let mut buf = vec![0; 16];
    let mut icmp_packet =
        pnet_packet::icmp::echo_request::MutableEchoRequestPacket::new(&mut buf[..]).unwrap();
    packet::icmp::build_icmp_packet(&mut icmp_packet);
    icmp_packet.packet().to_vec()
}

fn build_tcp_syn_packet(src_ip: IpAddr, src_port: u16, dst_ip: IpAddr, dst_port: u16) -> Vec<u8> {
    let mut vec: Vec<u8> = vec![0; 66];
    let mut tcp_packet = pnet_packet::tcp::MutableTcpPacket::new(
        &mut vec[(packet::ethernet::ETHERNET_HEADER_LEN + packet::ipv4::IPV4_HEADER_LEN)..],
    )
    .unwrap();
    packet::tcp::build_tcp_packet(&mut tcp_packet, src_ip, src_port, dst_ip, dst_port);
    tcp_packet.packet().to_vec()
}

fn build_udp_packet(src_ip: IpAddr, src_port: u16, dst_ip: IpAddr, dst_port: u16) -> Vec<u8> {
    let mut vec: Vec<u8> = vec![0; 66];
    let mut udp_packet = pnet_packet::udp::MutableUdpPacket::new(
        &mut vec[(packet::ethernet::ETHERNET_HEADER_LEN + packet::ipv4::IPV4_HEADER_LEN)..],
    )
    .unwrap();
    packet::udp::build_udp_packet(&mut udp_packet, src_ip, src_port, dst_ip, dst_port);
    udp_packet.packet().to_vec()
}

fn send_icmp_echo_packets(
    socket: &Socket,
    scan_setting: &ScanSetting,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
) {
    for dst in scan_setting.destinations.clone() {
        let socket_addr = SocketAddr::new(dst.dst_ip, 0);
        let sock_addr = SockAddr::from(socket_addr);
        let mut icmp_packet: Vec<u8> = build_icmpv4_echo_packet();
        match socket.send_to(&mut icmp_packet, &sock_addr) {
            Ok(_) => {}
            Err(_) => {}
        }
        match ptx.lock() {
            Ok(lr) => match lr.send(socket_addr) {
                Ok(_) => {}
                Err(_) => {}
            },
            Err(_) => {}
        }
        thread::sleep(scan_setting.send_rate);
    }
}

fn send_tcp_syn_packets(
    socket: &Socket,
    scan_setting: &ScanSetting,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
) {
    for dst in scan_setting.destinations.clone() {
        for port in dst.dst_ports {
            let socket_addr = SocketAddr::new(dst.dst_ip, port);
            let sock_addr = SockAddr::from(socket_addr);
            let mut tcp_packet: Vec<u8> =
                build_tcp_syn_packet(scan_setting.src_ip, scan_setting.src_port, dst.dst_ip, port);
            match socket.send_to(&mut tcp_packet, &sock_addr) {
                Ok(_) => {}
                Err(_) => {}
            }
            match ptx.lock() {
                Ok(lr) => match lr.send(socket_addr) {
                    Ok(_) => {}
                    Err(_) => {}
                },
                Err(_) => {}
            }
            thread::sleep(scan_setting.send_rate);
        }
    }
}

fn send_udp_packets(
    socket: &Socket,
    scan_setting: &ScanSetting,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
) {
    for dst in scan_setting.destinations.clone() {
        for port in dst.dst_ports {
            let socket_addr = SocketAddr::new(dst.dst_ip, port);
            let sock_addr = SockAddr::from(socket_addr);
            let mut udp_packet: Vec<u8> =
                build_udp_packet(scan_setting.src_ip, scan_setting.src_port, dst.dst_ip, port);
            match socket.send_to(&mut udp_packet, &sock_addr) {
                Ok(_) => {}
                Err(_) => {}
            }
            match ptx.lock() {
                Ok(lr) => match lr.send(socket_addr) {
                    Ok(_) => {}
                    Err(_) => {}
                },
                Err(_) => {}
            }
            thread::sleep(scan_setting.send_rate);
        }
    }
}

fn run_connect_scan(
    scan_setting: ScanSetting,
    scan_result: &Arc<Mutex<ScanResults>>,
    pstop: &Arc<Mutex<bool>>,
) {
    let start_time = Instant::now();
    let conn_timeout = Duration::from_millis(200);
    for dst in scan_setting.destinations.clone() {
        if pstop.lock().unwrap() {
            break;
        } else {
            let ip_addr: IpAddr = dst.dst_ip;
            dst.dst_ports.into_par_iter().for_each(|port| {
                let socket = Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap();
                let socket_addr: SocketAddr = SocketAddr::new(ip_addr, port);
                let sock_addr = SockAddr::from(socket_addr);
                match socket.connect_timeout(&sock_addr, conn_timeout) {
                    Ok(_) => {
                        let port_info = PortInfo {
                            port: socket_addr.port(),
                            status: PortStatus::Open,
                            describe: DATA.portmap.get(&socket_addr.port()).unwrap().to_string(),
                        };
                        // Avoid deadlock.
                        let exists: bool = if let Some(r) = scan_result
                            .lock()
                            .unwrap()
                            .result
                            .ip_with_port
                            .get_mut(&socket_addr.ip())
                        {
                            r.push(port_info.clone());
                            true
                        } else {
                            false
                        };
                        if !exists {
                            scan_result
                                .lock()
                                .unwrap()
                                .result
                                .ip_with_port
                                .insert(socket_addr.ip(), vec![port_info]);
                        }
                    }
                    Err(_) => {}
                }
                if Instant::now().duration_since(start_time) > scan_setting.timeout {
                    *stop.lock().unwrap() = true;
                    return;
                } else if pstop.lock().unwrap() {
                    return;
                } else {
                    thread::sleep(scan_setting.send_rate);
                }
            });
        }
    }
}

fn send_ping_packet(
    socket: &Socket,
    scan_setting: &ScanSetting,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
) {
    match scan_setting.scan_type {
        ScanType::IcmpPingScan => {
            send_icmp_echo_packets(socket, scan_setting, ptx);
        }
        ScanType::TcpPingScan => {
            send_tcp_syn_packets(socket, scan_setting, ptx);
        }
        ScanType::UdpPingScan => {
            send_udp_packets(socket, scan_setting, ptx);
        }
        _ => {
            return;
        }
    }
}

fn send_tcp_packets(
    socket: &Socket,
    scan_setting: &ScanSetting,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
) {
    match scan_setting.scan_type {
        ScanType::TcpSynScan => {
            send_tcp_syn_packets(socket, scan_setting, ptx);
        }
        _ => {
            return;
        }
    }
}

pub(crate) fn scan_target(
    scan_setting: ScanSetting,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
    pstop: Option<Arc<Mutex<bool>>>,
) -> ScanResult {
    let socket = match scan_setting.src_ip {
        IpAddr::V4(_) => match scan_setting.scan_type {
            ScanType::IcmpPingScan => {
                Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)).unwrap()
            }
            ScanType::TcpPingScan => {
                Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::TCP)).unwrap()
            }
            ScanType::UdpPingScan => {
                Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::UDP)).unwrap()
            }
            ScanType::TcpSynScan => {
                Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::TCP)).unwrap()
            }
            ScanType::TcpConnectScan => {
                Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)).unwrap()
            }
        },
        IpAddr::V6(_) => match scan_setting.scan_type {
            ScanType::IcmpPingScan => {
                Socket::new(Domain::IPV6, Type::RAW, Some(Protocol::ICMPV6)).unwrap()
            }
            ScanType::TcpPingScan => {
                Socket::new(Domain::IPV6, Type::RAW, Some(Protocol::TCP)).unwrap()
            }
            ScanType::UdpPingScan => {
                Socket::new(Domain::IPV6, Type::RAW, Some(Protocol::UDP)).unwrap()
            }
            ScanType::TcpSynScan => {
                Socket::new(Domain::IPV6, Type::RAW, Some(Protocol::TCP)).unwrap()
            }
            ScanType::TcpConnectScan => {
                Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP)).unwrap()
            }
        },
    };
    let interfaces = pnet_datalink::interfaces();
    let interface = match interfaces
        .into_iter()
        .filter(|interface: &pnet_datalink::NetworkInterface| {
            interface.index == scan_setting.if_index
        })
        .next()
    {
        Some(interface) => interface,
        None => return ScanResult::new(),
    };
    let config = pnet_datalink::Config {
        write_buffer_size: 4096,
        read_buffer_size: 4096,
        read_timeout: None,
        write_timeout: None,
        channel_type: pnet_datalink::ChannelType::Layer2,
        bpf_fd_attempts: 1000,
        linux_fanout: None,
        promiscuous: false,
    };
    let (mut _tx, mut rx) = match pnet_datalink::channel(&interface, config) {
        Ok(pnet_datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error happened {}", e),
    };
    let scan_result: Arc<Mutex<ScanResults>> = Arc::new(Mutex::new(ScanResults::new()));
    let stop: Arc<Mutex<bool>> = if let Some(p) = pstop {
        p
    } else {
        Arc::new(Mutex::new(false))
    };
    let receive_stop = Arc::clone(&stop);
    let receive_result: Arc<Mutex<ScanResults>> = Arc::clone(&scan_result);
    let receive_setting: ScanSetting = scan_setting.clone();
    match scan_setting.scan_type {
        ScanType::IcmpPingScan | ScanType::UdpPingScan | ScanType::TcpPingScan => {
            thread::spawn(move || {
                receiver::receive_packets(&mut rx, receive_setting, &receive_result, &receive_stop);
            });
            send_ping_packet(&socket, &scan_setting, ptx);
            thread::sleep(scan_setting.wait_time);
            *stop.lock().unwrap() = true;
        }
        ScanType::TcpSynScan => {
            thread::spawn(move || {
                receiver::receive_packets(&mut rx, receive_setting, &receive_result, &receive_stop);
            });
            send_tcp_packets(&socket, &scan_setting, ptx);
            thread::sleep(scan_setting.wait_time);
            *stop.lock().unwrap() = true;
        }
        ScanType::TcpConnectScan => {
            run_connect_scan(scan_setting, &receive_result, &stop);
        }
    }
    let result: ScanResult = scan_result.lock().unwrap().result.clone();
    return result;
}
