#![cfg(feature="service")]
use e_libscanner::service::{self, PortDatabase, ServiceDetector};
use e_libscanner::Opts;
use std::thread;

fn main() -> Result<(), String> {
    // more command information use: -h
    let mut scanner = Opts::new(Some(&[
        "e-libscanner",
        "--ips",
        "192.168.80.10",
        "--ports",
        "8000",
        "8080",
        "20-100",
        "--rate",
        "1",
        "--model",
        "service",
        "--scan",
        "tcpsyn",
        "--no-gui",
    ]))?
    .init()?
    .downcast::<service::Scanner>()
    .unwrap();
    let rx = scanner.get_progress_receiver();
    let time = std::time::Instant::now();
    // Run scan
    let handle = thread::spawn(move || scanner.scan(None));
    // Print progress
    while let Ok(socket_addr) = rx.lock().unwrap().recv() {
        println!("Check: {}", socket_addr);
    }
    let result = handle.join().unwrap();

    for (ip, _ports) in result.ip_with_port.clone() {
        let mut service_detector = ServiceDetector::new();
        service_detector.set_dst_ip(ip);
        service_detector.set_open_ports(result.get_open_ports(ip));
        println!("{}", service_detector.scan(Some(PortDatabase::default())));
    }
    println!("time -> {}/s", time.elapsed().as_secs_f64());
    Ok(())
}
