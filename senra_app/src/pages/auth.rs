use iced::widget::{
    Column, Row, Space, button, checkbox, column, container, row, text, text_input,
};
use iced::{Alignment, Element, Length, Task};
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
            button("Register")
                .on_press(Message::Switch(AuthState::Register))
                .padding(10)
                .width(100),
            Space::with_width(Length::Fill),
            button("Login")
                .on_press(Message::Switch(AuthState::Login))
                .padding(10)
                .width(100)
        ]
        .spacing(10);

        let username_input = text_input("Username", &self.username)
            .on_input(Message::InputUsername)
            .padding(10)
            .width(300);

        let email_input = match &self.state {
            AuthState::Register => Some(
                text_input("Email", &self.email)
                    .on_input(Message::InputEmail)
                    .padding(10)
                    .width(300),
            ),
            AuthState::Login => None,
        };

        let password_input = text_input("Password", &self.password)
            .on_input(Message::InputPassword)
            .padding(10)
            .width(300)
            .secure(!self.show_password);

        let show_password = Row::new()
            .push(
                checkbox("Show password", self.show_password)
                    .on_toggle(|_| Message::ToggleShowPassword),
            )
            .spacing(5);

        let error_message = match &self.error_message {
            Some(error) => Some(text(error).color(iced::Color::from_rgb(1.0, 0.0, 0.0))),
            None => None,
        };

        let submit_button = button(
            text(match &self.state {
                AuthState::Register => "Register",
                AuthState::Login => "Login",
            })
            .align_y(Alignment::Center),
        );

        let content = Column::new()
            .push(text("ShaderLab").size(40))
            .push(Space::with_height(20))
            .push(state_switch)
            .push(Space::with_height(20))
            .push(username_input)
            .push_maybe(email_input.map(|input| column![input, Space::with_height(10)]))
            .push(Space::with_height(10))
            .push(password_input)
            .push(Space::with_height(10))
            .push_maybe(error_message.map(|error| column![error, Space::with_height(10)]))
            .push(show_password)
            .push(Space::with_height(20))
            .push(submit_button);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Alignment::Center)
            .into()
    }
}
