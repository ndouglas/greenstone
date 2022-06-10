use iced::executor;
use iced::{Application, Column, Command, Container, Element, Length, Subscription};

#[derive(Debug, Clone)]
pub enum Message {}

pub struct UI {}

impl Application for UI {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (Self {}, Command::none())
    }

    fn title(&self) -> String {
        String::from("Greenstone")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {}
        // Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn view(&mut self) -> Element<Message> {
        let content = Column::new();
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            //        .style(style::Container)
            .into()
    }
}
