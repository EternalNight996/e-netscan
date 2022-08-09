###  📄  [Chinese](README.md)| 📄   [English](EN.md)

# ⚡  what is it?
A cross platform graphic visualization network scanning tool** Can quickly scan ports and hosts and complete tasks across network segments**

###  🤔  Support [platform | function]
+ [x] platform
    - [x] windows[10]
    - [ ] linux[ubuntu、kali]
+ [x] Skill
    - [x] fingerprint scanning
    - [x] asynchronous scan
    - [x] synchronous scanning
    - [x] service scan
    - [x] DNS resolution
    - [x] route tracing
    - [x] ARP Spoofing


# 🔭  How to package the application?
### Why do we need to package applications?
Because we normally compile. Exe applications, in complex projects, we need to rely on various resources [DLL, lib, PNG, ICO...], environment variables, dynamic installation, uninstallation, repair and update, etc!
So we need a package management tool to solve these problems. Of course, we can also use an integrated cross platform GUI framework such as elector and Tauri. If it is an ordinary GUI, these tools will be very helpful to you!
#### Windows
- Download E-NetScan
[Install-0.1.0.exe](https://github.com/EternalNight996/e-netscan/releases/download/v0.1.0/Installe.exe)

- First, we need a packaging tool;
[Microsoft summarizes third-party packaging tools](https://docs.microsoft.com/zh-cn/windows/msix/desktop/desktop-to-uwp-third-party-installer)
This project uses [advanced installer](https://www.advancedinstaller.com/), download and install by yourself!
- Package project ` e-netscan. API ';
```
# setup 1;  compile project
cargo build --release
# setup 2;  open E-NetScan. api from Advanced Installer;
# setup 3;  build E-NetScan.api;
# project will output to E-NetScan-SetupFiles/..
```

## ` 💡! Important:`
####There are three requirements for building on the windows system environment:
You must use the rust version using the MSVC toolchain
You must install [WinPcap]（ https://www.winpcap.org/ ）Or [npcap]（ https://nmap.org/npcap/ ）(using [WinPcap]（ https://www.winpcap.org/ ）Version 4.1.3) (if using [npcap]（ https://nmap.org/npcap/ ）, please make sure to use "in [WinPcap]（ https://www.winpcap.org/ ）Install [npcap] in API compatibility mode（ https://nmap.org/npcap/ )”）
You must put it in your bag. [WinPcap]( https://www.winpcap.org/ ）The Lib in the developer package is located in the directory named Lib in the root directory of the repository. Alternatively, you can use any location listed in the% lib% / $env: lib environment variable. For the 64 bit toolchain, it is located in wpdpack / lib / x64 / packet. For the 32-bit toolchain, it is located in wpdpack / lib / packet.lib.

#### If you do not want to package the deployment environment with advanced installer, run the following command!
```
# install npcap server
./e-netscan/scripts/npcap-1.70.exe
# Build Operating environment use of bat
./Install.bat
```

# 🚀  Fast running
```
#Compile to target / release [e-netscan, e-netscan GUI]
cargo build --release
#Start visual interface
e-netscan-gui
#Command line scan host 192.168.80.1 Baidu COM 192.168.1.1-254 range host
e-netscan -i 192.168.80.1 baidu. com 192.168.1.1/24 -m sync
#Command line scan port 192.168.80.1 Baidu Com [80, 20.. 30] port
e-netscan -i 192.168.80.1 baidu. com -p 80 20-30 -m sync
#Command line ARP Spoofing cross network segment scanning
e-netscan -i 192.168.1.1/24 -m sync -- -AS
#Command line asynchronous scan port
e-netscan -i 192.168.80.1 baidu. com -p 80 20-30 -m async
#Command line fingerprint scan
e-netscan -i 192.168.80.1 baidu. com -m os
#Command line service scan
e-netscan -i localhost baidu. com -p 80 8000 -m service
#Command line DNS resolution
e-netscan -i localhost baidu. com 114.114.114.114 -m dns
#Command line route tracing
e-netscan -i baidu. com -m traceroute
#Command line setting print level: - vvvvv [warn, error, info, debug, trace]
e-netscan -i 192.168.1.1/24 -vvv -m sync
#Help
e-netscan -h
#Version
e-netscan --version
```

# 🙋  Want secondary development?
✨ [e-libscanner]( https://github.com/EternalNight996/e-libscanner ）: API library that this project depends on for scanning

# 🤔  Why do I need e-netscan?
At first, I wanted to complete a cross network scanning project to help me complete some work. I referred to many open source projects, but these projects have some defects that do not meet my needs, so I have e-netscan.

# 💡 Tips?
[iced] for GUI of this program（ https://github.com/iced-rs/iced ）After development, I found that iced has a great compatibility problem!
Other GUIs are recommended for secondary development!
- iced_ glow supporting OpenGL 2.1+ and OpenGL ES 2.0+
- iced_ wgpu supporting Vulkan, Metal and DX12
-GUI cannot be run without corresponding GPU! And you can only choose one of two