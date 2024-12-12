use iced::widget::{focus_next, text_editor};
use iced::{Task, highlighter};

#[derive(Debug, Clone)]
pub enum Message {
    Action(text_editor::Action),
}

pub struct Editor {
    content: text_editor::Content,
    theme: highlighter::Theme,
    is_loading: bool,
}

impl Editor {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                content: text_editor::Content::new(),
                theme: highlighter::Theme::SolarizedDark,
                is_loading: true,
            },
            focus_next(),
        )
    }
}
