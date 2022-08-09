use super::receiver;
use super::socket::AsyncSocket;
use crate::data::DATA;
use crate::frame::{
    result::{PortInfo, PortStatus, ScanResult, ScanResults},
    Destination, ScanSetting, ScanType,
};
use crate::packet;
use async_io::{Async, Timer};
use futures::executor::ThreadPool;
use futures::stream::{self, StreamExt};
use futures::task::SpawnExt;
use futures_lite::{future::FutureExt, io};
use pnet_packet::Packet;
use socket2::{Protocol, SockAddr, Type};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

async fn build_icmpv4_echo_packet() -> Vec<u8> {
    let mut buf = vec![0; 16];
    let mut icmp_packet =
        pnet_packet::icmp::echo_request::MutableEchoRequestPacket::new(&mut buf[..]).unwrap();
    packet::icmp::build_icmp_packet(&mut icmp_packet);
    icmp_packet.packet().to_vec()
}

async fn build_tcp_syn_packet(
    src_ip: IpAddr,
    src_port: u16,
    dst_ip: IpAddr,
    dst_port: u16,
) -> Vec<u8> {
    let mut vec: Vec<u8> = vec![0; 66];
    let mut tcp_packet = pnet_packet::tcp::MutableTcpPacket::new(
        &mut vec[(packet::ethernet::ETHERNET_HEADER_LEN + packet::ipv4::IPV4_HEADER_LEN)..],
    )
    .unwrap();
    packet::tcp::build_tcp_packet(&mut tcp_packet, src_ip, src_port, dst_ip, dst_port);
    tcp_packet.packet().to_vec()
}

async fn build_udp_packet(src_ip: IpAddr, src_port: u16, dst_ip: IpAddr, dst_port: u16) -> Vec<u8> {
    let mut vec: Vec<u8> = vec![0; 66];
    let mut udp_packet = pnet_packet::udp::MutableUdpPacket::new(
        &mut vec[(packet::ethernet::ETHERNET_HEADER_LEN + packet::ipv4::IPV4_HEADER_LEN)..],
    )
    .unwrap();
    packet::udp::build_udp_packet(&mut udp_packet, src_ip, src_port, dst_ip, dst_port);
    udp_packet.packet().to_vec()
}

async fn send_icmp_echo_packets(
    socket: &AsyncSocket,
    scan_setting: &ScanSetting,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
) {
    let fut_host = stream::iter(scan_setting.destinations.clone()).for_each_concurrent(
        scan_setting.hosts_concurrency,
        |dst| {
            thread::sleep(scan_setting.send_rate);
            let socket_addr = SocketAddr::new(dst.dst_ip, 0);
            let sock_addr = SockAddr::from(socket_addr);
            async move {
                let mut icmp_packet: Vec<u8> = build_icmpv4_echo_packet().await;
                match socket.send_to(&mut icmp_packet, &sock_addr).await {
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
            }
        },
    );
    fut_host.await;
}

async fn send_tcp_syn_packets(
    socket: &AsyncSocket,
    scan_setting: &ScanSetting,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
) {
    let fut_host = stream::iter(scan_setting.destinations.clone()).for_each_concurrent(
        scan_setting.hosts_concurrency,
        |dst| async move {
            let fut_port = stream::iter(dst.dst_ports.clone()).for_each_concurrent(
                scan_setting.ports_concurrency,
                |port| {
                    thread::sleep(scan_setting.send_rate);
                    let dst = dst.clone();
                    let socket_addr = SocketAddr::new(dst.dst_ip, port);
                    let sock_addr = SockAddr::from(socket_addr);
                    async move {
                        let mut tcp_packet: Vec<u8> = build_tcp_syn_packet(
                            scan_setting.src_ip,
                            scan_setting.src_port,
                            dst.dst_ip,
                            port,
                        )
                        .await;
                        match socket.send_to(&mut tcp_packet, &sock_addr).await {
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
                    }
                },
            );
            fut_port.await;
        },
    );
    fut_host.await;
}

async fn send_udp_packets(
    socket: &AsyncSocket,
    scan_setting: &ScanSetting,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
) {
    let fut_host = stream::iter(scan_setting.destinations.clone()).for_each_concurrent(
        scan_setting.hosts_concurrency,
        |dst| async move {
            let fut_port = stream::iter(dst.dst_ports.clone()).for_each_concurrent(
                scan_setting.ports_concurrency,
                |port| {
                    thread::sleep(scan_setting.send_rate);
                    let dst = dst.clone();
                    let socket_addr = SocketAddr::new(dst.dst_ip, port);
                    let sock_addr = SockAddr::from(socket_addr);
                    async move {
                        let mut udp_packet: Vec<u8> = build_udp_packet(
                            scan_setting.src_ip,
                            scan_setting.src_port,
                            dst.dst_ip,
                            port,
                        )
                        .await;
                        match socket.send_to(&mut udp_packet, &sock_addr).await {
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
                    }
                },
            );
            fut_port.await;
        },
    );
    fut_host.await;
}

async fn try_connect_ports(
    concurrency: usize,
    send_rate: Duration,
    dst: Destination,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
) -> (IpAddr, Vec<PortInfo>) {
    let (channel_tx, channel_rx) = mpsc::channel();
    let conn_timeout = Duration::from_millis(200);
    let fut = stream::iter(dst.dst_ports.clone()).for_each_concurrent(concurrency, |port| {
        thread::sleep(send_rate);
        let dst = dst.clone();
        let channel_tx = channel_tx.clone();
        async move {
            let socket_addr = SocketAddr::new(dst.dst_ip, port);
            let stream = Async::<TcpStream>::connect(socket_addr)
                .or(async {
                    Timer::after(conn_timeout).await;
                    Err(io::ErrorKind::TimedOut.into())
                })
                .await;
            match stream {
                Ok(_) => {
                    let _ = channel_tx.send(port);
                }
                _ => {}
            }
            match ptx.lock() {
                Ok(lr) => match lr.send(socket_addr) {
                    Ok(_) => {}
                    Err(_) => {}
                },
                Err(_) => {}
            }
        }
    });
    fut.await;
    drop(channel_tx);
    let mut open_ports: Vec<PortInfo> = vec![];
    loop {
        match channel_rx.recv() {
            Ok(port) => {
                open_ports.push(PortInfo {
                    port,
                    status: PortStatus::Open,
                    describe: DATA.portmap.get(&port).unwrap().to_string(),
                });
            }
            Err(_) => {
                break;
            }
        }
    }
    (dst.dst_ip, open_ports)
}

async fn run_connect_scan(
    scan_setting: ScanSetting,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
) -> ScanResult {
    let scan_result: Vec<(IpAddr, Vec<PortInfo>)> =
        stream::iter(scan_setting.destinations.clone().into_iter())
            .map(|dst| try_connect_ports(scan_setting.ports_concurrency, scan_setting.send_rate, dst, ptx))
            .buffer_unordered(scan_setting.hosts_concurrency)
            .collect()
            .await;
    let mut ip_with_port: HashMap<IpAddr, Vec<PortInfo>> = HashMap::new();
    for (ip, ports) in scan_result {
        ip_with_port.insert(ip, ports);
    }
    let mut result = ScanResult::new();
    result.ip_with_port = ip_with_port;
    result
}

async fn send_ping_packet(
    socket: &AsyncSocket,
    scan_setting: &ScanSetting,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
) {
    match scan_setting.scan_type {
        ScanType::IcmpPingScan => {
            send_icmp_echo_packets(socket, scan_setting, ptx).await;
        }
        ScanType::TcpPingScan => {
            send_tcp_syn_packets(socket, scan_setting, ptx).await;
        }
        ScanType::UdpPingScan => {
            send_udp_packets(socket, scan_setting, ptx).await;
        }
        _ => {
            return;
        }
    }
}

async fn send_tcp_packets(
    socket: &AsyncSocket,
    scan_setting: &ScanSetting,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
) {
    match scan_setting.scan_type {
        ScanType::TcpSynScan => {
            send_tcp_syn_packets(socket, scan_setting, ptx).await;
        }
        _ => {
            return;
        }
    }
}

pub(crate) async fn scan_target(
    scan_setting: ScanSetting,
    ptx: &Arc<Mutex<Sender<SocketAddr>>>,
    pstop: Option<Arc<Mutex<bool>>>,
) -> ScanResult {
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
    let (_tx, mut rx) = match pnet_datalink::channel(&interface, config) {
        Ok(pnet_datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unknown channel type"),
        Err(e) => panic!("Error happened {}", e),
    };
    let scan_result: Arc<Mutex<ScanResults>> =
        Arc::new(Mutex::new(ScanResults::new()));
    let stop: Arc<Mutex<bool>> = if let Some(p) = pstop {
        p
    } else {
        Arc::new(Mutex::new(false))
    };
    let receive_stop = Arc::clone(&stop);
    let receive_result = Arc::clone(&scan_result);
    let receive_setting: ScanSetting = scan_setting.clone();

    let socket = match scan_setting.scan_type {
        ScanType::IcmpPingScan => {
            AsyncSocket::new(scan_setting.src_ip, Type::RAW, Protocol::ICMPV4).unwrap()
        }
        ScanType::TcpPingScan => {
            AsyncSocket::new(scan_setting.src_ip, Type::RAW, Protocol::TCP).unwrap()
        }
        ScanType::UdpPingScan => {
            AsyncSocket::new(scan_setting.src_ip, Type::RAW, Protocol::UDP).unwrap()
        }
        ScanType::TcpConnectScan => {
            return run_connect_scan(scan_setting, ptx).await;
        }
        ScanType::TcpSynScan => {
            AsyncSocket::new(scan_setting.src_ip, Type::RAW, Protocol::TCP).unwrap()
        }
    };
    let executor = ThreadPool::new().unwrap();
    let future = async move {
        receiver::receive_packets(&mut rx, receive_setting, &receive_result, &receive_stop).await;
    };
    executor.spawn(future).unwrap();
    if let ScanType::TcpSynScan = scan_setting.scan_type {
        send_tcp_packets(&socket, &scan_setting, ptx).await;
        thread::sleep(scan_setting.wait_time);
        *stop.lock().unwrap() = true;
    } else {
        send_ping_packet(&socket, &scan_setting, ptx).await;
        thread::sleep(scan_setting.wait_time);
    }

    let result: ScanResult = scan_result.lock().unwrap().result.clone();
    return result;
}
