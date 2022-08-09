use std::fmt;

#[allow(dead_code)]
pub type DnsResults = Vec<DnsResult>;
#[derive(Debug, Clone)]
pub struct DnsResult {
    pub src: String,
    pub result: DnsResultType,
}
impl fmt::Display for DnsResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[src_ip[{}] {}]", self.src, self.result)
    }
}
#[derive(Debug, Clone)]
pub enum DnsResultType {
    Host(String),
    Addr(Vec<std::net::IpAddr>),
    Error(String),
}
impl fmt::Display for DnsResultType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = match self {
            DnsResultType::Host(data) => format!("Host[{}]",data),
            DnsResultType::Addr(data) => format!("Addr[{:?}]", data),
            DnsResultType::Error(e) => format!("Err[{}]",e),
        };
        write!(f, "{}", result)
    }
}