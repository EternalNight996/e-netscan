
### 📄 [中文](README.md)  | 📄  [English](English.md)

# ⚡ 这是什么?

这是一个扫描集成接口，底层是通过抓包方式扫描. **能够快速扫描端口与主机，并能够跨网段完成任务**


### 🤔 支持[平台|功能]
+ [x] 平台
    - [x] windows[10]
    - [ ] linux[ubuntu、kali]
+ [x] 功能
    - [x] 指纹扫描
    - [x] 异步扫描
    - [x] 同步扫描
    - [x] 服务扫描
    - [x] DNS解析
    - [x] 路由追踪
    - [x] ARP欺骗

# ✨ 分支
- async = ["async-io", "futures-lite", "futures"]
- service = ["native-tls", "sync"]
- os = []
- sync = []
- full = ["async", "sync", "os", "service"]
- default = ["sync"]

# 📖 示例
[异步主机扫描](examples/async_host_scan.rs)
[异步端口扫描](examples/async_port_scan.rs)
[主机扫描](examples/host_scan.rs)
[端口扫描](examples/port_scan.rs)
[指纹扫描](examples/os.rs)
[服务扫描扫描](examples/service_detection.rs)
[DNS解析](examples/dns.rs)
[路由跟踪](examples/tracert.rs)
[命令行API示例](examples/cmd_input.rs)
##### `命令行示例` 
```
e_libscanner -i 192.168.1.1-10 www.baidu.com --model Sync --scan Icmp --no-gui -- -AS
```
## `💡!重要：`
#### Windows系统环境上构建有三个要求
您必须使用使用MSVC工具链的Rust版本
您必须安装[WinPcap](https://www.winpcap.org/)或[npcap](https://nmap.org/npcap/)（使用[WinPcap](https://www.winpcap.org/) 4.1.3版进行测试）（如果使用[npcap](https://nmap.org/npcap/)，请确保使用“在[WinPcap](https://www.winpcap.org/) API兼容模式下安装[npcap](https://nmap.org/npcap/)”）
你必须把它放在包里。[WinPcap](https://www.winpcap.org/)开发者包中的lib位于该存储库根目录中名为lib的目录中。或者，您可以使用%LIB%/$Env:LIB环境变量中列出的任何位置。对于64位工具链，它位于WpdPack/Lib/x64/Packet中。对于32位工具链，它位于WpdPack/lib/Packet.lib中。
```
# 1.安装npcap服务 https://npcap.com/dist/npcap-1.70.exe
setx LIB E:\libs\LIB
# 下载并解压 https://npcap.com/dist/npcap-sdk-1.13.zip
# 将npcap-sdk-1.13\Lib\x64\Packet.lib放到E:\libs\LIB
```

# 🚀 快速运行
```
# 主机/端口扫描
cargo run --example host_scan
cargo run --example port_scan
# 异步扫描
cargo run --example async_host_scan --features="async"
cargo run --example async_port_scan --features="async"
# 指纹扫描
cargo run --example os --features="os"
# 服务扫描
cargo run --example service_detection --features="service"
# dns解析
cargo run --example dns
# 路由跟踪
cargo run --example tracert
```

# 🦊 已运用项目
[E-NetScan](https://github.com/EternalNight996/e-netscan.git): 网络扫描项目（同时支持命令行与跨平台图形化界面）正在开发中。。

# 🔭 为什么需要e-libscanner?
起初是想完成一个跨网络扫描项目，帮助自己完成一些工作，参考许多开源项目,但这些项目多少有些缺陷并不满足自己需求，所以有了e-libscanner。
(处理主机和端口扫描，同时支持域名解析、路由跟踪、指纹扫描、服务扫描、异步扫描、可扩展更多)
底层是通过调用[npcap](https://nmap.org/npcap/)与[WinPcap](https://www.winpcap.org/)抓包服务；
服务api为[libpnet](https://github.com/libpnet/libpnet);

# 🙋 参考项目与资料
✨[RustScan](https://github.com/RustScan/RustScan) :Rust仿nmap扫描库
✨[netscan](https://github.com/shellrow/netscan) :Rust 网络扫描库
✨[libpnet](https://github.com/libpnet/libpnet) 跨平台网络底层库--主要是调用抓包服务([npcap](https://nmap.org/npcap/)与[WinPcap](https://www.winpcap.org/))