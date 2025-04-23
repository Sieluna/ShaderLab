mod bindings;
mod content;
mod editor;
mod highlighter;
mod state;
mod style;

use content::Content;
use editor::TextEditor;
use highlighter::{Highlighter, Settings};
use iced::advanced::text::editor::Action;
use iced::widget::column;
use iced::{Element, Task};

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Syntax {
    #[default]
    PlainText,
    Markdown,
    Wgsl,
}

impl Syntax {
    pub fn key(&self) -> &str {
        match self {
            Syntax::PlainText => "plain_text",
            Syntax::Markdown => "markdown",
            Syntax::Wgsl => "wgsl",
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Snapshot,
    SwitchSyntax(Syntax),

    Snapshoted(String),

    ActionPerformed(Action),
    WordWrapToggled(bool),
}

pub struct Editor {
    content: Content,
    theme: String,
    syntax: Syntax,
    word_wrap: bool,
    is_dirty: bool,
}

impl Editor {
    pub fn new(syntax: Syntax, content: Option<String>) -> Self {
        let content = content.unwrap_or_default();

        Self {
            content: Content::with_text(&content),
            theme: String::from("InspiredGitHub"),
            syntax,
            word_wrap: false,
            is_dirty: false,
        }
    }

    pub fn content(&self) -> String {
        self.content.text()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Snapshot => Task::done(Message::Snapshoted(self.content.text())),
            Message::SwitchSyntax(syntax) => {
                self.syntax = syntax;
                Task::none()
            }
            Message::ActionPerformed(action) => {
                self.is_dirty = self.is_dirty || action.is_edit();
                self.content.perform(action);
                Task::none()
            }
            Message::WordWrapToggled(word_wrap) => {
                self.word_wrap = word_wrap;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let text_editor = TextEditor::new(&self.content)
            .placeholder("Type your ideas here...")
            .padding(10)
            .highlight::<Highlighter>(
                Settings {
                    theme: self.theme.clone(),
                    token: self.syntax.clone(),
                    errors: vec![],
                },
                |highlight, _| highlight.to_format(),
            )
            .on_action(Message::ActionPerformed);

        column![text_editor].into()
    }
}
