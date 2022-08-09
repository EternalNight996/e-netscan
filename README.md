### 📄 [中文](README.md)  | 📄  [English](EN.md)

# ⚡ 这是什么?

一个支持跨平台图形可视化网络扫描工具. **能够快速扫描端口与主机，并能够跨网段完成任务**

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

# 🔭 如何打包应用?
### 为什么我们需要打包应用？
因为我们正常编译的.exe应用，在复杂的项目中，需要依赖各式各样的、资源[dll、lib、png、ico...]、环境变量、动态安装卸载修复更新等！
所以我们需要一个打包管理工具，能解决这些问题，当然也可以用如elector与tauri这样的集成跨平台GUI框架，如果普通的GUI那么这些工具将对你很有帮助！

#### Windows
- 下载E-NetScan
[Install-0.1.0.exe](https://github.com/EternalNight996/e-netscan/releases/download/v0.1.0/Installe.exe)

- 首先我们需要一个打包工具;
[微软总结第三方打包工具](https://docs.microsoft.com/zh-cn/windows/msix/desktop/desktop-to-uwp-third-party-installer)
本项目用的是[Advanced Installer](https://www.advancedinstaller.com/), 自行下载安装！

- 打包项目 `E-NetScan.api`;
```
# setup 1; compile project
cargo build --release
# setup 2; open E-NetScan.api from Advanced Installer;
# setup 3; build E-NetScan.api;
# project will output to E-NetScan-SetupFiles/..
```

## `💡!重要：`
#### Windows系统环境上构建有三个要求
您必须使用使用MSVC工具链的Rust版本
您必须安装[WinPcap](https://www.winpcap.org/)或[npcap](https://nmap.org/npcap/)（使用[WinPcap](https://www.winpcap.org/) 4.1.3版进行测试）（如果使用[npcap](https://nmap.org/npcap/)，请确保使用“在[WinPcap](https://www.winpcap.org/) API兼容模式下安装[npcap](https://nmap.org/npcap/)”）
你必须把它放在包里。[WinPcap](https://www.winpcap.org/)开发者包中的lib位于该存储库根目录中名为lib的目录中。或者，您可以使用%LIB%/$Env:LIB环境变量中列出的任何位置。对于64位工具链，它位于WpdPack/Lib/x64/Packet中。对于32位工具链，它位于WpdPack/lib/Packet.lib中。
#### 如果你不想用Advanced Installer打包部署环境，则运行以下命令！
```
# install npcap server
./e-netscan/scripts/npcap-1.70.exe
# Build Operating environment use of bat
./Install.bat
```

# 🚀 快速运行
```
# 编译到 target/release [e-netscan, e-netscan-gui]
cargo build --release
# 启动可视化界面
e-netscan-gui

# 命令行扫描主机 192.168.80.1 baidu.com 192.168.1.1-254 范围主机
e-netscan -i 192.168.80.1 baidu.com 192.168.1.1/24 -m sync
# 命令行扫描端口 192.168.80.1 baidu.com [80, 20..30]端口 
e-netscan -i 192.168.80.1 baidu.com -p 80 20-30 -m sync
# 命令行ARP欺骗跨网段扫描
e-netscan -i 192.168.1.1/24 -m sync -- -AS
# 命令行异步扫描端口
e-netscan -i 192.168.80.1 baidu.com -p 80 20-30 -m async
# 命令行指纹扫描
e-netscan -i 192.168.80.1 baidu.com -m os
# 命令行服务扫描
e-netscan -i localhost baidu.com -p 80 8000 -m service
# 命令行DNS解析
e-netscan -i localhost baidu.com 114.114.114.114 -m dns
# 命令行路由跟踪
e-netscan -i baidu.com -m traceroute
# 命令行设置打印等级: -vvvvv[warn, error, info, debug, tracert]
e-netscan -i 192.168.1.1/24 -vvv -m sync
# 帮助
e-netscan -h
# 版本
e-netscan --version
```

# 🙋 想二次开发？
✨[e-libscanner](https://github.com/EternalNight996/e-libscanner) : 本项目扫描所依赖的API库

# 🤔 为什么需要E-NetScan?
起初是想完成一个跨网络扫描项目，帮助自己完成一些工作，参考许多开源项目,但这些项目多少有些缺陷并不满足自己需求，所以有了E-NetScan。

# 💡提示?
本程序GUI用的[iced](https://github.com/iced-rs/iced)开发完发现iced存在有很大兼容性问题！
如果想二次开发则推荐使用其他GUI!
- iced_glow supporting OpenGL 2.1+ and OpenGL ES 2.0+
- iced_wgpu supporting Vulkan, Metal and DX12
- 没有相对应的GPU则无法运行GUI！并且只能二选一