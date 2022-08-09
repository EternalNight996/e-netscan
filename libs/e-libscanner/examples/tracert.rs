use e_libscanner::{traceroute::Tracert, Opts};
fn main() -> Result<(), String> {
    let opts = Opts::new(Some(&[
        "e-libscanner",
        "--ips",
        "114.114.114.114",
        "--model",
        "traceroute",
    ]))?
    .init()?
    .downcast::<Tracert>();
    match opts {
        Ok(opt) => {
            let prx = opt.get_progress_receiver();
            let handle = std::thread::spawn(move || {
                while let Ok(msg) = prx.lock().unwrap().recv() {
                    // TODO Something
                    eprintln!("recv {:?}", msg);
                }
            });
            let results = opt.scan(None);
            handle.join().unwrap();
            println!("count result -> {}", results.len());
        }
        Err(e) => panic!("{:?}", e),
    }
    Ok(())
}
