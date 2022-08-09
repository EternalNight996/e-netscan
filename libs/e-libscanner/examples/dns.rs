use e_libscanner::{dns::DnsResults, Opts};
fn main() -> Result<(), String> {
    // more command information use: -h
    let opts = Opts::new(Some(&[
        "e-libscanner",
        "--ips",
        "baidu.com",
        "127.0.0.1",
        "localhost",
        "--model",
        "dns",
    ]))?
    .init()?
    .downcast::<DnsResults>();
    match opts {
        Ok(opt) => {
            let mut n = 0i32;
            for r in *opt {
                n += 1;
                eprintln!("{}- src[ {} ] parse [{:?}]", n, r.src, r.result);
            }
        }
        Err(e) => panic!("{:?}", e),
    }
    Ok(())
}
