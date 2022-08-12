use crate::datas::{ScanResultType, DATAS};
use futures_lite::future;
use iced::{
    button, pick_list, text_input, Alignment, Button, Column, Element, Length, PickList,
    ProgressBar, Row, Subscription, Text,
};
use iced_native::subscription;
use e_libscanner::frame::result::PortInfo;
use e_libscanner::service::{PortDatabase, ServiceDetector};
use e_libscanner::{async_scan, dns, os, service, sync_scan, traceroute, Opts, ScanModelType};
use std::collections::HashMap;
use std::hash::Hash;
use std::net::{IpAddr, SocketAddr};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use super::styles;

/// progress model
#[derive(Debug, Clone)]
pub(crate) enum Progress {
    Started,
    Advanced(f32, f32, usize),
    Finished,
    Errored,
}
/// progress state model 
#[derive(Debug)]
pub(crate) enum State {
    Ready(String),
    Starting {
        model: ScanModelType,
        total: usize,
        value: f32,
        rx: Option<Arc<Mutex<Receiver<SocketAddr>>>>,
    },
    Finished,
}
/// command model of message
#[derive(Debug, Clone)]
pub(crate) enum CmdMessage {
    Input(String),
    ExampleSelected(ExampleCmdList),
    Scan(ScanMessage),
    ScanSubmit,
}
/// command model
#[derive(Default, Debug, Clone)]
pub(crate) struct Cmd {
    scan: Scan,
    input: CmdInput,
    example_list: pick_list::State<ExampleCmdList>,
    example_selected: Option<ExampleCmdList>,
}

/// command input model
#[derive(Debug, Default, Clone)]
pub(crate) struct CmdInput {
    pub(crate) state: text_input::State,
    pub(crate) value: String,
}
/// command example model
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExampleCmdList {
    Host,
    Port,
    AsyncHost,
    AsyncPort,
    Os,
    Service,
    Traceroute,
    Dns,
    Version,
    Help,
}

// A simple host scan;
// -- -AS: Arp spoofing
const HOST_TEMPLATE: [&'static str; 11] = [
    "net",
    "-i",
    "192.168.80.0/24",
    "192.168.0-1.1-100",
    "baidu.com",
    "-m",
    "Sync",
    "-s",
    "Icmp",
    "--",
    "-AS",
];
// A simple ports scan;
const PORT_TEMPLATE: [&'static str; 15] = [
    "net",
    "-i",
    "127.0.0.1",
    "baidu.com",
    "-p",
    "80",
    "8000",
    "3389",
    "20-30",
    "-m",
    "Sync",
    "-s",
    "TcpConnect",
    "--",
    "-AS",
];
// A simple async ports scan;
const ASYNC_PORT_TEMPLATE: [&'static str; 16] = [
    "net",
    "-i",
    "127.0.0.1",
    "192.168.80.1-10",
    "baidu.com",
    "-p",
    "80",
    "8000",
    "3389",
    "20-30",
    "-m",
    "Async",
    "-s",
    "TcpConnect",
    "--",
    "-AS",
];
// A simple async ports scan;
const ASYNC_HOST_TEMPLATE: [&'static str; 10] = [
    "net",
    "--ips",
    "127.0.0.1",
    "baidu.com",
    "--model",
    "Async",
    "--scan",
    "Icmp",
    "--",
    "-AS",
];
// A simple os fingerprint probe scan;
const OS_TEMPLATE: [&'static str; 13] = [
    "net",
    "-i",
    "127.0.0.1",
    "baidu.com",
    "-p",
    "80",
    "135",
    "554",
    "8000",
    "-m",
    "Os",
    "--",
    "-AS",
];
// A simple service scan
const SERVICE_TEMPLATE: [&'static str; 11] = [
    "net",
    "--ips",
    "baidu.com",
    "--ports",
    "80",
    "--model",
    "Service",
    "--scan",
    "Tcpsyn",
    "--",
    "-AS",
];
// A simple help
const HELP_TEMPLATE: [&'static str; 2] = ["net", "-h"];
// A simple parse DNS
const DNS_TEMPLATE: [&'static str; 7] = [
    "net",
    "--ips",
    "localhost",
    "127.0.0.1",
    "baidu.com",
    "--model",
    "dns",
];
// A simple trace route
const TRACEROUTE_TEMPLATE: [&'static str; 5] =
    ["net", "--ips", "baidu.com", "--model", "traceroute"];
// A simple version
const VERSION_TEMPLATE: [&'static str; 2] = ["net", "--version"];

// A Help template content
const HELP_TEMPLATE_CONTENT: &'static str = r#"USAGE:
net [FLAGS] [OPTIONS] [-- <command>...]

FLAGS:
-h, --help       Prints help information
    --no-gui     no gui window
-V, --version    Prints version information
-v, --verbose    Verbose mode (-v, -vv, -vvv, etc.)

OPTIONS:
-i, --ips <ips>...               host list; example: "192.168.1.1", "192.168.1.0/24", "192.168.8-9.80-100",
                                 "baidu.com"
-p, --ports <ports>...           port list; Example: 80,443,8080,100-1000
    --rate <rate>                Packet sending interval(0 for unlimited); default is 0 [default: 0]
    --scripts <scripts>          scripts [default: default]  [possible values: None, Default, Custom]
    --src-ip <src-ip>            use it ip match network interface of hardware; [default: ]
-t, --timeout <timeout>          The timeout in milliseconds before a port is assumed to be closed; default is
                                 3_600_000ms [default: 3600000]
-m,  --model <type-model>    scan type; [ Sync, Async, Os, Service, Dns, Traceroute ]; default: sync [default: none]
                                 [possible values: Sync, Async, Os, Service, Dns, Traceroute, None]
-s,  --scan <type-scan>      send type; [ Icmp, TcpConnect, Udp, Tcp, TcpSyn ]; default: None; [default: none]
                                 [possible values: None, Icmp, TcpConnect, Udp, Tcp, TcpSyn]
    --wait-time <wait-time>      Waiting time after packet sending task is completed; default is 3000ms [default:
                                 3000]

ARGS:
<command>...    Extend command; example: e_libscanner --ips baidu.com 192.168.1.0/24 --model sync --scan
                icmp --no-gui -- -AS commads: -- -AS: ARP Spoofing;"#;

// A Version template content
const VERSION_TEMPLATE_CONTENT: &'static str = "VERSION = 0.1.0;";

impl ExampleCmdList {
    pub(crate) fn get_template(&self) -> String {
        let handle = |x: &[&'static str]| x.iter().map(|x| format!("{} ", x)).collect::<String>();
        match self {
            ExampleCmdList::Host => handle(&HOST_TEMPLATE),
            ExampleCmdList::Port => handle(&PORT_TEMPLATE),
            ExampleCmdList::AsyncHost => handle(&ASYNC_HOST_TEMPLATE),
            ExampleCmdList::AsyncPort => handle(&ASYNC_PORT_TEMPLATE),
            ExampleCmdList::Os => handle(&OS_TEMPLATE),
            ExampleCmdList::Service => handle(&SERVICE_TEMPLATE),
            ExampleCmdList::Traceroute => handle(&TRACEROUTE_TEMPLATE),
            ExampleCmdList::Dns => handle(&DNS_TEMPLATE),
            ExampleCmdList::Version => handle(&VERSION_TEMPLATE),
            ExampleCmdList::Help => handle(&HELP_TEMPLATE),
        }
    }
}
impl Default for ExampleCmdList {
    fn default() -> Self {
        Self::Port
    }
}
impl ExampleCmdList {
    pub(crate) const ALL: [Self; 10] = [
        Self::Host,
        Self::Port,
        Self::AsyncHost,
        Self::AsyncPort,
        Self::Os,
        Self::Service,
        Self::Traceroute,
        Self::Dns,
        Self::Version,
        Self::Help,
    ];
}
impl std::fmt::Display for ExampleCmdList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ExampleCmdList::Host => "主机扫描",
                ExampleCmdList::Port => "端口扫描",
                ExampleCmdList::AsyncHost => "异步主机扫描",
                ExampleCmdList::AsyncPort => "异步端口扫描",
                ExampleCmdList::Os => "指纹扫描",
                ExampleCmdList::Service => "服务扫描",
                ExampleCmdList::Traceroute => "跟踪路由",
                ExampleCmdList::Dns => "动态域名解析",
                ExampleCmdList::Version => "软件版本",
                ExampleCmdList::Help => "更多命令",
            }
        )
    }
}
/// 扫描消息体模块
#[derive(Debug, Clone)]
pub(crate) enum ScanMessage {
    Ready(String),
    Stop,
    StartProgressed((usize, Progress)),
}

/// 扫描按钮触发模块
#[derive(Debug, Clone)]
pub(crate) enum ScanState {
    Ready {
        button: button::State,
    },
    Starting {
        cmd: String,
        progress: f32,
        value: f32,
        total: usize,
        button: button::State,
    },
    Finished {
        button: button::State,
    },
}
/// scan model
#[derive(Debug, Clone)]
pub(crate) struct Scan {
    id: usize,
    state: ScanState,
}
impl Default for Scan {
    fn default() -> Self {
        Self {
            id: 0,
            state: ScanState::Ready {
                button: button::State::new(),
            },
        }
    }
}

impl Scan {
    pub(crate) fn update(&mut self, message: ScanMessage) {
        match message {
            // init state
            ScanMessage::Ready(cmd) => {
                if !cmd.is_empty() {
                    self.state = ScanState::Starting {
                        cmd,
                        progress: 0.01,
                        value: 0.01,
                        total: 0,
                        button: button::State::new(),
                    };
                }
            }
            // stop command
            ScanMessage::Stop => {
                log::info!("Trying stop scan {:?}", self.state);
                DATAS.scan.set_stop(true);
                DATAS.scan.waiting_for_lock();
                self.state = ScanState::Ready {
                    button: button::State::new(),
                };
            }
            // scanning then handle progress
            ScanMessage::StartProgressed((_id, start_progressed)) => {
                if let ScanState::Starting {
                    progress,
                    value,
                    total,
                    ..
                } = &mut self.state
                {
                    match start_progressed {
                        Progress::Started => {
                            *progress = 0.0;
                        }
                        Progress::Advanced(p, v, t) => {
                            *progress = p;
                            *value = v;
                            *total = t;
                        }
                        Progress::Finished => {
                            log::debug!(
                                "Progress finished scan count[{}]",
                                DATAS.scan.get_result_len()
                            );
                            self.state = ScanState::Finished {
                                button: button::State::new(),
                            }
                        }
                        Progress::Errored => {
                            log::error!("Progress Errored {:?}", self.state);
                            self.state = ScanState::Ready {
                                button: button::State::new(),
                            }
                        }
                    }
                } else {
                    self.state = ScanState::Ready {
                        button: button::State::new(),
                    }
                }
            }
        }
    }
    pub(crate) fn subscription(&self) -> Subscription<ScanMessage> {
        match &self.state {
            ScanState::Starting { cmd, .. } => {
                // subscription task
                scan_run(self.id, cmd).map(ScanMessage::StartProgressed)
            }
            _ => Subscription::none(),
        }
    }
}

impl Cmd {
    pub(crate) fn subscription(&self) -> Subscription<CmdMessage> {
        self.scan.subscription().map(CmdMessage::Scan)
    }
    pub(crate) fn update(&mut self, ctx: CmdMessage) {
        match ctx {
            CmdMessage::Input(value) => self.input.value = value,
            CmdMessage::ExampleSelected(cmd) => {
                self.input.value = cmd.get_template();
                self.example_selected = Some(cmd);
            }
            CmdMessage::Scan(ctx) => {
                self.scan.update(ctx);
            }
            CmdMessage::ScanSubmit => match self.scan.state {
                ScanState::Ready { .. } | ScanState::Finished { .. } => self
                    .scan
                    .update(ScanMessage::Ready(self.input.value.clone())),
                ScanState::Starting { .. } => self.scan.update(ScanMessage::Stop),
            },
        }
    }
    pub(crate) fn view(&mut self) -> Element<'_, CmdMessage> {
        // command input ui
        let input = text_input::TextInput::new(
            &mut self.input.state,
            "请输入命令",
            &mut self.input.value,
            CmdMessage::Input,
        )
        .padding(6)
        .width(Length::Fill)
        .size(14)
        .style(styles::TextInput::Primary)
        .on_submit(CmdMessage::ScanSubmit);
        // example list ui
        let plist = PickList::new(
            &mut self.example_list,
            &ExampleCmdList::ALL[..],
            self.example_selected.clone(),
            CmdMessage::ExampleSelected,
        )
        .text_size(14)
        .placeholder("示例");
        // progress ui
        let (current_progress, finished_total, total) = match &self.scan.state {
            ScanState::Ready { .. } => (0.0, 0.0, 0),
            ScanState::Starting {
                progress,
                value,
                total,
                ..
            } => (*progress, *value, *total),
            ScanState::Finished { .. } => (100.0, 0.0, 0),
        };
        // scan button ui
        let scanbutton = match &mut self.scan.state {
            ScanState::Ready { button } => Button::new(button, Text::new("扫描").size(15))
                .on_press(CmdMessage::Scan(ScanMessage::Ready(
                    self.input.value.clone(),
                ))),
            ScanState::Starting { button, .. } => Button::new(button, Text::new("停止").size(15))
                .on_press(CmdMessage::Scan(ScanMessage::Stop)),
            ScanState::Finished { button } => Button::new(button, Text::new("扫描").size(15))
                .on_press(CmdMessage::Scan(ScanMessage::Ready(
                    self.input.value.clone(),
                ))),
        };
        // result length
        let result_len = DATAS.scan.get_result_len();
        // progress bar ui
        let progressbar = Row::new()
            .push(ProgressBar::new(0.0..=100.0, current_progress).height(Length::Units(18)))
            .push(
                Text::new(if current_progress > 0.0 || result_len > 0 {
                    format!(
                        "{}%|[{}/{}]|L-{}|{:.1}/s",
                        current_progress as u32,
                        finished_total as u32,
                        total,
                        result_len,
                        DATAS.scan.get_finished_time()
                    )
                } else {
                    "0%|[0/0]|L-0|0.0/s".to_owned()
                })
                .size(15),
            );

        Column::new()
            .spacing(3)
            .push(progressbar)
            .push(
                Row::new()
                    .align_items(Alignment::Center)
                    .spacing(5)
                    .push(input)
                    .push(scanbutton)
                    .push(plist),
            )
            .into()
    }
}
///  scanning
pub(crate) fn scan_run<I: 'static + Hash + Copy + Send + Sync, T: ToString>(
    id: I,
    cmd: T,
) -> Subscription<(I, Progress)> {
    subscription::unfold(id, State::Ready(cmd.to_string()), move |state| {
        scan_runing(id, state)
    })
}
/// scanning
async fn scan_runing<I: Copy>(id: I, state: State) -> (Option<(I, Progress)>, State) {
    let mut process_state = Progress::Errored;
    match state {
        State::Ready(cmd) => {
            let cmds = cmd.split_ascii_whitespace().collect::<Vec<&str>>();
            let opts = if cmds.contains(&"-h") || cmds.contains(&"--help") {
                Opts::new(Some(&["net", "--", "help"]))
            } else if cmds.contains(&"--version") {
                Opts::new(Some(&["net", "--", "version"]))
            } else {
                Opts::new(Some(cmds))
            }
            .unwrap_or(Opts::new(Some(&["net", "--", "help"])).unwrap());

            let mut total = 0;
            let mut rx = None;
            match opts.init() {
                Ok(pointer) => match opts.model {
                    ScanModelType::Sync => match pointer.downcast::<sync_scan::Scanner>() {
                        Ok(mut scanner) => {
                            total = scanner.len();
                            rx = Some(scanner.get_progress_receiver());
                            // Run scan
                            thread::spawn(move || {
                                DATAS.scan.set_locked(true);
                                let pstop = Arc::clone(&DATAS.scan.stop);
                                let result = scanner.scan(Some(pstop));
                                if result.ips.len() > 0 || result.ip_with_port.len() > 0 {
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
                                    DATAS.scan.history.write().unwrap().push((
                                        data.clone(),
                                        chrono::Local::now().timestamp_millis(),
                                    ));
                                    *DATAS.scan.results.lock().unwrap() = data;
                                }
                                DATAS.scan.set_locked(false);
                                DATAS.scan.set_stop(false);
                                DATAS.scan.set_finished_time(result.scan_time.as_secs_f64());
                            });
                            process_state = Progress::Started;
                        }
                        Err(e) => log::error!("{:?}", e),
                    },
                    ScanModelType::Async => match pointer.downcast::<async_scan::Scanner>() {
                        Ok(mut scanner) => {
                            total = scanner.len();
                            rx = Some(scanner.get_progress_receiver());
                            DATAS.scan.set_locked(true);
                            // Run async scan
                            thread::spawn(move || {
                                future::block_on(async {
                                    let pstop = Arc::clone(&DATAS.scan.stop);
                                    let result = scanner.scan(Some(pstop)).await;
                                    if result.ips.len() > 0 || result.ip_with_port.len() > 0 {
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
                                        DATAS.scan.history.write().unwrap().push((
                                            data.clone(),
                                            chrono::Local::now().timestamp_millis(),
                                        ));
                                        *DATAS.scan.results.lock().unwrap() = data;
                                    }
                                    DATAS.scan.set_locked(false);
                                    DATAS.scan.set_stop(false);
                                    DATAS.scan.set_finished_time(result.scan_time.as_secs_f64());
                                })
                            });
                            process_state = Progress::Started;
                        }
                        Err(e) => log::error!("{:?}", e),
                    },
                    ScanModelType::Os => match pointer.downcast::<os::Scanner>() {
                        Ok(mut scanner) => {
                            DATAS.scan.set_locked(true);
                            thread::spawn(move || {
                                let time = Instant::now();
                                let result = scanner.scan(None);
                                if result.len() > 0 {
                                    let data = ScanResultType::Os(result);
                                    DATAS.scan.history.write().unwrap().push((
                                        data.clone(),
                                        chrono::Local::now().timestamp_millis(),
                                    ));
                                    *DATAS.scan.results.lock().unwrap() = data;
                                }
                                DATAS.scan.set_locked(false);
                                DATAS.scan.set_stop(false);
                                DATAS.scan.set_finished_time(time.elapsed().as_secs_f64());
                            })
                            .join()
                            .unwrap();
                            process_state = Progress::Finished;
                        }
                        Err(e) => log::error!("{:?}", e),
                    },
                    ScanModelType::Service => match pointer.downcast::<service::Scanner>() {
                        Ok(mut scanner) => {
                            rx = Some(scanner.get_progress_receiver());
                            total = scanner.len();
                            DATAS.scan.set_locked(true);
                            thread::spawn(move || {
                                let time = Instant::now();
                                let pstop = Arc::new(Mutex::new(false));
                                let result = scanner.scan(Some(pstop));
                                let mut service_result = vec![];
                                for (ip, _ports) in result.ip_with_port.clone() {
                                    let mut service_detector = ServiceDetector::new();
                                    service_detector.set_dst_ip(ip);
                                    service_detector.set_open_ports(result.get_open_ports(ip));
                                    service_result
                                        .push(service_detector.scan(Some(PortDatabase::default())));
                                }
                                if service_result.len() > 0 {
                                    let data = ScanResultType::Service(service_result);
                                    DATAS.scan.history.write().unwrap().push((
                                        data.clone(),
                                        chrono::Local::now().timestamp_millis(),
                                    ));
                                    *DATAS.scan.results.lock().unwrap() = data;
                                }
                                DATAS.scan.set_locked(false);
                                DATAS.scan.set_stop(false);
                                DATAS.scan.set_finished_time(time.elapsed().as_secs_f64());
                            });
                            process_state = Progress::Started;
                        }
                        Err(e) => log::error!("{:?}", e),
                    },
                    ScanModelType::Dns => match pointer.downcast::<dns::DnsResults>() {
                        Ok(scanner) => {
                            let data = ScanResultType::Dns(*scanner);
                            DATAS
                                .scan
                                .history
                                .write()
                                .unwrap()
                                .push((data.clone(), chrono::Local::now().timestamp_millis()));
                            *DATAS.scan.results.lock().unwrap() = data;
                            process_state = Progress::Finished;
                        }
                        Err(e) => log::error!("{:?}", e),
                    },
                    ScanModelType::Traceroute => {
                        match pointer.downcast::<traceroute::Tracert>() {
                            Ok(scanner) => {
                                total = scanner.len();
                                DATAS.scan.set_locked(true);
                                // Run scan
                                thread::spawn(move || {
                                    let time = Instant::now();
                                    let pstop = Arc::clone(&DATAS.scan.stop);
                                    let result = scanner.scan(Some(pstop));
                                    if result.len() > 0 {
                                        let data = ScanResultType::Tracert(result);
                                        DATAS.scan.history.write().unwrap().push((
                                            data.clone(),
                                            chrono::Local::now().timestamp_millis(),
                                        ));
                                        *DATAS.scan.results.lock().unwrap() = data;
                                    }
                                    DATAS.scan.set_locked(false);
                                    DATAS.scan.set_stop(false);
                                    DATAS.scan.set_finished_time(time.elapsed().as_secs_f64());
                                })
                                .join()
                                .unwrap();
                                process_state = Progress::Finished;
                            }
                            Err(e) => log::error!("{:?}", e),
                        }
                    }
                    ScanModelType::None => {
                        if opts.command.contains(&String::from("help")) {
                            *DATAS.scan.results.lock().unwrap() =
                                ScanResultType::None(Some(HELP_TEMPLATE_CONTENT.to_owned()));
                        } else if opts.command.contains(&String::from("version")) {
                            *DATAS.scan.results.lock().unwrap() =
                                ScanResultType::None(Some(VERSION_TEMPLATE_CONTENT.to_owned()));
                        }
                        process_state = Progress::Finished;
                    }
                },
                Err(e) => log::error!("{:?}", e),
            }
            (
                Some((id, process_state)),
                State::Starting {
                    model: opts.model,
                    total,
                    value: 0.0,
                    rx,
                },
            )
        }
        // handle process
        State::Starting {
            model,
            total,
            value,
            rx,
        } => {
            let value = value + 1.0;
            if total as f32 >= value {
                if let Some(ref rx_parse) = rx {
                    if let Ok(_socket_addr) = Arc::clone(&rx_parse).lock().unwrap().recv() {
                        let res = (value / total as f32) * 100.0;
                        // log::debug!(
                        //     "Scaning progress {} [{} / {} * 100 = {}]",
                        //     _socket_addr,
                        //     value,
                        //     total,
                        //     res
                        // );
                        (
                            Some((id, Progress::Advanced(res, value, total))),
                            State::Starting {
                                model,
                                total,
                                value,
                                rx,
                            },
                        )
                    } else {
                        (Some((id, Progress::Errored)), State::Finished)
                    }
                } else {
                    (Some((id, Progress::Errored)), State::Finished)
                }
            } else {
                DATAS.scan.waiting_for_lock();
                (Some((id, Progress::Finished)), State::Finished)
            }
        }
        State::Finished => {
            // We do not let the stream die, as it would start a
            // new download repeatedly if the user is not careful
            // in case of errors.
            iced::futures::future::pending().await
        }
    }
}
