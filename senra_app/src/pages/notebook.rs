use std::collections::HashMap;

use iced::widget::{button, column, container, mouse_area, row, scrollable};
use iced::{Element, Length, Task};
use senra_api::{Request, Response};

use crate::widgets::{Cell, CellMessage, CellType};

#[derive(Debug, Clone)]
pub enum Message {
    Send(Request),
    Receive(Response),

    CreateCell(CellType, Option<u32>),
    RemoveCell(u32),
    MoveUp(u32),
    MoveDown(u32),
    Cell(u32, CellMessage),
    ShowButtons(Option<u32>),
}

pub struct NotebookPage {
    cells: HashMap<u32, Cell>,
    cell_order: Vec<u32>,
    next_id: u32,
    selected: Option<u32>,
    hovered: Option<u32>,
}

impl NotebookPage {
    pub fn new(id: Option<u64>) -> (Self, Task<Message>) {
        (
            Self {
                cells: HashMap::new(),
                cell_order: Vec::new(),
                next_id: 0,
                selected: None,
                hovered: None,
            },
            id.map_or(Task::none(), |id| {
                Task::done(Message::Send(Request::GetNotebook(id)))
            }),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Receive(response) => match response {
                _ => Task::none(),
            },
            Message::CreateCell(cell_type, position) => {
                let id = self.next_id;
                self.next_id += 1;
                let (cell, task) = Cell::new(cell_type, None);
                self.cells.insert(id, cell);

                if let Some(pos) = position {
                    if let Some(index) = self.cell_order.iter().position(|&x| x == pos) {
                        self.cell_order.insert(index, id);
                    } else {
                        self.cell_order.push(id);
                    }
                } else {
                    self.cell_order.push(id);
                }

                self.selected = Some(id);
                task.map(move |msg| Message::Cell(id, msg))
            }
            Message::RemoveCell(id) => {
                self.cells.remove(&id);
                if let Some(pos) = self.cell_order.iter().position(|&x| x == id) {
                    self.cell_order.remove(pos);
                }
                if self.selected == Some(id) {
                    self.selected = None;
                }
                if self.hovered == Some(id) {
                    self.hovered = None;
                }
                Task::none()
            }
            Message::MoveUp(id) => {
                if let Some(pos) = self.cell_order.iter().position(|&x| x == id) {
                    if pos > 0 {
                        self.cell_order.swap(pos, pos - 1);
                    }
                }
                Task::none()
            }
            Message::MoveDown(id) => {
                if let Some(pos) = self.cell_order.iter().position(|&x| x == id) {
                    if pos < self.cell_order.len() - 1 {
                        self.cell_order.swap(pos, pos + 1);
                    }
                }
                Task::none()
            }
            Message::Cell(id, cell_message) => {
                if let Some(cell) = self.cells.get_mut(&id) {
                    cell.update(cell_message)
                        .map(move |msg| Message::Cell(id, msg))
                } else {
                    Task::none()
                }
            }
            Message::ShowButtons(cell_id) => {
                self.hovered = cell_id;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let mut content = column![].spacing(20).padding(10);

        for &id in &self.cell_order {
            if let Some(cell) = self.cells.get(&id) {
                // Add cell creation buttons between cells (only visible when hovering)
                let show_buttons = self.hovered == Some(id);
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

        let last_id = self.cell_order.last().map(|x| x + 1);
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
