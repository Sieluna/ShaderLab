use std::cell::RefCell;

use iced::advanced::{mouse, text, widget};
use iced::time::Instant;

#[derive(Debug)]
pub struct State<Highlighter: text::Highlighter> {
    pub focus: Option<Focus>,
    pub last_click: Option<mouse::Click>,
    pub drag_click: Option<mouse::click::Kind>,
    pub accumulate_scroll: f32,
    pub partial_scroll: f32,
    pub highlighter: RefCell<Highlighter>,
    pub highlighter_settings: Highlighter::Settings,
    pub highlighter_format_address: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Focus {
    pub updated_at: Instant,
    pub now: Instant,
    pub is_window_focused: bool,
}

impl Focus {
    pub const CURSOR_BLINK_INTERVAL_MILLIS: u128 = 500;

    pub fn now() -> Self {
        let now = Instant::now();
        Self {
            updated_at: now,
            now,
            is_window_focused: true,
        }
    }

    pub fn is_cursor_visible(&self) -> bool {
        self.is_window_focused
            && ((self.now - self.updated_at).as_millis() / Self::CURSOR_BLINK_INTERVAL_MILLIS) % 2
                == 0
    }
}

impl<Highlighter: text::Highlighter> State<Highlighter> {
    pub fn is_focused(&self) -> bool {
        self.focus.is_some()
    }
}

impl<Highlighter: text::Highlighter> widget::operation::Focusable for State<Highlighter> {
    fn is_focused(&self) -> bool {
        self.focus.is_some()
    }

    fn focus(&mut self) {
        self.focus = Some(Focus::now());
    }

    fn unfocus(&mut self) {
        self.focus = None;
    }
}
