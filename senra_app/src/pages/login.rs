use iced::widget::{Space, button, checkbox, column, container, row, text, text_input};
use iced::{Element, Length, Task};

use senra_api::{LoginRequest, RegisterRequest, Request};

#[derive(Debug, Clone)]
pub enum Message {
    Submit(Request),

    InputUsername(String),
    InputEmail(String),
    InputPassword(String),
    ToggleShowPassword,
    ClickRegister,
    ClickLogin,
    Clear,
}

#[derive(Debug, Clone)]
pub struct LoginPage {
    username: String,
    email: String,
    password: String,
    show_password: bool,
}

impl LoginPage {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                username: String::new(),
                email: String::new(),
                password: String::new(),
                show_password: false,
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
            Message::ToggleShowPassword => {
                self.show_password = !self.show_password;
                Task::none()
            }
            Message::ClickLogin => Task::done(Message::Submit(Request::Login(LoginRequest {
                username: self.username.clone(),
                password: self.password.clone(),
            }))),
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
        let username_input = text_input("Username", &self.username)
            .on_input(Message::InputUsername)
            .padding(10)
            .width(300);

        let password_input = text_input("Password", &self.password)
            .on_input(Message::InputPassword)
            .padding(10)
            .width(300)
            .secure(!self.show_password);

        let show_password_toggle = row![
            checkbox("Show password", self.show_password)
                .on_toggle(|_| Message::ToggleShowPassword),
        ]
        .spacing(5);

        let action_buttons = row![
            button("Register")
                .on_press(Message::ClickRegister)
                .padding(10)
                .width(100),
            Space::with_width(Length::Fill),
            button("Login")
                .on_press(Message::ClickLogin)
                .padding(10)
                .width(100),
        ]
        .spacing(10);

        let content = column![
            text("ShaderLab").size(40),
            Space::with_height(30),
            username_input,
            Space::with_height(15),
            password_input,
            show_password_toggle,
            //error_message,
            Space::with_height(30),
            action_buttons,
        ];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::Alignment::Center)
            .into()
    }
}
