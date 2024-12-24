mod auth;
mod home;
mod notebook;

use iced::widget::{button, center, column, container, row, text, text_input};
use iced::{Alignment, Element, Length, Renderer, Task, Theme};
use senra_api::{Request, Response};

use auth::{AuthPage, Message as AuthMessage};
use home::{HomePage, Message as HomeMessage};
use notebook::{Message as NotebookMessage, NotebookPage};

use crate::widgets::menu::{Item, Menu, MenuBar};
use crate::{NetworkMessage, Protocol, StorageMessage};

#[derive(Debug, Clone)]
pub struct User {
    id: u64,
    username: String,
    // avatar:
}

#[derive(Debug, Clone)]
pub enum Message {
    LoginRequest(User),
    ShowAuthRequest,
    ShowHomeRequest,
    ShowNotebookRequest(Option<u64>),

    LogoutRespond,

    Send(Protocol, Request),
    Receive(Response),

    Auth(AuthMessage),
    Home(HomeMessage),
    Notebook(NotebookMessage),
}

pub enum PageState {
    Login(AuthPage),
    Home(HomePage),
    Notebook(NotebookPage),
    UserProfile(String),
}

pub struct Page {
    state: PageState,
    current_user: Option<User>,
}

impl Page {
    pub fn new() -> (Self, Task<Message>) {
        let (page, task) = HomePage::new();
        (
            Self {
                state: PageState::Home(page),
                current_user: None,
            },
            task.map(Message::Home),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoginRequest(user) => {
                self.current_user = Some(user);
                Task::none()
            },
            Message::ShowAuthRequest => {
                let (page, task) = AuthPage::new();
                self.state = PageState::Login(page);
                task.map(Message::Auth)
            }
            Message::ShowHomeRequest => {
                let (page, task) = HomePage::new();
                self.state = PageState::Home(page);
                task.map(Message::Home)
            }
            Message::ShowNotebookRequest(id) => {
                let (page, task) = NotebookPage::new(id);
                self.state = PageState::Notebook(page);
                task.map(Message::Notebook)
            }
            Message::LogoutRespond => {
                self.current_user = None;
                let (page, task) = HomePage::new();
                self.state = PageState::Home(page);
                task.map(Message::Home)
            }
            Message::Send(protocol, request) => Task::done(Message::Send(protocol, request)),
            Message::Receive(response) => Task::done(Message::Receive(response)),
            Message::Auth(message) => match &mut self.state {
                PageState::Login(page) => page.update(message).map(Message::Auth),
                _ => Task::none(),
            },
            Message::Home(message) => match &mut self.state {
                PageState::Home(page) => page.update(message).map(Message::Home),
                _ => Task::none(),
            },
            Message::Notebook(message) => match &mut self.state {
                PageState::Notebook(page) => page.update(message).map(Message::Notebook),
                _ => Task::none(),
            },
        }
    }

    pub fn view(&self) -> Element<Message> {
        // Title bar
        let left_bar = MenuBar::<Message, Theme, Renderer>::new(vec![
            Item::new(
                button("Home")
                    .width(Length::Shrink)
                    .padding([6, 12])
                    .on_press(Message::ShowHomeRequest)
                    .style(button::primary),
            ),
            Item::with_menu(
                button("File").width(Length::Shrink).style(button::primary),
                Menu::new(vec![
                    Item::new(
                        button("New")
                            .width(Length::Fill)
                            .padding([6, 12])
                            .on_press(Message::ShowNotebookRequest(None))
                            .style(button::primary),
                    ),
                    Item::new(
                        button("Open")
                            .width(Length::Fill)
                            .padding([6, 12])
                            .on_press(Message::ShowAuthRequest)
                            .style(button::primary),
                    ),
                    Item::new(
                        button("Save")
                            .width(Length::Fill)
                            .padding([6, 12])
                            .on_press(Message::ShowAuthRequest)
                            .style(button::primary),
                    ),
                ])
                .max_width(180.0)
                .offset(16.0)
                .spacing(6),
            ),
            Item::with_menu(
                button("Help").width(Length::Shrink).style(button::primary),
                Menu::new(vec![Item::new(
                    button("About")
                        .width(Length::Fill)
                        .padding([6, 12])
                        .on_press(Message::ShowAuthRequest)
                        .style(button::primary),
                )])
                .max_width(180.0)
                .offset(16.0)
                .spacing(6),
            ),
        ])
        .spacing(6);

        let right_bar = row![]
            .push(match &self.current_user {
                Some(user) => {
                    button(user)
                        .width(Length::Shrink)
                        .padding([6, 12])
                        .on_press(Message::ShowHomeRequest)
                        .style(button::primary)
                }
                None => {
                    button("Login")
                        .width(Length::Shrink)
                        .padding([6, 12])
                        .on_press(Message::ShowAuthRequest)
                        .style(button::primary)
                }
            })
            .push(
                button("+ Notebook")
                    .width(Length::Shrink)
                    .padding([6, 12])
                    .on_press(Message::ShowNotebookRequest(None))
                    .style(button::primary)
            )
            .spacing(12);

        let menu_bar = row![
            container(left_bar)
                .align_x(Alignment::Start)
                .width(Length::FillPortion(1)),
            text_input("Search...", "")
                .width(Length::FillPortion(1))
                .padding([6, 10]),
            container(right_bar)
                .align_x(Alignment::End)
                .width(Length::FillPortion(1)),
        ]
        .spacing(12)
        .padding(12)
        .width(Length::Fill)
        .align_y(Alignment::Center);

        // Main content
        let content = match &self.state {
            PageState::Login(page) => page.view().map(Message::Auth),
            PageState::Home(page) => page.view().map(Message::Home),
            PageState::Notebook(page) => page.view().map(Message::Notebook),
            PageState::UserProfile(_) => text("User Profile").into(),
        };

        column![menu_bar, center(content)].into()
    }
}
