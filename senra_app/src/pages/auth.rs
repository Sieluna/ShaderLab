use iced::widget::{button, checkbox, column, container, row, text, text_input};
use iced::{Alignment, Color, Element, Length, Task};
use senra_api::{LoginRequest, RegisterRequest, Request};

#[derive(Debug, Clone)]
pub enum Message {
    Error(String),

    Submit(Request),

    Switch(AuthState),
    InputUsername(String),
    InputEmail(String),
    InputPassword(String),
    ToggleShowPassword,
    ClickRegister,
    ClickLogin,
    Clear,
}

#[derive(Debug, Clone)]
pub enum AuthState {
    Login,
    Register,
}

#[derive(Debug, Clone)]
pub struct AuthPage {
    state: AuthState,
    username: String,
    email: String,
    password: String,
    show_password: bool,
    error_message: Option<String>,
}

impl AuthPage {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: AuthState::Login,
                username: Default::default(),
                email: Default::default(),
                password: Default::default(),
                show_password: false,
                error_message: None,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Error(error) => {
                self.error_message = Some(error);
                Task::none()
            }
            Message::Switch(state) => {
                self.state = state;
                self.error_message = None;
                Task::none()
            }
            Message::InputUsername(username) => {
                self.username = username;
                self.error_message = None;
                Task::none()
            }
            Message::InputEmail(email) => {
                self.email = email;
                self.error_message = None;
                Task::none()
            }
            Message::InputPassword(password) => {
                self.password = password;
                self.error_message = None;
                Task::none()
            }
            Message::ToggleShowPassword => {
                self.show_password = !self.show_password;
                Task::none()
            }
            Message::ClickLogin => {
                if self.username.is_empty() || self.password.is_empty() {
                    self.error_message = Some("Username and password are required".to_string());
                    return Task::none();
                }
                self.error_message = None;
                Task::done(Message::Submit(Request::Login(LoginRequest {
                    username: self.username.clone(),
                    password: self.password.clone(),
                })))
            }
            Message::ClickRegister => {
                if self.username.is_empty() || self.email.is_empty() || self.password.is_empty() {
                    self.error_message = Some("All fields are required".to_string());
                    return Task::none();
                }
                self.error_message = None;
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
                self.error_message = None;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        let state_switch = row![
            button(text("Register").align_x(Alignment::Center))
                .width(Length::FillPortion(1))
                .padding([8, 12])
                .on_press(Message::Switch(AuthState::Register))
                .style(match self.state {
                    AuthState::Register => button::primary,
                    AuthState::Login => button::secondary,
                }),
            button(text("Login").align_x(Alignment::Center))
                .width(Length::FillPortion(1))
                .padding([8, 12])
                .on_press(Message::Switch(AuthState::Login))
                .style(match self.state {
                    AuthState::Register => button::secondary,
                    AuthState::Login => button::primary,
                }),
        ]
        .spacing(6)
        .width(Length::Fill);

        let form = column![]
            .push(
                text_input("Username", &self.username)
                    .on_input(Message::InputUsername)
                    .width(Length::Fill)
                    .padding([8, 12]),
            )
            .push_maybe(match &self.state {
                AuthState::Register => Some(
                    text_input("Email", &self.email)
                        .on_input(Message::InputEmail)
                        .width(Length::Fill)
                        .padding([8, 12]),
                ),
                AuthState::Login => None,
            })
            .push(
                text_input("Password", &self.password)
                    .secure(!self.show_password)
                    .on_input(Message::InputPassword)
                    .width(Length::Fill)
                    .padding([8, 12]),
            )
            .push_maybe(self.error_message.as_ref().map(|error| {
                text(error)
                    .size(14)
                    .color(Color::from_rgb(1.0, 0.0, 0.0))
            }))
            .push(
                checkbox("Show password", self.show_password)
                    .on_toggle(|_| Message::ToggleShowPassword)
                    .width(Length::Fill)
                    .spacing(12)
                    .text_size(14),
            )
            .spacing(12);

        let submit_button = button(
            text(match self.state {
                AuthState::Register => "Register",
                AuthState::Login => "Login",
            })
            .width(Length::Fill)
            .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([8, 12])
        .on_press(match self.state {
            AuthState::Register => Message::ClickRegister,
            AuthState::Login => Message::ClickLogin,
        })
        .style(button::primary);

        let content = column![state_switch, form, submit_button]
            .spacing(24)
            .padding([24, 0])
            .max_width(350);

        container(content)
            .center_x(Length::Fill)
            .align_top(Length::Fill)
            .into()
    }
}
