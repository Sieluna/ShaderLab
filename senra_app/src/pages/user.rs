use iced::widget::{
    button, center, column, container, horizontal_space, mouse_area, row, scrollable, text,
};
use iced::{Alignment, Element, Length, Task};
use senra_api::{NotebookListResponse, UserResponse};

#[derive(Debug, Clone)]
pub enum Message {
    ErrorRequest(String),
    GetUserRequest(UserResponse),

    GetUserRespond(u64),
    GetNotebookRespond(u64),

    LoadUser(u64),
    OpenNotebook(u64),
}

#[derive(Debug, Clone)]
struct NotebookCard {
    id: u64,
    title: String,
    likes: i64,
    preview: Option<Vec<u8>>,
}

impl NotebookCard {
    fn view(&self) -> Element<Message> {
        let card = container(
            column![
                // Preview image placeholder
                container(
                    row![]
                        .width(Length::Fixed(200.0))
                        .height(Length::Fixed(120.0))
                ),
                text(&self.title).size(16).width(Length::Fixed(200.0)),
                row![
                    horizontal_space(),
                    text(format!("❤️ {}", self.likes)).size(12),
                ]
                .width(Length::Fixed(200.0))
            ]
            .spacing(8),
        )
        .padding(8);

        mouse_area(card)
            .on_press(Message::OpenNotebook(self.id))
            .into()
    }
}

pub enum UserPage {
    Loading,
    Page {
        user_id: u64,
        username: String,
        avatar: Option<Vec<u8>>,
        created_at: String,
        notebooks: Vec<NotebookCard>,
        error: Option<String>,
    },
}

impl UserPage {
    pub fn new(user_id: u64) -> (Self, Task<Message>) {
        (Self::Loading, Task::done(Message::LoadUser(user_id)))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::GetUserRequest(response) => {
                *self = Self::Page {
                    user_id: response.id as u64,
                    username: response.username,
                    avatar: response.avatar,
                    created_at: response.created_at,
                    notebooks: response
                        .notebooks
                        .notebooks
                        .into_iter()
                        .map(|notebook| NotebookCard {
                            id: notebook.inner.id as u64,
                            title: notebook.inner.title,
                            likes: notebook.stats.like_count,
                            preview: notebook.preview,
                        })
                        .collect(),
                    error: None,
                };
                Task::none()
            }
            Message::LoadUser(id) => Task::done(Message::GetUserRespond(id)),
            Message::OpenNotebook(id) => Task::done(Message::GetNotebookRespond(id)),
            Message::ErrorRequest(error) => {
                if let Self::Page {
                    error: page_error, ..
                } = self
                {
                    *page_error = Some(error);
                }
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        match self {
            Self::Loading => center(text("Loading...").size(24)).into(),
            Self::Page {
                username,
                avatar,
                created_at,
                notebooks,
                error,
                ..
            } => {
                // Header
                let header = container(
                    column![
                        text(username).size(32),
                        text(format!("Joined at {}", created_at)).size(16),
                    ]
                    .spacing(8),
                )
                .padding(20)
                .align_x(Alignment::Center);

                // Content
                let content = if let Some(error) = error {
                    container(
                        text(error)
                            .size(16)
                            .color(iced::Color::from_rgb(1.0, 0.0, 0.0)),
                    )
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .width(Length::Fill)
                    .height(Length::Fill)
                } else {
                    let notebooks_grid = row![].spacing(20).padding(20).width(Length::Fill);
                    let notebooks_grid = notebooks
                        .iter()
                        .fold(notebooks_grid, |row, notebook| row.push(notebook.view()));

                    container(column![header, notebooks_grid].spacing(30).padding(20))
                };

                scrollable(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
    }
}
