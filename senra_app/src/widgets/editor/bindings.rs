use iced::advanced::text::editor::Motion;
use iced::advanced::{mouse, text};
use iced::keyboard::{self, Key, Modifiers, key};
use iced::{Event, Padding, Point, Rectangle, Vector};
use smol_str::SmolStr;

use super::state::State;
use super::style::Status;

#[derive(Debug, Clone, PartialEq)]
pub enum Binding<Message> {
    Unfocus,
    Copy,
    Cut,
    Paste,
    Move(Motion),
    Select(Motion),
    SelectWord,
    SelectLine,
    SelectAll,
    Insert(char),
    Enter,
    Backspace,
    Delete,
    Sequence(Vec<Self>),
    Custom(Message),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyPress {
    pub key: Key,
    pub modifiers: Modifiers,
    pub text: Option<SmolStr>,
    pub status: Status,
}

impl<Message> Binding<Message> {
    pub fn from_key_press(event: KeyPress) -> Option<Self> {
        let KeyPress {
            key,
            modifiers,
            text,
            status,
        } = event;

        if status != Status::Focused {
            return None;
        }

        match key.as_ref() {
            Key::Named(key::Named::Enter) => Some(Self::Enter),
            Key::Named(key::Named::Backspace) => Some(Self::Backspace),
            Key::Named(key::Named::Delete) if text.is_none() => Some(Self::Delete),
            Key::Named(key::Named::Escape) => Some(Self::Unfocus),
            Key::Character("c") if modifiers.command() => Some(Self::Copy),
            Key::Character("x") if modifiers.command() => Some(Self::Cut),
            Key::Character("v") if modifiers.command() && !modifiers.alt() => Some(Self::Paste),
            Key::Character("a") if modifiers.command() => Some(Self::SelectAll),
            _ => {
                if let Some(text) = text {
                    let c = text.chars().find(|c| !c.is_control())?;
                    Some(Self::Insert(c))
                } else if let Key::Named(named_key) = key.as_ref() {
                    let motion = motion(named_key)?;
                    let motion = if modifiers.macos_command() {
                        match motion {
                            Motion::Left => Motion::Home,
                            Motion::Right => Motion::End,
                            _ => motion,
                        }
                    } else {
                        motion
                    };

                    let motion = if modifiers.jump() {
                        motion.widen()
                    } else {
                        motion
                    };

                    Some(if modifiers.shift() {
                        Self::Select(motion)
                    } else {
                        Self::Move(motion)
                    })
                } else {
                    None
                }
            }
        }
    }
}

pub enum Update<Message> {
    Click(mouse::Click),
    Drag(Point),
    Release,
    Scroll(f32),
    Binding(Binding<Message>),
}

impl<Message> Update<Message> {
    pub fn from_event<H: text::Highlighter>(
        event: Event,
        state: &State<H>,
        bounds: Rectangle,
        padding: Padding,
        cursor: mouse::Cursor,
        key_binding: Option<&dyn Fn(KeyPress) -> Option<Binding<Message>>>,
    ) -> Option<Self> {
        match event {
            Event::Mouse(event) => match event {
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(cursor_position) = cursor.position_in(bounds) {
                        let click = mouse::Click::new(
                            cursor_position - Vector::new(padding.top, padding.left),
                            mouse::Button::Left,
                            state.last_click,
                        );
                        Some(Update::Click(click))
                    } else if state.focus.is_some() {
                        Some(Update::Binding(Binding::Unfocus))
                    } else {
                        None
                    }
                }
                mouse::Event::ButtonReleased(mouse::Button::Left) => Some(Update::Release),
                mouse::Event::CursorMoved { .. } => match state.drag_click {
                    Some(mouse::click::Kind::Single) => Some(Update::Drag(
                        cursor.position_in(bounds)? - Vector::new(padding.top, padding.left),
                    )),
                    _ => None,
                },
                mouse::Event::WheelScrolled { delta } if cursor.is_over(bounds) => {
                    Some(Update::Scroll(match delta {
                        mouse::ScrollDelta::Lines { y, .. } => {
                            if y.abs() > 0.0 {
                                y.signum() * -(y.abs() * 4.0).max(1.0)
                            } else {
                                0.0
                            }
                        }
                        mouse::ScrollDelta::Pixels { y, .. } => -y / 4.0,
                    }))
                }
                _ => None,
            },
            Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modifiers,
                text,
                ..
            }) => {
                let status = if state.focus.is_some() {
                    Status::Focused
                } else {
                    Status::Active
                };

                let key_press = KeyPress {
                    key,
                    modifiers,
                    text,
                    status,
                };

                if let Some(key_binding) = key_binding {
                    key_binding(key_press)
                } else {
                    Binding::from_key_press(key_press)
                }
                .map(Self::Binding)
            }
            _ => None,
        }
    }
}

fn motion(key: key::Named) -> Option<Motion> {
    match key {
        key::Named::ArrowLeft => Some(Motion::Left),
        key::Named::ArrowRight => Some(Motion::Right),
        key::Named::ArrowUp => Some(Motion::Up),
        key::Named::ArrowDown => Some(Motion::Down),
        key::Named::Home => Some(Motion::Home),
        key::Named::End => Some(Motion::End),
        key::Named::PageUp => Some(Motion::PageUp),
        key::Named::PageDown => Some(Motion::PageDown),
        _ => None,
    }
}
