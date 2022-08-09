use sys_info::info::Info;
fn main() {
    {
        let info = Info::new();
        println!("info {:?}", info.get_monitor());
    }
}
