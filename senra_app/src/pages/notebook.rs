use crate::widgets::{Cell, CellMessage, CellType};

#[derive(Debug, Clone)]
pub enum Direction {
    Up,
    Down,
}

#[derive(Debug, Clone)]
pub enum Message {
    CreateCell(CellType),
    RemoveCell(u32),
    MoveCell(u32, Direction),

    Cell(u32, CellMessage),
}

pub struct Notebook {
    cells: Vec<Cell>,
    selected: Option<u32>,
}

impl Notebook {
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            selected: None,
        }
    }
}
