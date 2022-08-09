#![cfg(feature="os")]
use e_libscanner::os;
use e_libscanner::Opts;

fn main() -> Result<(), String> {
    // more command information use: -h
    let mut scanner = Opts::new(Some(&[
        "e-libscanner",
        "--ips",
        "192.168.80.8",
        "192.168.80.1",
        "--ports",
        "80",
        "135",
        "554",
        "8000",
        "22",
        "--model",
        "os",
        "--no-gui",
        "--",
        "-AS",
    ]))?
    .init()?
    .downcast::<os::Scanner>()
    .unwrap();
    let results = scanner.scan(None);
    for result in results {
        println!("{}", result.ip_addr);
        println!("{:?}", result.icmp_echo_result);
        println!("{:?}", result.icmp_timestamp_result);
        println!("{:?}", result.icmp_address_mask_result);
        println!("{:?}", result.icmp_information_result);
        println!("{:?}", result.icmp_unreachable_ip_result);
        println!("{:?}", result.icmp_unreachable_data_result);
        println!("{:?}", result.tcp_syn_ack_result);
        println!("{:?}", result.tcp_rst_ack_result);
        println!("{:?}", result.tcp_ecn_result);
        println!("{:?}", result.tcp_header_result);
        println!();
    }
    Ok(())
}
