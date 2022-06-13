use iced::executor;
use iced::{Application, Column, Command, Container, Element, Length, Subscription, Text};

pub mod messages;
pub use messages::*;

pub struct UI {}

impl Application for UI {
  type Message = Message;
  type Executor = executor::Default;
  type Flags = ();

  #[named]
  fn new(_flags: ()) -> (Self, Command<Message>) {
    (Self {}, Command::none())
  }

  #[named]
  fn title(&self) -> String {
    String::from("Greenstone")
  }

  #[named]
  fn update(&mut self, message: Message) -> Command<Message> {
    match message {}
    // Command::none()
  }

  #[named]
  fn subscription(&self) -> Subscription<Message> {
    Subscription::none()
  }

  #[named]
  fn view(&mut self) -> Element<Message> {
    Column::new().spacing(20).push(Text::new("This is kinda baking my brain TBH.")).into()
  }
}
