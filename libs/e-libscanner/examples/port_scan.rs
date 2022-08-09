#![cfg(feature="sync")]
use e_libscanner::sync_scan;
use e_libscanner::Opts;
use std::thread;
fn main() -> Result<(), String> {
    // more command information use: -h
    let mut scanner = Opts::new(Some(&[
        "e-libscanner",
        "--ips",
        "192.168.96.101",
        "192.168.96.2",
        "192.168.80.3",
        "192.168.80.31",
        "--ports",
        "8000",
        "8080",
        "20-30",
        "80",
        "--model",
        "sync",
        "--scan",
        "TcpConnect",
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
    for (ip, ports) in result.ip_with_port {
        println!("{}", ip);
        for port in ports {
            println!("{:?}", port);
        }
    }
    println!("Scan Time: {:?}", result.scan_time);
    Ok(())
}
