#![cfg(feature="async")]
use e_libscanner::{async_scan, Opts};
use std::thread;

fn main() -> Result<(), String> {
    // more command information use: -h
    let mut scanner = Opts::new(Some(&[
        "e-libscanner",
        "--ips",
        "192.168.80.1",
        "--ports",
        "8000",
        "8080",
        "80",
        "20-26",
        "--model",
        "async",
        "--scan",
        "TcpConnect",
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
    for (ip, ports) in result.ip_with_port {
        println!("{}", ip);
        for port in ports {
            println!("{:?}", port);
        }
    }
    println!("Scan Time: {:?}", result.scan_time);
    Ok(())
}
