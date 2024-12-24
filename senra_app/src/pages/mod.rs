mod auth;
mod home;
mod notebook;

use iced::widget::{button, center, column, container, row, text, text_input};
use iced::{Alignment, Element, Length, Renderer, Task, Theme};
use senra_api::{Request, Response};

use auth::{AuthPage, Message as AuthMessage};
use home::{HomePage, Message as HomeMessage};
use notebook::{Message as NotebookMessage, NotebookPage};

use crate::Protocol;
use crate::widgets::menu::{Item, Menu, MenuBar};

#[derive(Debug, Clone)]
pub enum Message {
    ShowAuth,
    ShowHome,
    ShowNotebook,
    Response(Response),

    Request(Protocol, Request),

    Auth(AuthMessage),
    Home(HomeMessage),
    Notebook(NotebookMessage),
}

pub enum PageState {
    Login(AuthPage),
    Home(HomePage),
    Notebook(NotebookPage),
}

pub struct Page {
    state: PageState,
}

impl Page {
    pub fn new() -> (Self, Task<Message>) {
        let (page, task) = HomePage::new();
        (
            Self {
                state: PageState::Home(page),
            },
            task.map(Message::Home),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ShowAuth => {
                let (page, task) = AuthPage::new();
                self.state = PageState::Login(page);
                task.map(Message::Auth)
            }
            Message::ShowHome => {
                let (page, task) = HomePage::new();
                self.state = PageState::Home(page);
                task.map(Message::Home)
            }
            Message::ShowNotebook => {
                let (page, task) = NotebookPage::new();
                self.state = PageState::Notebook(page);
                task.map(Message::Notebook)
            }
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
            Message::Response(response) => match &mut self.state {
                PageState::Login(page) => match response {
                    Response::Auth(auth) => Task::none(),
                    Response::Verify(verify) => {
                        if verify.token.is_none() {
                            let (page, task) = AuthPage::new();
                            self.state = PageState::Login(page);
                            return task.map(Message::Auth);
                        }
                        Task::none()
                    }
                    _ => Task::none(),
                },
                PageState::Home(page) => Task::none(),
                PageState::Notebook(page) => Task::none(),
            },
            Message::Request(protocol, request) => Task::done(Message::Request(protocol, request)),
        }
    }

    pub fn view(&self) -> Element<Message> {
        // Title bar
        let left_bar = MenuBar::<Message, Theme, Renderer>::new(vec![
            Item::new(
                button("Home")
                    .width(Length::Shrink)
                    .padding([6, 12])
                    .on_press(Message::ShowHome)
                    .style(button::primary),
            ),
            Item::with_menu(
                button("File").width(Length::Shrink).style(button::primary),
                Menu::new(vec![
                    Item::new(
                        button("New")
                            .width(Length::Fill)
                            .padding([6, 12])
                            .on_press(Message::ShowAuth)
                            .style(button::primary),
                    ),
                    Item::new(
                        button("Open")
                            .width(Length::Fill)
                            .padding([6, 12])
                            .on_press(Message::ShowAuth)
                            .style(button::primary),
                    ),
                    Item::new(
                        button("Save")
                            .width(Length::Fill)
                            .padding([6, 12])
                            .on_press(Message::ShowAuth)
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
                        .on_press(Message::ShowAuth)
                        .style(button::primary),
                )])
                .max_width(180.0)
                .offset(16.0)
                .spacing(6),
            ),
        ])
        .spacing(6);
        let right_bar = row![
            button("Login")
                .width(Length::Shrink)
                .padding([6, 12])
                .on_press(Message::ShowAuth)
                .style(button::primary),
            button("+ Notebook")
                .width(Length::Shrink)
                .padding([6, 12])
                .on_press(Message::ShowNotebook)
                .style(button::primary),
        ]
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
        };

        column![menu_bar, center(content)].into()
    }
}
