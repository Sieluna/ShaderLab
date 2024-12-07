mod login;

use std::sync::Arc;
use iced::widget::text;
use iced::{Element, Task};

pub use login::{LoginPage, Message as LoginMessage};
use senra_api::{Request, Response};
use crate::{Network, Protocol};

#[derive(Debug, Clone)]
pub enum Message {
    ShowLogin,
    Response(Response),

    Request(Protocol, Request),

    Login(LoginMessage),
}

#[derive(Debug, Clone)]
pub enum PageState {
    Login(LoginPage),
}

pub struct Page {
    state: PageState,
}

impl Page {
    pub fn new() -> (Self, Task<Message>) {
        let (page, task) = LoginPage::new();
        (
            Self {
                state: PageState::Login(page),
            },
            task.map(Message::Login),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowLogin => {
                let (page, task) = LoginPage::new();
                self.state = PageState::Login(page);
                task.map(Message::Login)
            },
            Message::Login(message) => match message {
                LoginMessage::Submit(request) => {
                    Task::done(Message::Request(Protocol::Http, request))
                },
                _ => Task::none(),
            },
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        match &self.state {
            PageState::Login(page) => page.view().map(Message::Login),
            _ => text("Not implemented").size(20).into(),
        }
    }
}
