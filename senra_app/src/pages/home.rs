use iced::widget::{
    button, center, column, container, horizontal_space, mouse_area, row, scrollable, text,
};
use iced::{Alignment, Element, Length, Task};
use senra_api::{NotebookListResponse, NotebookResponse};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub enum Message {
    ErrorRequest(String),

    ListNotebooksRespond(NotebookListResponse),
    NotebookRespond(NotebookResponse),
    OpenNotebookRespond(u64),

    LoadNotebooks,
    SelectCategory(String),
    OpenNotebook(u64),
}

#[derive(Debug, Clone)]
struct NotebookCard {
    id: u64,
    title: String,
    author: String,
    likes: i64,
    preview: Option<Vec<u8>>,
    category: String,
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
                    text(&self.author).size(12),
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

pub enum HomePage {
    Loading,
    Page {
        selected_category: String,
        categories: Vec<String>,
        notebooks: Vec<NotebookCard>,
        error: Option<String>,
    },
}

impl HomePage {
    pub fn new() -> (Self, Task<Message>) {
        (Self::Loading, Task::done(Message::LoadNotebooks))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ListNotebooksRespond(response) => {
                *self = Self::Page {
                    selected_category: "Featured".to_string(),
                    categories: vec![
                        "Featured".to_string(),
                        "Popular".to_string(),
                        "Latest".to_string(),
                        "Shader".to_string(),
                        "Markdown".to_string(),
                    ],
                    notebooks: response
                        .notebooks
                        .into_iter()
                        .map(|notebook| NotebookCard {
                            id: notebook.inner.id as u64,
                            title: notebook.inner.title,
                            author: notebook.author.username,
                            likes: notebook.stats.like_count,
                            preview: notebook.preview,
                            category: "Featured".to_string(),
                        })
                        .collect(),
                    error: None,
                };
                Task::none()
            }
            Message::LoadNotebooks => {
                Task::done(Message::ListNotebooksRespond(NotebookListResponse {
                    notebooks: vec![],
                    total: 0,
                }))
            }
            Message::SelectCategory(category) => {
                if let Self::Page {
                    selected_category, ..
                } = self
                {
                    *selected_category = category;
                }
                Task::none()
            }
            Message::OpenNotebook(id) => Task::done(Message::OpenNotebookRespond(id)),
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
                selected_category,
                categories,
                notebooks,
                error,
            } => {
                // Header
                let header = container(text("ShaderLab").size(32))
                    .padding(20)
                    .align_x(Alignment::Center);

                // Category bar
                let category_bar = row![]
                    .spacing(20)
                    .padding([10, 20])
                    .align_y(Alignment::Center);

                let category_bar = categories.iter().fold(category_bar, |row, category| {
                    row.push(
                        button(text(category))
                            .padding([12, 24])
                            .style(if category == selected_category {
                                button::primary
                            } else {
                                button::secondary
                            })
                            .on_press(Message::SelectCategory(category.clone())),
                    )
                });

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
                        .filter(|n| {
                            n.category == *selected_category || *selected_category == "Featured"
                        })
                        .fold(notebooks_grid, |row, notebook| row.push(notebook.view()));

                    container(
                        column![header, category_bar, notebooks_grid]
                            .spacing(30)
                            .padding(20),
                    )
                };

                scrollable(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
    }
}
