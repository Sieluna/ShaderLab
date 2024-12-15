use iced::widget::{Shader, button, column, container, markdown, pane_grid, row, scrollable};
use iced::{Alignment, Element, Length, Task, Theme};

use super::editor::{Editor, Message as EditorMessage, Syntax};
use super::viewer::Viewer;

#[derive(Debug, Clone)]
pub enum Message {
    SelectType(CellType),

    MoveUp,
    MoveDown,
    Delete,

    CompileShader,
    Editor(EditorMessage),
    Markdown(markdown::Url),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellType {
    Markdown,
    Shader,
}

pub enum CellPreview {
    Markdown(Vec<markdown::Item>),
    Renderer(Viewer),
}

pub enum CellPane {
    Editor,
    Preview,
}

pub struct Cell {
    panes: pane_grid::State<CellPane>,
    editor: Editor,
    preview: CellPreview,
}

impl Cell {
    pub fn new(cell_type: CellType, content: Option<String>) -> (Self, Task<Message>) {
        let (editor, preview, task) = match cell_type {
            CellType::Markdown => {
                let markdown = content.as_ref().map_or(Vec::new(), |content| {
                    markdown::parse(content.as_str()).collect()
                });
                let editor = Editor::new(Syntax::Markdown, content);
                let preview = CellPreview::Markdown(markdown);
                (editor, preview, Task::<Message>::none())
            }
            CellType::Shader => {
                let editor = Editor::new(Syntax::Wgsl, content);
                let preview = CellPreview::Renderer(Viewer::default());
                (editor, preview, Task::<Message>::none())
            }
        };
        let panes = pane_grid::State::with_configuration(pane_grid::Configuration::Split {
            axis: pane_grid::Axis::Vertical,
            ratio: 0.5,
            a: Box::new(pane_grid::Configuration::Pane(CellPane::Editor)),
            b: Box::new(pane_grid::Configuration::Pane(CellPane::Preview)),
        });

        (
            Self {
                panes,
                editor,
                preview,
            },
            task,
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectType(cell_type) => {
                self.preview = match cell_type {
                    CellType::Markdown => {
                        let markdown = markdown::parse(&self.editor.content()).collect();
                        CellPreview::Markdown(markdown)
                    }
                    CellType::Shader => CellPreview::Renderer(Viewer::default()),
                };

                Task::none()
            }
            Message::Editor(message) => {
                if let EditorMessage::ActionPerformed(action) = &message {
                    let is_edit = action.is_edit();

                    if matches!(self.preview, CellPreview::Markdown { .. }) && is_edit {
                        let markdown = markdown::parse(&self.editor.content()).collect();
                        self.preview = CellPreview::Markdown(markdown);
                    }
                }
                self.editor.update(message).map(Message::Editor)
            }
            Message::CompileShader => {
                let viewer = Viewer::new(self.editor.content());
                self.preview = CellPreview::Renderer(viewer);
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let title_bar = row![
            button(" ↑ ").on_press(Message::MoveUp),
            button(" ↓ ").on_press(Message::MoveDown),
            button(" x ").on_press(Message::Delete),
        ]
        .push_maybe(match &self.preview {
            CellPreview::Renderer { .. } => Some(button(" > ").on_press(Message::CompileShader)),
            _ => None,
        })
        .padding(8)
        .align_y(Alignment::Center);

        let pane_grid = pane_grid::PaneGrid::new(&self.panes, |_, pane, _| match pane {
            CellPane::Editor => pane_grid::Content::new(self.editor.view().map(Message::Editor)),
            CellPane::Preview => match &self.preview {
                CellPreview::Markdown(markdown) => pane_grid::Content::new(
                    scrollable(
                        markdown::view(
                            markdown,
                            markdown::Settings::default(),
                            markdown::Style::from_palette(Theme::TokyoNightStorm.palette()),
                        )
                        .map(Message::Markdown),
                    )
                    .width(Length::Fill),
                ),
                CellPreview::Renderer(viewer) => Shader::new(viewer)
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .into(),
            },
        });

        column![
            title_bar,
            container(pane_grid)
                .width(Length::Fill)
                .height(Length::Fixed(300.0))
                .padding(10)
        ]
        .into()
    }
}
