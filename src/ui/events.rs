use super::styles;
use crate::datas::{ScanResultType, DATAS};
use chrono::TimeZone;
use iced::{
    alignment, button, text_input, Alignment, Button, Element, Length, Row, Subscription, Text,
    TextInput, Toggler,
};

use clipboard::{ClipboardContext, ClipboardProvider};
use iced_native::window::Event::CloseRequested;
use iced_native::Event;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// copy to clipboard state
#[derive(Debug, Clone)]
pub enum CopyToClipboardState {
    Scan,
}
impl Default for CopyToClipboardState {
    fn default() -> Self {
        CopyToClipboardState::Scan
    }
}

/// scan result of copy
#[derive(Debug, Clone, Default)]
pub struct ScanResultCopy {
    pub state: button::State,
    pub boolean: bool,
}

/// scan history result of download
#[derive(Debug, Clone)]
pub struct ScanHistoryDownload {
    pub input_state: text_input::State,
    pub input_value: String,
    pub button_state: button::State,
    pub button_boolean: bool,
}
impl Default for ScanHistoryDownload {
    fn default() -> Self {
        let input_value = dirs::desktop_dir()
            .and_then(|x| {
                Some(
                    x.join(format!("{}.txt", get_random_filename(16)))
                        .display()
                        .to_string(),
                )
            })
            .unwrap();
        ScanHistoryDownload {
            input_value,
            input_state: text_input::State::new(),
            button_state: button::State::new(),
            button_boolean: false,
        }
    }
}
/// event occur model
#[derive(Debug, Default)]
pub struct EventOccur {
    pub should_exit: bool,
    log_enable: bool,
    scan_result_copy: ScanResultCopy,
    scan_history_download: ScanHistoryDownload,
}
/// event occur message model
#[derive(Debug, Clone)]
pub enum EventMessage {
    Occurred(iced_native::Event),
    LogOnOff(bool),
    ScanHistoryDownload(String),
    OnScanHistoryDownload,
    Copy(CopyToClipboardState),
    ScanHistoryDownloadSubmit,
}

impl EventOccur {
    pub fn update(&mut self, message: EventMessage) {
        if self.log_enable {
            // saving event log
            log::info!("{:?}", message);
        }
        match message {
            EventMessage::Occurred(event) => match event {
                // right up exit of event
                Event::Window(CloseRequested) => {
                    self.should_exit = true;
                    DATAS.scan.set_stop(true);
                    DATAS.scan.waiting_for_lock();
                }
                _ => {}
            },
            // log onoff of message
            EventMessage::LogOnOff(state) => {
                self.log_enable = state;
            }
            // history download of message
            EventMessage::ScanHistoryDownload(value) => {
                self.scan_history_download.input_value = value
            }
            // Enter button with download button of message
            EventMessage::OnScanHistoryDownload | EventMessage::ScanHistoryDownloadSubmit => {
                if !self.scan_history_download.input_value.is_empty() {
                    // downloaded result
                    if let Err(e) = download_scan_history(&PathBuf::from(
                        &self.scan_history_download.input_value,
                    )) {
                        log::error!("EventMessage::OnScanHistoryDownload | EventMessage::ScanHistoryDownloadSubmit {} ", e);
                    } else {
                        self.scan_history_download.button_boolean = true;
                    };
                }
            }
            // copy to clipboard of message
            EventMessage::Copy(state) => match state {
                CopyToClipboardState::Scan => {
                    self.scan_result_copy.boolean = true;
                    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                    log::debug!("CopyToClipboard {:?}", ctx.get_contents());
                    ctx.set_contents(DATAS.scan.result_to_string().unwrap_or("".to_owned()))
                        .unwrap();
                }
            },
        }
    }
    pub fn subscription(&self) -> Subscription<EventMessage> {
        // subscription event of message
        iced_native::subscription::events().map(EventMessage::Occurred)
    }
    pub fn view(&mut self) -> Element<EventMessage> {
        let enable_event_log = Toggler::new(
            self.log_enable,
            "事件\n日志开关".to_string(),
            EventMessage::LogOnOff,
        )
        .text_size(11)
        .width(Length::Shrink)
        .text_alignment(alignment::Horizontal::Center);

        let copy_on = Button::new(
            &mut self.scan_result_copy.state,
            Text::new(if self.scan_result_copy.boolean {
                "已复制\n扫描结果"
            } else {
                "复制\n扫描结果"
            })
            .size(11)
            .horizontal_alignment(alignment::Horizontal::Center),
        )
        .style(if self.scan_result_copy.boolean {
            styles::Button::Finished
        } else {
            styles::Button::Primary
        })
        .on_press(EventMessage::Copy(CopyToClipboardState::Scan));

        let scan_history_download = TextInput::new(
            &mut self.scan_history_download.input_state,
            "导出扫描历史结果",
            &mut self.scan_history_download.input_value,
            EventMessage::ScanHistoryDownload,
        )
        .size(14)
        .padding(6)
        .on_submit(EventMessage::ScanHistoryDownloadSubmit);

        let scan_history_download_button = Button::new(
            &mut self.scan_history_download.button_state,
            Text::new(if self.scan_history_download.button_boolean {
                "已下载\n历史扫描"
            } else {
                "下载\n历史扫描"
            })
            .size(11)
            .horizontal_alignment(alignment::Horizontal::Center),
        )
        .style(if self.scan_history_download.button_boolean {
            styles::Button::Finished
        } else {
            styles::Button::Primary
        })
        .on_press(EventMessage::OnScanHistoryDownload);

        Row::new()
            .align_items(Alignment::Center)
            .spacing(5)
            .push(enable_event_log)
            .push(copy_on)
            .push(scan_history_download)
            .push(scan_history_download_button)
            .into()
    }
}

/// get random filename
fn get_random_filename(length: usize) -> String {
    let now = chrono::Local::now().format("%Y年%m月%d日");
    let random = e_utils::random!(nanoid length);
    format!("{}_{}", now, random)
}

/// download scan history
fn download_scan_history(path: &PathBuf) -> std::io::Result<()> {
    if DATAS.scan.get_history_len() > 0 {
        match fs::OpenOptions::new().append(true).create(true).open(path) {
            Ok(mut f) => {
                let mut num = 0;
                if let Ok(datas) = DATAS.scan.history.read() {
                    for data in &*datas {
                        num += 1;
                        let date = chrono::Local
                            .timestamp_millis(data.1)
                            .format("%Y年%m月%d日 %H:%M:%s")
                            .to_string();
                        let result = match &data.0 {
                            ScanResultType::Host(data) => {
                                data.iter().map(|x| format!("{}\n", x)).collect::<String>()
                            }
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
                            ScanResultType::Os(data) => {
                                data.iter().map(|x| format!("{}\n",x.to_string())).collect::<String>()
                            }
                            ScanResultType::Service(data) => {
                                data.iter().map(|x| format!("{}\n",x.to_string())).collect::<String>()
                            }
                            ScanResultType::Dns(data) => {
                                data.iter().map(|x| format!("{}\n",x.to_string())).collect::<String>()
                            }
                            ScanResultType::Tracert(data) => {
                                data.iter().map(|x| format!("{}\n",x.to_string())).collect::<String>()
                            }
                            ScanResultType::None(data) => data.clone().unwrap_or(String::from("")),
                        };
                        f.write_all(format!("[{}]\n[{}] {}", date, num, result).as_bytes())?;
                        f.write(&[0x0A])?;
                    }
                }
                f.sync_data()?;
                Ok(())
            }
            Err(e) => Err(e),
        }
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Scan history of empty, couldn't download",
        ))
    }
}
