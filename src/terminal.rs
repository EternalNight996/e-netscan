#[cfg(target_feature = "libscan")]
extern crate e_libscanner;
mod configs;
#[path = "./datas.rs"]
mod datas;
mod ui;

use datas::ScanResultType;
use futures_lite::future;
use e_libscanner::{
    async_scan, dns,
    frame::result::PortInfo,
    os,
    service::{self, PortDatabase, ServiceDetector},
    sync_scan, traceroute, Opts, ScanModelType,
};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    thread,
};

const APP_NAME: &'static str = "e-netscan";
/// main app
pub fn main() -> Result<(), String> {
    let opts = Opts::new::<&[&str]>(None).unwrap();
    #[cfg(debug_assertions)]
    let verbose = if opts.verbose > 5 { 5 } else { opts.verbose };
    #[cfg(not(debug_assertions))]
    let verbose = if opts.verbose > 5 { 3 } else { opts.verbose };
    configs::logger::init_logger(APP_NAME, verbose);
    // parse opts command and running
    input_run(opts)
}

/// input parse and run; no gui
pub fn input_run(opts: Opts) -> Result<(), String> {
    match opts.init() {
        Ok(pointer) => match opts.model {
            ScanModelType::Sync => match pointer.downcast::<sync_scan::Scanner>() {
                Ok(mut scanner) => {
                    let total = scanner.len();
                    let rx = scanner.get_progress_receiver();
                    // Run sync scan
                    let handle = thread::spawn(move || {
                        let result = scanner.scan(None);
                        let data = if opts.ports.len() > 0 {
                            let filter = result
                                .ip_with_port
                                .clone()
                                .drain()
                                .filter(|x| x.1.len() > 0)
                                .collect::<HashMap<IpAddr, Vec<PortInfo>>>();
                            ScanResultType::Port(filter)
                        } else {
                            ScanResultType::Host(result.ips.clone())
                        };
                        let s = format!(
                            "\n{}\n Done {}/s",
                            result_to_string(&data).unwrap(),
                            result.scan_time.as_secs_f64()
                        );
                        s
                    });
                    let mut value = 0.0;
                    while let Ok(socket) = rx.lock().unwrap().recv() {
                        value += 1.0;
                        let s = to_process(value, total, socket);
                        out(&s, Some((0, 255, 0)));
                    }
                    out(&handle.join().unwrap(), Some((0, 255, 0)));
                }
                Err(e) => log::error!("{:?}", e),
            },
            ScanModelType::Async => match pointer.downcast::<async_scan::Scanner>() {
                Ok(mut scanner) => {
                    let total = scanner.len();
                    let rx = scanner.get_progress_receiver();
                    // Run async scan
                    let handle = thread::spawn(move || {
                        future::block_on(async {
                            let result = scanner.scan(None).await;
                            let data = if opts.ports.len() > 0 {
                                let filter = result
                                    .ip_with_port
                                    .clone()
                                    .drain()
                                    .filter(|x| x.1.len() > 0)
                                    .collect::<HashMap<IpAddr, Vec<PortInfo>>>();
                                ScanResultType::Port(filter)
                            } else {
                                ScanResultType::Host(result.ips.clone())
                            };
                            let s = format!(
                                "\n{}\n Done {}/s",
                                result_to_string(&data).unwrap(),
                                result.scan_time.as_secs_f64()
                            );
                            s
                        })
                    });
                    let mut value = 0.0;
                    while let Ok(socket) = rx.lock().unwrap().recv() {
                        value += 1.0;
                        let s = to_process(value, total, socket);
                        out(&s, Some((0, 255, 0)));
                    }
                    out(&handle.join().unwrap(), Some((0, 255, 0)));
                }
                Err(e) => log::error!("{:?}", e),
            },
            ScanModelType::Os => match pointer.downcast::<os::Scanner>() {
                Ok(mut scanner) => {
                    // Run os scan
                    let time = std::time::Instant::now();
                    let result = scanner.scan(None);
                    for res in result {
                        let s = format!("{}\n", res.display());
                        out(&s, Some((0, 255, 0)));
                    }
                    let s = format!("\nDone {}/s", time.elapsed().as_secs_f64());
                    out(&s, Some((0, 255, 0)));
                }
                Err(e) => log::error!("{:?}", e),
            },
            ScanModelType::Service => match pointer.downcast::<service::Scanner>() {
                Ok(mut scanner) => {
                    let total = scanner.len();
                    let rx = scanner.get_progress_receiver();
                    // Run sync scan
                    let handle = thread::spawn(move || {
                        let time = std::time::Instant::now();
                        let result = scanner.scan(None);
                        let mut service_result = vec![];
                        for (ip, _ports) in result.ip_with_port.clone() {
                            let mut service_detector = ServiceDetector::new();
                            service_detector.set_dst_ip(ip);
                            service_detector.set_open_ports(result.get_open_ports(ip));
                            service_result
                                .push(service_detector.scan(Some(PortDatabase::default())));
                        }
                        let data = ScanResultType::Service(service_result);
                        let s = format!(
                            "\n{}\n Done {}/s",
                            result_to_string(&data).unwrap(),
                            time.elapsed().as_secs_f64()
                        );
                        s
                    });
                    let mut value = 0.0;

                    while let Ok(socket) = rx.lock().unwrap().recv() {
                        value += 1.0;
                        let s = to_process(value, total, socket);
                        out(&s, Some((0, 255, 0)));
                    }
                    out(&handle.join().unwrap(), Some((0, 255, 0)));
                }
                Err(e) => log::error!("{:?}", e),
            },
            ScanModelType::Dns => match pointer.downcast::<dns::DnsResults>() {
                Ok(scanner) => {
                    let mut n = 0i32;
                    for r in *scanner {
                        n += 1;
                        let s = format!("{}- src[ {} ] parse [{:?}]", n, r.src, r.result);
                        out(&s, Some((0, 255, 0)));
                    }
                }
                Err(e) => log::error!("{:?}", e),
            },
            ScanModelType::Traceroute => {
                match pointer.downcast::<traceroute::Tracert>() {
                    Ok(scanner) => {
                        let prx = scanner.get_progress_receiver();
                        let handle = std::thread::spawn(move || {
                            while let Ok(msg) = prx.lock().unwrap().recv() {
                                // TODO Something
                                let s =
                                    format!("{} {}ms {:?}", msg.id, msg.rtt.as_millis(), msg.addr);
                                out(&s, Some((0, 255, 0)));
                            }
                        });
                        let pstop = Arc::new(Mutex::new(false));
                        let results = scanner.scan(Some(pstop));
                        if let Ok(_) = handle.join() {
                            let s = format!("count result -> {}", results.len());
                            out(&s, Some((0, 255, 0)));
                        }
                    }
                    Err(e) => log::error!("{:?}", e),
                }
            }
            ScanModelType::None => {}
        },
        Err(e) => log::error!("{:?}", e),
    }
    Ok(())
}

#[allow(dead_code)]
fn result_to_string(result: &ScanResultType) -> Result<String, String> {
    Ok(match result {
        ScanResultType::Host(data) => data.iter().map(|x| x.to_string()).collect::<String>(),
        ScanResultType::Port(data) => data
            .iter()
            .map(|x| {
                format!(
                    "{}: [{}]\n",
                    x.0,
                    x.1.iter().map(|xx| xx.to_string()).collect::<String>()
                )
            })
            .collect::<String>(),
        ScanResultType::Os(data) => data.iter().map(|x| x.to_string()).collect::<String>(),
        ScanResultType::Service(data) => {
            data.iter().map(|x| format!("{}\n", x)).collect::<String>()
        }
        ScanResultType::Dns(data) => data.iter().map(|x| x.to_string()).collect::<String>(),
        ScanResultType::Tracert(data) => data.iter().map(|x| x.to_string()).collect::<String>(),
        ScanResultType::None(data) => match data {
            Some(d) => d.clone(),
            None => return Err(String::from("Scan result of type is None.")),
        },
    })
}

fn out(s: &str, _f: Option<(u8, u8, u8)>) {
    #[cfg(not(target_os = "windows"))]
    e_utils::output!(rgb[_f, None] s);
    #[cfg(target_os = "windows")]
    println!("{}", s);
}

fn to_process<I>(value: f32, total: usize, data: I) -> String
where
    I: core::fmt::Debug,
{
    let res = (value / total as f32) * 25.0;
    format!(
        "[{}/{}] [{:?}] {:.0}%{}\r",
        value,
        total,
        data,
        res * 4.0,
        (0..res as usize).map(|_x| "■").collect::<String>(),
    )
}
