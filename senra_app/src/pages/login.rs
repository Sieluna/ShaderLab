use iced::widget::{Column, button, container, text, text_input};
use iced::{Element, Length, Task};

use senra_api::{Request, LoginRequest, RegisterRequest};

#[derive(Debug, Clone)]
pub enum Message {
    Submit(Request),

    InputUsername(String),
    InputEmail(String),
    InputPassword(String),
    ClickRegister,
    ClickLogin,
    Clear,
}

#[derive(Debug, Clone)]
pub struct LoginPage {
    username: String,
    email: String,
    password: String,
}

impl LoginPage {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                username: String::new(),
                email: String::new(),
                password: String::new(),
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::InputUsername(username) => {
                self.username = username;
                Task::none()
            }
            Message::InputEmail(email) => {
                self.email = email;
                Task::none()
            }
            Message::InputPassword(password) => {
                self.password = password;
                Task::none()
            }
            Message::ClickLogin => {
                Task::done(Message::Submit(Request::Login(LoginRequest {
                    username: self.username.clone(),
                    password: self.password.clone(),
                })))
            }
            Message::ClickRegister => {
                Task::done(Message::Submit(Request::Register(RegisterRequest {
                    username: self.username.clone(),
                    email: self.email.clone(),
                    password: self.password.clone(),
                })))
            }
            _ => {
                self.username.clear();
                self.email.clear();
                self.password.clear();
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut content = Column::new()
            .spacing(10)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::Alignment::Center);

        content = content
            .push(text("ShaderLab").size(40))
            .push(
                text_input("Username", &self.username)
                    .on_input(Message::InputUsername)
                    .padding(10)
                    .width(300),
            )
            .push(
                text_input("Password", &self.password)
                    .on_input(Message::InputPassword)
                    .padding(10)
                    .width(300),
            )
            .push(button("Login").on_press(Message::ClickLogin).padding(10));

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(0)
            .center_y(0)
            .into()
    }
}
