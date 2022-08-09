#![cfg(feature="sync")]
use e_libscanner::sync_scan;
use e_libscanner::Opts;
use std::thread;
fn main() -> Result<(), String> {
    // more command information use: -h
    let mut scanner = Opts::new(Some(&[
        "e-libscanner",
        "--ips",
        "192.168.1.0/24",
        "192.168.2-3.1-10",
        "baidu.com",
        "--model",
        "sync",
        "--scan",
        "Icmp",
        "--no-gui",
        "--",
        "-AS",
    ]))?
    .init()?
    .downcast::<sync_scan::Scanner>()
    .unwrap();
    let rx = scanner.get_progress_receiver();
    // Run scan
    let handle = thread::spawn(move || scanner.scan(None));
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
