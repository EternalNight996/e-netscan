pub mod id;
mod portmap;
use std::collections::HashMap;

use once_cell::sync::Lazy;

pub struct Data {
    pub portmap: HashMap<u16, &'static str>,
}
// 静态数据接口
pub static DATA: Lazy<Data> = Lazy::new(|| Data {
    portmap: portmap::get_tcp_portmap(),
});
