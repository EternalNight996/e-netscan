use std::convert::TryFrom;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, UdpSocket};
#[cfg(target_os = "windows")]
#[path = "./windows.rs"]
mod win;
use pnet::datalink;
#[cfg(target_os = "windows")]
use win::get_interfaces;
mod memalloc;

/// Structure of Network Interface information
#[derive(Clone, Debug)]
pub struct Interface {
    /// Index of network interface
    pub index: u32,
    /// Name of network interface
    pub name: String,
    /// Friendly Name of network interface
    pub friendly_name: Option<String>,
    /// Description of the network interface
    pub description: Option<String>,
    /// Interface Type
    pub if_type: InterfaceType,
    /// MAC address of network interface
    pub mac_addr: Option<MacAddr>,
    /// List of Ipv4Net for the network interface
    pub ipv4: Vec<Ipv4Net>,
    /// List of Ipv6Net for the network interface
    pub ipv6: Vec<Ipv6Net>,
    /// Flags for the network interface (OS Specific)
    pub flags: u32,
    /// Speed in bits per second of the transmit for the network interface
    pub transmit_speed: Option<u64>,
    /// Speed in bits per second of the receive for the network interface
    pub receive_speed: Option<u64>,
    /// Default gateway for the network interface
    pub gateway: Option<Gateway>,
}

/// Structure of default Gateway information
#[derive(Clone, Debug)]
pub struct Gateway {
    /// MAC address of Gateway
    pub mac_addr: MacAddr,
    /// IP address of Gateway
    pub ip_addr: IpAddr,
}

impl Gateway {
    /// Construct a new Gateway instance
    pub fn new() -> Gateway {
        Gateway {
            mac_addr: MacAddr::zero(),
            ip_addr: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        }
    }
}
/// Get default Gateway
pub fn get_default_gateway() -> Result<Gateway, String> {
    let local_ip: IpAddr = match get_local_ipaddr() {
        Ok(local_ip) => local_ip,
        Err(_) => return Err(String::from("Local IP address not found")),
    };
    let interfaces: Vec<Interface> = get_interfaces();
    for iface in interfaces {
        match local_ip {
            IpAddr::V4(local_ipv4) => {
                if iface.ipv4.iter().any(|x| x.addr == local_ipv4) {
                    if let Some(gateway) = iface.gateway {
                        return Ok(gateway);
                    }
                }
            }
            IpAddr::V6(local_ipv6) => {
                if iface.ipv6.iter().any(|x| x.addr == local_ipv6) {
                    if let Some(gateway) = iface.gateway {
                        return Ok(gateway);
                    }
                }
            }
        }
    }
    Err(String::from("Default Gateway not found"))
}

/// Get IP address of the default Network Interface
pub fn get_local_ipaddr() -> Result<IpAddr, String> {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => return Err(String::from(e.to_string())),
    };
    if let Err(e) = socket.connect("1.1.1.1:80") {
        return Err(String::from(e.to_string()));
    };
    match socket.local_addr() {
        Ok(addr) => Ok(addr.ip()),
        Err(e) => return Err(String::from(e.to_string())),
    }
}

#[allow(dead_code)]
pub fn get_interface_index_by_ip(ip_addr: IpAddr) -> Option<u32> {
    for iface in datalink::interfaces() {
        for ip in iface.ips {
            if ip.ip() == ip_addr {
                return Some(iface.index);
            }
        }
    }
    return None;
}

#[cfg(target_os = "windows")]
pub fn get_default_gateway_macaddr() -> [u8; 6] {
    match get_default_gateway() {
        Ok(gateway) => gateway.mac_addr.octets(),
        Err(_) => MacAddr::zero().octets(),
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_default_gateway_macaddr() -> [u8; 6] {
    MacAddr::zero().octets()
}

/// Type of Network Interface
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InterfaceType {
    /// Unknown interface type
    Unknown,
    /// The network interface using an Ethernet connection
    Ethernet,
    /// The network interface using a Token-Ring connection
    TokenRing,
    /// The network interface using a Fiber Distributed Data Interface (FDDI) connection
    Fddi,
    /// The network interface using a basic rate interface Integrated Services Digital Network (ISDN) connection
    BasicIsdn,
    /// The network interface using a primary rate interface Integrated Services Digital Network (ISDN) connection
    PrimaryIsdn,
    /// The network interface using a Point-To-Point protocol (PPP) connection
    Ppp,
    /// The loopback interface (often used for testing)
    Loopback,
    /// The network interface using an Ethernet 3 megabit/second connection
    Ethernet3Megabit,
    /// The network interface using a Serial Line Internet Protocol (SLIP) connection
    Slip,
    /// The network interface using asynchronous transfer mode (ATM) for data transmission
    Atm,
    /// The network interface using a modem
    GenericModem,
    /// The network interface using a Fast Ethernet connection over twisted pair and provides a data rate of 100 megabits per second (100BASE-T)
    FastEthernetT,
    /// The network interface using a connection configured for ISDN and the X.25 protocol.
    Isdn,
    /// The network interface using a Fast Ethernet connection over optical fiber and provides a data rate of 100 megabits per second (100Base-FX)
    FastEthernetFx,
    /// The network interface using a wireless LAN connection (IEEE 802.11)
    Wireless80211,
    /// The network interface using an Asymmetric Digital Subscriber Line (ADSL)
    AsymmetricDsl,
    /// The network interface using a Rate Adaptive Digital Subscriber Line (RADSL)
    RateAdaptDsl,
    /// The network interface using a Symmetric Digital Subscriber Line (SDSL)
    SymmetricDsl,
    /// The network interface using a Very High Data Rate Digital Subscriber Line (VDSL)
    VeryHighSpeedDsl,
    /// The network interface using the Internet Protocol (IP) in combination with asynchronous transfer mode (ATM) for data transmission
    IPOverAtm,
    /// The network interface using a gigabit Ethernet connection and provides a data rate of 1,000 megabits per second (1 gigabit per second)
    GigabitEthernet,
    /// The network interface using a tunnel connection
    Tunnel,
    /// The network interface using a Multirate Digital Subscriber Line
    MultiRateSymmetricDsl,
    /// The network interface using a High Performance Serial Bus
    HighPerformanceSerialBus,
    /// The network interface using a mobile broadband interface for WiMax devices
    Wman,
    /// The network interface using a mobile broadband interface for GSM-based devices
    Wwanpp,
    /// The network interface using a mobile broadband interface for CDMA-based devices
    Wwanpp2,
}

impl InterfaceType {
    /// Returns OS-specific value of InterfaceType
    #[cfg(target_os = "windows")]
    pub fn value(&self) -> u32 {
        match *self {
            InterfaceType::Unknown => 1,
            InterfaceType::Ethernet => 6,
            InterfaceType::TokenRing => 9,
            InterfaceType::Fddi => 15,
            InterfaceType::BasicIsdn => 20,
            InterfaceType::PrimaryIsdn => 21,
            InterfaceType::Ppp => 23,
            InterfaceType::Loopback => 24,
            InterfaceType::Ethernet3Megabit => 26,
            InterfaceType::Slip => 28,
            InterfaceType::Atm => 37,
            InterfaceType::GenericModem => 48,
            InterfaceType::FastEthernetT => 62,
            InterfaceType::Isdn => 63,
            InterfaceType::FastEthernetFx => 69,
            InterfaceType::Wireless80211 => 71,
            InterfaceType::AsymmetricDsl => 94,
            InterfaceType::RateAdaptDsl => 95,
            InterfaceType::SymmetricDsl => 96,
            InterfaceType::VeryHighSpeedDsl => 97,
            InterfaceType::IPOverAtm => 114,
            InterfaceType::GigabitEthernet => 117,
            InterfaceType::Tunnel => 131,
            InterfaceType::MultiRateSymmetricDsl => 143,
            InterfaceType::HighPerformanceSerialBus => 144,
            InterfaceType::Wman => 237,
            InterfaceType::Wwanpp => 243,
            InterfaceType::Wwanpp2 => 244,
        }
    }
    /// Returns OS-specific value of InterfaceType
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub fn value(&self) -> u32 {
        match *self {
            InterfaceType::Ethernet => 1,
            InterfaceType::TokenRing => 4,
            InterfaceType::Fddi => 774,
            InterfaceType::Ppp => 512,
            InterfaceType::Loopback => 772,
            InterfaceType::Ethernet3Megabit => 2,
            InterfaceType::Slip => 256,
            InterfaceType::Atm => 19,
            InterfaceType::Wireless80211 => 801,
            InterfaceType::Tunnel => 768,
            _ => u32::MAX,
        }
    }
    /// Returns OS-specific value of InterfaceType
    #[cfg(any(
        target_os = "macos",
        target_os = "openbsd",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "ios"
    ))]
    pub fn value(&self) -> u32 {
        // TODO
        match *self {
            _ => 0,
        }
    }
    /// Returns name of InterfaceType
    pub fn name(&self) -> String {
        match *self {
            InterfaceType::Unknown => String::from("Unknown"),
            InterfaceType::Ethernet => String::from("Ethernet"),
            InterfaceType::TokenRing => String::from("Token Ring"),
            InterfaceType::Fddi => String::from("FDDI"),
            InterfaceType::BasicIsdn => String::from("Basic ISDN"),
            InterfaceType::PrimaryIsdn => String::from("Primary ISDN"),
            InterfaceType::Ppp => String::from("PPP"),
            InterfaceType::Loopback => String::from("Loopback"),
            InterfaceType::Ethernet3Megabit => String::from("Ethernet 3 megabit"),
            InterfaceType::Slip => String::from("SLIP"),
            InterfaceType::Atm => String::from("ATM"),
            InterfaceType::GenericModem => String::from("Generic Modem"),
            InterfaceType::FastEthernetT => String::from("Fast Ethernet T"),
            InterfaceType::Isdn => String::from("ISDN"),
            InterfaceType::FastEthernetFx => String::from("Fast Ethernet FX"),
            InterfaceType::Wireless80211 => String::from("Wireless IEEE 802.11"),
            InterfaceType::AsymmetricDsl => String::from("Asymmetric DSL"),
            InterfaceType::RateAdaptDsl => String::from("Rate Adaptive DSL"),
            InterfaceType::SymmetricDsl => String::from("Symmetric DSL"),
            InterfaceType::VeryHighSpeedDsl => String::from("Very High Data Rate DSL"),
            InterfaceType::IPOverAtm => String::from("IP over ATM"),
            InterfaceType::GigabitEthernet => String::from("Gigabit Ethernet"),
            InterfaceType::Tunnel => String::from("Tunnel"),
            InterfaceType::MultiRateSymmetricDsl => String::from("Multi-Rate Symmetric DSL"),
            InterfaceType::HighPerformanceSerialBus => String::from("High Performance Serial Bus"),
            InterfaceType::Wman => String::from("WMAN"),
            InterfaceType::Wwanpp => String::from("WWANPP"),
            InterfaceType::Wwanpp2 => String::from("WWANPP2"),
        }
    }
}

impl TryFrom<u32> for InterfaceType {
    type Error = ();
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            x if x == InterfaceType::Unknown.value() => Ok(InterfaceType::Unknown),
            x if x == InterfaceType::Ethernet.value() => Ok(InterfaceType::Ethernet),
            x if x == InterfaceType::TokenRing.value() => Ok(InterfaceType::TokenRing),
            x if x == InterfaceType::Fddi.value() => Ok(InterfaceType::Fddi),
            x if x == InterfaceType::BasicIsdn.value() => Ok(InterfaceType::BasicIsdn),
            x if x == InterfaceType::PrimaryIsdn.value() => Ok(InterfaceType::PrimaryIsdn),
            x if x == InterfaceType::Ppp.value() => Ok(InterfaceType::Ppp),
            x if x == InterfaceType::Loopback.value() => Ok(InterfaceType::Loopback),
            x if x == InterfaceType::Ethernet3Megabit.value() => {
                Ok(InterfaceType::Ethernet3Megabit)
            }
            x if x == InterfaceType::Slip.value() => Ok(InterfaceType::Slip),
            x if x == InterfaceType::Atm.value() => Ok(InterfaceType::Atm),
            x if x == InterfaceType::GenericModem.value() => Ok(InterfaceType::GenericModem),
            x if x == InterfaceType::FastEthernetT.value() => Ok(InterfaceType::FastEthernetT),
            x if x == InterfaceType::Isdn.value() => Ok(InterfaceType::Isdn),
            x if x == InterfaceType::FastEthernetFx.value() => Ok(InterfaceType::FastEthernetFx),
            x if x == InterfaceType::Wireless80211.value() => Ok(InterfaceType::Wireless80211),
            x if x == InterfaceType::AsymmetricDsl.value() => Ok(InterfaceType::AsymmetricDsl),
            x if x == InterfaceType::RateAdaptDsl.value() => Ok(InterfaceType::RateAdaptDsl),
            x if x == InterfaceType::SymmetricDsl.value() => Ok(InterfaceType::SymmetricDsl),
            x if x == InterfaceType::VeryHighSpeedDsl.value() => {
                Ok(InterfaceType::VeryHighSpeedDsl)
            }
            x if x == InterfaceType::IPOverAtm.value() => Ok(InterfaceType::IPOverAtm),
            x if x == InterfaceType::GigabitEthernet.value() => Ok(InterfaceType::GigabitEthernet),
            x if x == InterfaceType::Tunnel.value() => Ok(InterfaceType::Tunnel),
            x if x == InterfaceType::MultiRateSymmetricDsl.value() => {
                Ok(InterfaceType::MultiRateSymmetricDsl)
            }
            x if x == InterfaceType::HighPerformanceSerialBus.value() => {
                Ok(InterfaceType::HighPerformanceSerialBus)
            }
            x if x == InterfaceType::Wman.value() => Ok(InterfaceType::Wman),
            x if x == InterfaceType::Wwanpp.value() => Ok(InterfaceType::Wwanpp),
            x if x == InterfaceType::Wwanpp2.value() => Ok(InterfaceType::Wwanpp2),
            _ => Err(()),
        }
    }
}

/// Structure of IPv4 Network
#[derive(Clone, Debug)]
pub struct Ipv4Net {
    /// IPv4 Address
    pub addr: Ipv4Addr,
    /// Prefix Length
    pub prefix_len: u8,
    /// Network Mask
    pub netmask: Ipv4Addr,
}

impl Ipv4Net {
    /// Construct a new Ipv4Net instance from IPv4 Address and Prefix Length
    pub fn new(ipv4_addr: Ipv4Addr, prefix_len: u8) -> Ipv4Net {
        Ipv4Net {
            addr: ipv4_addr,
            prefix_len: prefix_len,
            netmask: prefix_to_ipv4_netmask(prefix_len),
        }
    }
    /// Construct a new Ipv4Net instance from IPv4 Address and Network Mask
    pub fn new_with_netmask(ipv4_addr: Ipv4Addr, netmask: Ipv4Addr) -> Ipv4Net {
        Ipv4Net {
            addr: ipv4_addr,
            prefix_len: ipv4_netmask_to_prefix(netmask),
            netmask: netmask,
        }
    }
}

/// Structure of IPv6 Network
#[derive(Clone, Debug)]
pub struct Ipv6Net {
    /// IPv6 Address
    pub addr: Ipv6Addr,
    /// Prefix Length
    pub prefix_len: u8,
    /// Network Mask
    pub netmask: Ipv6Addr,
}

impl Ipv6Net {
    /// Construct a new Ipv6Net instance from IPv6 Address and Prefix Length
    pub fn new(ipv6_addr: Ipv6Addr, prefix_len: u8) -> Ipv6Net {
        Ipv6Net {
            addr: ipv6_addr,
            prefix_len: prefix_len,
            netmask: prefix_to_ipv6_netmask(prefix_len),
        }
    }
    /// Construct a new Ipv6Net instance from IPv6 Address and Network Mask
    pub fn new_with_netmask(ipv6_addr: Ipv6Addr, netmask: Ipv6Addr) -> Ipv6Net {
        Ipv6Net {
            addr: ipv6_addr,
            prefix_len: ipv6_netmask_to_prefix(netmask),
            netmask: netmask,
        }
    }
}

fn ipv4_netmask_to_prefix(netmask: Ipv4Addr) -> u8 {
    let netmask = u32::from(netmask);
    let prefix = (!netmask).leading_zeros() as u8;
    if (u64::from(netmask) << prefix) & 0xffff_ffff != 0 {
        0
    } else {
        prefix
    }
}

fn ipv6_netmask_to_prefix(netmask: Ipv6Addr) -> u8 {
    let netmask = netmask.segments();
    let mut mask_iter = netmask.iter();
    let mut prefix = 0;
    for &segment in &mut mask_iter {
        if segment == 0xffff {
            prefix += 16;
        } else if segment == 0 {
            break;
        } else {
            let prefix_bits = (!segment).leading_zeros() as u8;
            if segment << prefix_bits != 0 {
                return 0;
            }
            prefix += prefix_bits;
            break;
        }
    }
    for &segment in mask_iter {
        if segment != 0 {
            return 0;
        }
    }
    prefix
}

fn prefix_to_ipv4_netmask(prefix_len: u8) -> Ipv4Addr {
    let netmask_u32: u32 = u32::max_value()
        .checked_shl(32 - prefix_len as u32)
        .unwrap_or(0);
    Ipv4Addr::from(netmask_u32)
}

fn prefix_to_ipv6_netmask(prefix_len: u8) -> Ipv6Addr {
    let netmask_u128: u128 = u128::max_value()
        .checked_shl((128 - prefix_len) as u32)
        .unwrap_or(u128::min_value());
    Ipv6Addr::from(netmask_u128)
}

#[cfg(target_endian = "little")]
fn htonl(val: u32) -> u32 {
    let o3 = (val >> 24) as u8;
    let o2 = (val >> 16) as u8;
    let o1 = (val >> 8) as u8;
    let o0 = val as u8;
    (o0 as u32) << 24 | (o1 as u32) << 16 | (o2 as u32) << 8 | (o3 as u32)
}

#[cfg(target_endian = "big")]
fn htonl(val: u32) -> u32 {
    val
}

/// Structure of MAC address
#[derive(Clone, Debug)]
pub struct MacAddr(u8, u8, u8, u8, u8, u8);

impl MacAddr {
    /// Construct a new MacAddr instance from the given octets
    pub fn new(octets: [u8; 6]) -> MacAddr {
        MacAddr(
            octets[0], octets[1], octets[2], octets[3], octets[4], octets[5],
        )
    }
    /// Returns an array of MAC address octets
    pub fn octets(&self) -> [u8; 6] {
        [self.0, self.1, self.2, self.3, self.4, self.5]
    }
    /// Return a formatted string of MAC address
    pub fn address(&self) -> String {
        format!(
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0, self.1, self.2, self.3, self.4, self.5
        )
    }
    /// Construct an all-zero MacAddr instance
    pub fn zero() -> MacAddr {
        MacAddr(0, 0, 0, 0, 0, 0)
    }
    /// Construct a new MacAddr instance from a colon-separated string of hex format
    pub fn from_hex_format(hex_mac_addr: &str) -> MacAddr {
        if hex_mac_addr.len() != 17 {
            return MacAddr(0, 0, 0, 0, 0, 0);
        }
        let fields: Vec<&str> = hex_mac_addr.split(":").collect();
        let o1: u8 = u8::from_str_radix(&fields[0], 0x10).unwrap_or(0);
        let o2: u8 = u8::from_str_radix(&fields[1], 0x10).unwrap_or(0);
        let o3: u8 = u8::from_str_radix(&fields[2], 0x10).unwrap_or(0);
        let o4: u8 = u8::from_str_radix(&fields[3], 0x10).unwrap_or(0);
        let o5: u8 = u8::from_str_radix(&fields[4], 0x10).unwrap_or(0);
        let o6: u8 = u8::from_str_radix(&fields[5], 0x10).unwrap_or(0);
        MacAddr(o1, o2, o3, o4, o5, o6)
    }
}

impl std::fmt::Display for MacAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = write!(
            f,
            "{:<02x}:{:<02x}:{:<02x}:{:<02x}:{:<02x}:{:<02x}",
            self.0, self.1, self.2, self.3, self.4, self.5
        );
        Ok(())
    }
}
