use iced::{Element, Sandbox, Settings};
use iced::widget::{column, button, text};

fn main() -> iced::Result {
    Counter::run(Settings::default())
}

struct Counter {
    value: i32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
}

impl Sandbox for Counter {
    type Message = Message;

    fn new() -> Self {
        Self { value: 0 }
    }

    fn title(&self) -> String {
        String::from("My First Iced App")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Increment => {
                self.value += 1;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            text(self.value),
            button("Click me").on_press(Message::Increment),
        ]
            .into()
    }
}