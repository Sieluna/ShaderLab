mod flex;
mod menu_bar;
mod menu_bar_overlay;
mod menu_tree;
mod style;

use iced::{Padding, Rectangle, Size};
pub use menu_bar::MenuBar;
pub use menu_tree::{Item, Menu};
pub use style::{Catalog, Style, default};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawPath {
    FakeHovering,
    Backdrop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Positive,
    Negative,
}

impl Direction {
    pub fn flip(self) -> Self {
        match self {
            Self::Positive => Self::Negative,
            Self::Negative => Self::Positive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    pub fn main(&self, size: Size) -> f32 {
        match self {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }

    pub fn cross(&self, size: Size) -> f32 {
        match self {
            Axis::Horizontal => size.height,
            Axis::Vertical => size.width,
        }
    }

    pub fn pack<T>(&self, main: T, cross: T) -> (T, T) {
        match self {
            Axis::Horizontal => (main, cross),
            Axis::Vertical => (cross, main),
        }
    }
}

pub type Index = Option<usize>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecEvent {
    Event,
    Close,
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct ScrollSpeed {
    pub line: f32,
    pub pixel: f32,
}

pub fn pad_rectangle(rect: Rectangle, padding: Padding) -> Rectangle {
    Rectangle {
        x: rect.x - padding.left,
        y: rect.y - padding.top,
        width: rect.width + padding.horizontal(),
        height: rect.height + padding.vertical(),
    }
}
