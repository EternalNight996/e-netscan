use std::net::{IpAddr, Ipv4Addr};

use crate::datas::DATAS;
use iced::{
    alignment::{self, Horizontal},
    button, scrollable, Alignment, Button, Column, Element, Length, Row, Rule, Scrollable, Text,
};
#[derive(Debug, Clone)]
pub(crate) enum DisplayMessage {
    ScrollToTop,
    ScrollToBottom,
    Scrolled(f32),
}
#[derive(Debug, Clone)]
pub(crate) struct DisplayContent {
    scrollable: scrollable::State,
    scroll_to_top: button::State,
    scroll_to_bottom: button::State,
    latest_offset: f32,
}
impl Default for DisplayContent {
    fn default() -> Self {
        Self {
            scrollable: scrollable::State::new(),
            scroll_to_top: button::State::new(),
            scroll_to_bottom: button::State::new(),
            latest_offset: 0.0,
        }
    }
}
impl DisplayContent {
    pub(crate) fn view(&mut self) -> Element<'_, DisplayMessage> {
        let contxt_count = DATAS.scan.get_result_len();
        let mut num = 0;
        let mut contxt = Column::new();
        match DATAS.scan.result_to_vec() {
            Ok(data) => {
                contxt = data.iter().fold(Column::new(), |column, info| {
                    num += 1;
                    column
                        .push(Text::new(format!("[{}] {}", num, info.to_string())).size(13))
                        .push(Rule::horizontal(15))
                });
            }
            Err(_e) => {}
        }

        // network interface infomation
        let interface_info = Text::new(format!(
            "IP[{}] Gateway[{}]",
            DATAS
                .sysinfo
                .iface
                .local_addr
                .unwrap_or(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            DATAS
                .sysinfo
                .iface
                .gateway
                .as_ref()
                .and_then(|x| Some(format!("{}, {}", x.ip_addr, x.mac_addr)))
                .unwrap_or("".to_owned())
        ))
        .size(14)
        .height(Length::Units(25))
        .width(Length::Fill)
        .horizontal_alignment(alignment::Horizontal::Left)
        .vertical_alignment(alignment::Vertical::Center)
        .color(iced::Color::from_rgb(
            0x10 as f32 / 255.0,
            0x20 as f32 / 255.0,
            0xaf as f32 / 255.0,
        ));

        // 滚动条与输出结果内容
        let scroll = Scrollable::new(&mut self.scrollable)
            .align_items(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .on_scroll(move |offset| DisplayMessage::Scrolled(offset))
            .push(contxt);

        let content = Column::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::End)
            .push(scroll.spacing(5));
        if contxt_count > 40 {
            let go_top = Button::new(
                &mut self.scroll_to_top,
                Text::new("顶")
                    .size(15)
                    .horizontal_alignment(Horizontal::Center),
            )
            .width(Length::Units(35))
            .height(Length::Units(25))
            .on_press(DisplayMessage::ScrollToTop);
            let go_bottom = Button::new(
                &mut self.scroll_to_bottom,
                Text::new("底")
                    .size(15)
                    .horizontal_alignment(Horizontal::Center),
            )
            .width(Length::Units(35))
            .height(Length::Units(25))
            .on_press(DisplayMessage::ScrollToBottom);
            content
                .push(Rule::horizontal(10))
                .push(Row::new().push(interface_info).push(go_bottom).push(go_top))
                .into()
        } else if contxt_count == 0 {
            content
                .push(
                    iced::Text::new("消息结果为空")
                        .horizontal_alignment(alignment::Horizontal::Center)
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .push(Rule::horizontal(10))
                .push(interface_info)
                .into()
        } else {
            content
                .push(Rule::horizontal(10))
                .push(interface_info)
                .into()
        }
    }
    // scroll event occur to update
    pub(crate) fn update(&mut self, msg: DisplayMessage) {
        match msg {
            DisplayMessage::ScrollToTop => {
                self.scrollable.snap_to(0.0);
                self.latest_offset = 0.0;
            }
            DisplayMessage::ScrollToBottom => {
                self.scrollable.snap_to(1.0);
                self.latest_offset = 1.0;
            }
            DisplayMessage::Scrolled(offset) => {
                self.latest_offset = offset;
            }
        }
    }
}
