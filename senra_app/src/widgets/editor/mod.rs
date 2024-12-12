mod highlighter;

use iced::widget::{focus_next, text_editor, TextEditor, column};
use iced::{Element, Task};

use highlighter::{Highlighter, Settings};

#[derive(Debug, Clone)]
pub enum Message {
    ActionPerformed(text_editor::Action),
    WordWrapToggled(bool),
}


pub struct Editor {
    content: text_editor::Content,
    theme: String,
    word_wrap: bool,
    is_dirty: bool,
}

impl Editor {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                content: text_editor::Content::new(),
                theme: String::from("InspiredGitHub"),
                word_wrap: false,
                is_dirty: false,
            },
            focus_next(),
        )
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();

                self.content.perform(action);

                Task::none()
            }
            Message::WordWrapToggled(word_wrap) => {
                self.word_wrap = word_wrap;

                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let text_editor = TextEditor::new(&self.content)
            .padding(10)
            .highlight_with::<Highlighter>(
                Settings {
                    theme: self.theme.clone(),
                    token: String::from("wgsl"),
                    errors: vec![],
                },
                |highlight, _theme| highlight.to_format(),
            )
            .on_action(Message::ActionPerformed);

        column![
            text_editor
        ].into()
    }
}
