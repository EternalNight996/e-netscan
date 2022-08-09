#![cfg(feature="async")]
use async_io;
use e_libscanner::{async_scan, Opts};
use std::thread;

fn main() -> Result<(), String> {
    // more command information use: -h
    let mut scanner = Opts::new(Some(&[
        "e-libscanner",
        "--ips",
        "192.168.20.0/23",
        "192.168.28-31.1-10",
        "baidu.com",
        "--model",
        "async",
        "--scan",
        "icmp",
        "--no-gui",
    ]))?
    .init()?
    .downcast::<async_scan::Scanner>()
    .unwrap();
    let rx = scanner.get_progress_receiver();
    // Run scan
    let handle = thread::spawn(move || async_io::block_on(async { scanner.scan(None).await }));
    // Print progress
    while let Ok(socket_addr) = rx.lock().unwrap().recv() {
        println!("Check: {}", socket_addr);
    }
    let result = handle.join().unwrap();
    // Print results
    println!("Status: {:?}", result.scan_status);
    println!("UP Hosts:");
    let len = result.ips.len();
    for host in result.ips {
        println!("{:?}", host);
    }
    println!("Scan Time: {:?} count[ {} ]", result.scan_time, len);
    Ok(())
}
