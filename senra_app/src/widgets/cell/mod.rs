use iced::{Element, Task};

#[derive(Debug, Clone)]
pub enum CellType {
    Markdown,
    Shader,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleMode,
    Content(String),
}

pub trait CellInner {
    fn cell_type(&self) -> CellType;

    fn content(&self) -> &str;

    fn set_content(&mut self, content: String);
}

pub struct Cell {
    inner: Box<dyn CellInner>,
}

impl Cell {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }
    pub fn view(&self) -> Element<Message> {
        todo!()
    }
}
