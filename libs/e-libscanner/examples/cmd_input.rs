use e_libscanner::Opts;
fn main() -> Result<(), String> {
    // more command information use: -h
    let opts = Opts::new(Some(&[
        "e-libscanner",
        "--ips",
        "192.168.9.8",
        "192.168.1.0/23",
        "192.168.10-11.0-254",
        "--ports",
        "80",
        "22",
        // "--src-ip",
        // "127.0.0.1",
        "--timeout",
        "3000",
        "--wait-time",
        "1000",
        "--rate",
        "0",
        "--model",
        "os",
        "--scan",
        "tcp",
        "--scripts",
        "default",
        "--no-gui",
        "-vvv",
        "--",
        "-AS",
    ]))?
    .init()?;
    println!("{:?}", opts);
    Ok(())
}
