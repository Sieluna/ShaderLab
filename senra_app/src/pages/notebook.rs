use std::collections::HashMap;

use iced::widget::{button, center, column, container, mouse_area, row, scrollable, text};
use iced::{Element, Length, Task};
use senra_api::{CreateNotebookRequest, EditNotebookRequest, NotebookResponse};
use serde_json::json;

use crate::widgets::{Cell, CellMessage, CellType};

#[derive(Debug, Clone)]
pub enum Message {
    ErrorRequest(String),
    GetNotebookRequest(NotebookResponse),

    GetNotebookRespond(u64),
    SaveNotebookRespond(CreateNotebookRequest),
    EditNotebookRespond(EditNotebookRequest),

    CreateCell(CellType, Option<u32>),
    RemoveCell(u32),
    MoveUp(u32),
    MoveDown(u32),
    Cell(u32, CellMessage),
    ShowButtons(Option<u32>),
    ClickSave,
}

pub enum NotebookPage {
    Loading,
    Page {
        id: Option<u64>,
        title: String,
        description: Option<String>,
        cells: HashMap<u32, Cell>,
        cell_order: Vec<u32>,
        next_id: u32,
        selected: Option<u32>,
        hovered: Option<u32>,
        error: Option<String>,
    },
}

impl NotebookPage {
    pub fn new(id: Option<u64>) -> (Self, Task<Message>) {
        match id {
            Some(id) => (Self::Loading, Task::done(Message::GetNotebookRespond(id))),
            None => (
                Self::Page {
                    id: None,
                    title: String::new(),
                    description: None,
                    cells: HashMap::new(),
                    cell_order: Vec::new(),
                    next_id: 0,
                    selected: None,
                    hovered: None,
                    error: None,
                },
                Task::none(),
            ),
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ErrorRequest(error) => match self {
                Self::Page {
                    error: page_error, ..
                } => {
                    *page_error = Some(error);
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::GetNotebookRequest(response) => {
                *self = Self::Page {
                    id: Some(response.inner.id as u64),
                    title: response.inner.title,
                    description: response.inner.description,
                    cells: HashMap::new(),
                    cell_order: Vec::new(),
                    next_id: 0,
                    selected: None,
                    hovered: None,
                    error: None,
                };
                Task::none()
            }
            Message::CreateCell(cell_type, position) => match self {
                Self::Page {
                    cells,
                    cell_order,
                    next_id,
                    selected,
                    ..
                } => {
                    let id = *next_id;
                    *next_id += 1;
                    let (cell, task) = Cell::new(cell_type, None);
                    cells.insert(id, cell);

                    if let Some(pos) = position {
                        if let Some(index) = cell_order.iter().position(|&x| x == pos) {
                            cell_order.insert(index, id);
                        } else {
                            cell_order.push(id);
                        }
                    } else {
                        cell_order.push(id);
                    }

                    *selected = Some(id);
                    task.map(move |msg| Message::Cell(id, msg))
                }
                _ => Task::none(),
            },
            Message::RemoveCell(id) => match self {
                Self::Page {
                    cells,
                    cell_order,
                    selected,
                    hovered,
                    ..
                } => {
                    cells.remove(&id);
                    if let Some(pos) = cell_order.iter().position(|&x| x == id) {
                        cell_order.remove(pos);
                    }
                    if *selected == Some(id) {
                        *selected = None;
                    }
                    if *hovered == Some(id) {
                        *hovered = None;
                    }
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::MoveUp(id) => match self {
                Self::Page { cell_order, .. } => {
                    if let Some(pos) = cell_order.iter().position(|&x| x == id) {
                        if pos > 0 {
                            cell_order.swap(pos, pos - 1);
                        }
                    }
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::MoveDown(id) => match self {
                Self::Page { cell_order, .. } => {
                    if let Some(pos) = cell_order.iter().position(|&x| x == id) {
                        if pos < cell_order.len() - 1 {
                            cell_order.swap(pos, pos + 1);
                        }
                    }
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::Cell(id, cell_message) => match self {
                Self::Page { cells, .. } => {
                    if let Some(cell) = cells.get_mut(&id) {
                        cell.update(cell_message)
                            .map(move |msg| Message::Cell(id, msg))
                    } else {
                        Task::none()
                    }
                }
                _ => Task::none(),
            },
            Message::ShowButtons(cell_id) => match self {
                Self::Page { hovered, .. } => {
                    *hovered = cell_id;
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::ClickSave => match self {
                Self::Page {
                    id,
                    title,
                    description,
                    ..
                } => {
                    if let Some(id) = id {
                        Task::done(Message::EditNotebookRespond(EditNotebookRequest {
                            title: None,
                            description: None,
                            content: None,
                            resources: None,
                            shaders: None,
                            tags: None,
                            preview: None,
                            visibility: None,
                        }))
                    } else {
                        Task::done(Message::SaveNotebookRespond(CreateNotebookRequest {
                            title: title.clone(),
                            description: description.clone(),
                            content: json!({}),
                            resources: Vec::new(),
                            shaders: Vec::new(),
                            tags: Vec::new(),
                            preview: None,
                            visibility: "public".to_string(),
                        }))
                    }
                }
                _ => Task::none(),
            },
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        match self {
            Self::Loading => center(text("Loading...").size(24)).into(),
            Self::Page {
                title,
                description,
                cells,
                cell_order,
                hovered,
                error,
                ..
            } => {
                let mut content = column![].spacing(20).padding(10);

                // Title and description
                content = content.push(
                    column![text("Title").size(16), text(title).size(24),]
                        .push_maybe(description.as_ref().map(|d| text(d).size(16)))
                        .spacing(8),
                );

                if let Some(error) = error {
                    content = content.push(
                        container(
                            text(error)
                                .size(16)
                                .color(iced::Color::from_rgb(1.0, 0.0, 0.0)),
                        )
                        .padding(10),
                    );
                }

                // Cells
                for &id in cell_order {
                    if let Some(cell) = cells.get(&id) {
                        // Add cell creation buttons between cells (only visible when hovering)
                        let show_buttons = *hovered == Some(id);
                        let buttons = if show_buttons {
                            row![
                                button("+ Markdown")
                                    .on_press(Message::CreateCell(CellType::Markdown, Some(id)))
                                    .padding(5),
                                button("+ Shader")
                                    .on_press(Message::CreateCell(CellType::Shader, Some(id)))
                                    .padding(5),
                            ]
                            .spacing(10)
                            .align_y(iced::Alignment::Center)
                        } else {
                            row![].spacing(10)
                        };

                        content = content.push(
                            mouse_area(
                                container(buttons)
                                    .padding(5)
                                    .align_x(iced::Alignment::Center)
                                    .width(Length::Fill)
                                    .height(Length::Shrink),
                            )
                            .on_enter(Message::ShowButtons(Some(id)))
                            .on_exit(Message::ShowButtons(None)),
                        );

                        // Add cell
                        content = content.push(
                            container(cell.view().map(move |msg| match msg {
                                CellMessage::MoveUp => Message::MoveUp(id),
                                CellMessage::MoveDown => Message::MoveDown(id),
                                CellMessage::Delete => Message::RemoveCell(id),
                                _ => Message::Cell(id, msg),
                            }))
                            .padding(10),
                        );
                    }
                }

                // Bottom buttons
                let last_id = cell_order.last().map(|x| x + 1);
                let bottom_buttons = row![
                    button("+ Markdown")
                        .on_press(Message::CreateCell(CellType::Markdown, last_id))
                        .padding(5),
                    button("+ Shader")
                        .on_press(Message::CreateCell(CellType::Shader, last_id))
                        .padding(5),
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center);

                content = content.push(
                    container(bottom_buttons)
                        .padding(5)
                        .align_x(iced::Alignment::Center)
                        .width(Length::Fill)
                        .height(Length::Shrink),
                );

                scrollable(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
    }
}
