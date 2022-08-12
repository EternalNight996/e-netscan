use iced::{
    alignment, executor, window, Alignment, Application, Color, Column, Command, Container,
    Element, Length, Row, Rule, Subscription,
};

mod cmd;
mod display_content;
mod events;
mod left_list;
mod styles;

#[derive(Default, Debug)]
pub(crate) struct App {
    cmd: cmd::Cmd,
    display_contents: display_content::DisplayContent,
    events: events::EventOccur,
}
#[derive(Debug, Clone)]
pub(crate) enum Message {
    Cmd(cmd::CmdMessage),
    Display(display_content::DisplayMessage),
    Events(events::EventMessage),
}

impl Application for App {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();
    fn new(_flags: ()) -> (App, Command<Self::Message>) {
        (App::default(), Command::none())
    }
    /// display model: fullscreen or window or hidden
    fn mode(&self) -> window::Mode {
        window::Mode::Windowed
    }
    /// background color
    fn background_color(&self) -> Color {
        Color::WHITE
    }
    /// content size
    fn scale_factor(&self) -> f64 {
        1.0
    }
    /// titil
    fn title(&self) -> String {
        String::from("A cool application")
    }
    /// subscription some
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            self.cmd.subscription().map(Message::Cmd),
            self.events.subscription().map(Message::Events),
        ])
    }
    /// return exit command
    fn should_exit(&self) -> bool {
        self.events.should_exit
    }
    /// update message
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Cmd(ctx) => {
                self.cmd.update(ctx);
            }
            Message::Display(ctx) => {
                self.display_contents.update(ctx);
            }
            Message::Events(ctx) => {
                self.events.update(ctx);
            }
        }
        Command::none()
    }
    /// window view element
    fn view(&mut self) -> Element<'_, Self::Message> {
        let content = Column::new()
            .align_items(Alignment::Center)
            .spacing(3)
            .padding(5)
            .push(self.cmd.view().map(Message::Cmd))
            .push(Rule::horizontal(3))
            .push(self.events.view().map(Message::Events))
            .push(
                Row::new()
                    .push(Rule::vertical(10))
                    .push(
                        iced::Text::new("Looking Forward")
                            .vertical_alignment(alignment::Vertical::Center)
                            .width(Length::Units(80))
                            .height(Length::Fill),
                    )
                    .push(Rule::vertical(30))
                    .push(self.display_contents.view().map(Message::Display))
                    .push(Rule::vertical(10)),
            );
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Left)
            .align_y(alignment::Vertical::Top)
            .into()
    }
}
