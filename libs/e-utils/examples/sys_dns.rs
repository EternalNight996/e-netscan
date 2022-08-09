use std::net::{IpAddr, SocketAddr};

use sys_utils::dns::{
    get_hostname, getaddrinfo, getnameinfo, lookup_addr, lookup_host, AddrInfoHints, SockType,
};
fn main() {
    {
        let hostname = "baidu.com";
        let ips: Vec<std::net::IpAddr> = lookup_host(hostname).unwrap();
        println!("dn -> ip {:?}", ips);
    }
    {
        let ip: std::net::IpAddr = "114.114.114.114".parse().unwrap();
        let host = lookup_addr(&ip).unwrap();
        println!("ip -> dn {:?}", host);
        // The string "localhost" on unix, and the hostname on Windows.
    }
    {
        let hostname = "localhost";
        let service = "ssh";
        let hints = AddrInfoHints {
            socktype: SockType::Stream.into(),
            ..AddrInfoHints::default()
        };
        let sockets = getaddrinfo(Some(hostname), Some(service), Some(hints))
            .unwrap()
            .collect::<std::io::Result<Vec<_>>>()
            .unwrap();

        for socket in sockets {
            // Try connecting to socket
            println!("{:?}", socket);
        }
    }

    {
        let ip: IpAddr = "127.0.0.1".parse().unwrap();
        let port = 22;
        let socket: SocketAddr = (ip, port).into();

        let (name, service) = match getnameinfo(&socket, 0) {
            Ok((n, s)) => (n, s),
            Err(e) => panic!("Failed to lookup socket {:?}", e),
        };

        println!("{:?} {:?}", name, service);
        let _ = (name, service);
    }

    {
        let hostname = get_hostname().unwrap();
        println!("get_hostname {}", hostname);
    }
}
