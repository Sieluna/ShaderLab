mod auth;

use iced::widget::text;
use iced::{Element, Task};
use iced::application::Update;
use iced::futures::StreamExt;
use senra_api::{Request, Response};

pub use auth::{AuthPage, Message as AuthMessage};

use crate::Protocol;

#[derive(Debug, Clone)]
pub enum Message {
    ShowAuth,
    Response(Response),

    Request(Protocol, Request),

    Auth(AuthMessage),
}

#[derive(Debug, Clone)]
pub enum PageState {
    Login(AuthPage),
}

pub struct Page {
    state: PageState,
}

impl Page {
    pub fn new() -> (Self, Task<Message>) {
        let (page, task) = AuthPage::new();
        (
            Self {
                state: PageState::Login(page),
            },
            task.map(Message::Auth),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowAuth => {
                let (page, task) = AuthPage::new();
                self.state = PageState::Login(page);
                task.map(Message::Auth)
            }
            Message::Auth(message) => match &mut self.state {
                PageState::Login(page) => page.update(message).map(Message::Auth),
                _ => Task::none(),
            },
            Message::Response(response) => match &mut self.state {
                PageState::Login(page) => {
                    match response {
                        Response::Auth(auth) => {
                            // 登录成功，可以在这里添加跳转到主界面的逻辑
                            Task::none()
                        }
                        Response::Verify(verify) => {
                            if verify.token.is_none() {
                                // Token 无效，显示登录页面
                                let (page, task) = AuthPage::new();
                                self.state = PageState::Login(page);
                                return task.map(Message::Auth);
                            }
                            Task::none()
                        }
                        _ => Task::none(),
                    }
                }
            }
            Message::Request(protocol, request) => {
                Task::done(Message::Request(protocol, request))
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        match &self.state {
            PageState::Login(page) => page.view().map(Message::Auth),
            _ => text("Not implemented").size(20).into(),
        }
    }
}
