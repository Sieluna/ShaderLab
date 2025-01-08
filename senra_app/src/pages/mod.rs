mod auth;
mod home;
mod notebook;
mod user;

use iced::advanced::image::Handle;
use iced::widget::{button, center, column, container, image, row, text, text_input};
use iced::{Alignment, Element, Length, Renderer, Task, Theme};
use senra_api::{Request, Response, UserInfoResponse};
use tracing::{debug, info};

use auth::{AuthPage, Message as AuthMessage};
use home::{HomePage, Message as HomeMessage};
use notebook::{Message as NotebookMessage, NotebookPage};
use user::{Message as UserMessage, UserPage};

use crate::widgets::menu::{Item, Menu, MenuBar};
use crate::{Protocol, StorageMessage};

#[derive(Debug, Clone)]
pub struct User {
    id: u64,
    username: String,
    avatar: Vec<u8>,
}

impl From<UserInfoResponse> for User {
    fn from(message: UserInfoResponse) -> Self {
        User {
            id: message.id as u64,
            username: message.username,
            avatar: message.avatar,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ShowAuthRequest,
    ShowHomeRequest,
    ShowNotebookRequest(Option<u64>),
    ShowUserRequest(Option<u64>),

    LogoutRespond,
    Noop,

    Send(Protocol, Request),
    Receive(Response),

    SearchInputChanged(String),
    SearchSubmit,

    Auth(AuthMessage),
    Home(HomeMessage),
    Notebook(NotebookMessage),
    User(UserMessage),
}

pub enum PageState {
    Login(AuthPage),
    Home(HomePage),
    Notebook(NotebookPage),
    User(UserPage),
}

pub struct Page {
    state: PageState,
    current_user: Option<User>,
    search_input: String,
}

impl Page {
    pub fn new() -> (Self, Task<Message>) {
        let (page, task) = HomePage::new();
        (
            Self {
                state: PageState::Home(page),
                current_user: None,
                search_input: String::new(),
            },
            task.map(Message::Home),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
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
            Message::ShowUserRequest(id) => {
                if let Some(id) = id.or(self.current_user.as_ref().map(|user| user.id.clone())) {
                    let (page, task) = UserPage::new(id);
                    self.state = PageState::User(page);
                    task.map(Message::User)
                } else {
                    Task::none()
                }
            }
            Message::LogoutRespond => {
                self.current_user = None;
                let (page, task) = HomePage::new();
                self.state = PageState::Home(page);
                task.map(Message::Home)
            }
            Message::Receive(response) => {
                debug!("Received response: {:?}", response);
                match response {
                    Response::Auth(auth) => {
                        self.current_user = Some(auth.user.into());
                        let (page, task) = HomePage::new();
                        self.state = PageState::Home(page);
                        task.map(Message::Home)
                    }
                    _ => Task::none(),
                }
            }
            Message::SearchInputChanged(value) => {
                self.search_input = value;
                Task::none()
            }
            Message::Auth(message) => match &mut self.state {
                PageState::Login(page) => Task::batch([
                    match &message {
                        AuthMessage::LoginRespond(request) => {
                            let request = Request::Login(request.to_owned());
                            Task::done(Message::Send(Protocol::Http, request))
                        }
                        AuthMessage::RegisterRespond(request) => {
                            let request = Request::Register(request.to_owned());
                            Task::done(Message::Send(Protocol::Http, request))
                        }
                        _ => Task::none(),
                    },
                    page.update(message).map(Message::Auth),
                ]),
                _ => Task::none(),
            },
            Message::Home(message) => match &mut self.state {
                PageState::Home(page) => Task::batch([
                    match &message {
                        HomeMessage::OpenNotebook(id) => {
                            Task::done(Message::ShowNotebookRequest(Some(*id)))
                        }
                        _ => Task::none(),
                    },
                    page.update(message).map(Message::Home),
                ]),
                _ => Task::none(),
            },
            Message::Notebook(message) => match &mut self.state {
                PageState::Notebook(page) => Task::batch([
                    match &message {
                        NotebookMessage::GetNotebookRespond(id) => {
                            let request = Request::GetNotebook(*id);
                            Task::done(Message::Send(Protocol::Http, request))
                        }
                        NotebookMessage::SaveNotebookRespond(request) => {
                            let request = Request::CreateNotebook(request.to_owned());
                            Task::done(Message::Send(Protocol::Http, request))
                        }
                        _ => Task::none(),
                    },
                    page.update(message).map(Message::Notebook),
                ]),
                _ => Task::none(),
            },
            Message::User(message) => match &mut self.state {
                PageState::User(page) => Task::batch([
                    match &message {
                        UserMessage::GetUserRespond(id) => {
                            let request = Request::GetUser(*id);
                            Task::done(Message::Send(Protocol::Http, request))
                        }
                        _ => Task::none(),
                    },
                    page.update(message).map(Message::User),
                ]),
                _ => Task::none(),
            },
            _ => Task::none(),
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
                button("File")
                    .width(Length::Shrink)
                    .on_press(Message::Noop)
                    .style(button::primary),
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
                button("Help")
                    .width(Length::Shrink)
                    .on_press(Message::Noop)
                    .style(button::primary),
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
                Some(user) => button(
                    image(Handle::from_bytes(user.avatar.clone()))
                        .width(Length::Fixed(24.0))
                        .height(Length::Fixed(24.0)),
                )
                .width(Length::Shrink)
                .on_press(Message::ShowHomeRequest)
                .style(button::primary),
                None => button("Login")
                    .width(Length::Shrink)
                    .padding([6, 12])
                    .on_press(Message::ShowAuthRequest)
                    .style(button::primary),
            })
            .push(
                button("+ Notebook")
                    .width(Length::Shrink)
                    .padding([6, 12])
                    .on_press(Message::ShowNotebookRequest(None))
                    .style(button::primary),
            )
            .spacing(12);

        let menu_bar = row![
            container(left_bar)
                .align_x(Alignment::Start)
                .width(Length::FillPortion(1)),
            text_input("Search...", &self.search_input)
                .width(Length::FillPortion(1))
                .padding([6, 10])
                .on_input(Message::SearchInputChanged)
                .on_submit(Message::SearchSubmit),
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
            PageState::User(page) => page.view().map(Message::User),
        };

        column![menu_bar, center(content)].into()
    }
}
