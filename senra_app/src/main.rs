mod state;
mod transport;

use iced::{Application, Task, Element, Length, Settings, Theme, keyboard, window, Size, Subscription, event, mouse, Event};
use iced::widget::{self, Container, button, column, container, row, text, text_editor};

use crate::state::{AppState, Page};
use crate::transport::Transport;

#[derive(Debug, Clone)]
pub enum Message {
    WindowResized(Size),
    ToggleFullscreen(window::Mode),
}

struct ShaderLab {
    state: AppState,
}

impl ShaderLab {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                state: AppState {
                    size: Default::default(),
                    user: None,
                    current: Page::NotebookList,
                    transport: Transport {},
                },
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("ShaderLab")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowResized(size) => {
                self.state.size = size;
                window::get_latest()
                    .and_then(move |window| window::resize(window, size))
            }
            Message::ToggleFullscreen(mode) => {
                window::get_latest()
                    .and_then(move |window| window::change_mode(window, mode))
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            text("Hello, world!"),
        ]
        .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _window| match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                match key {
                    keyboard::Key::Named(key) => {
                        match (key, modifiers) {
                            (keyboard::key::Named::ArrowUp, keyboard::Modifiers::SHIFT) => {
                                Some(Message::ToggleFullscreen(window::Mode::Fullscreen))
                            }
                            (keyboard::key::Named::ArrowDown, keyboard::Modifiers::SHIFT) => {
                                Some(Message::ToggleFullscreen(window::Mode::Windowed))
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            Event::Window(window::Event::Resized(size)) => {
                Some(Message::WindowResized(size))
            }
            _ => None,
        })
    }
}

fn main() -> iced::Result {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    iced::application(ShaderLab::title, ShaderLab::update, ShaderLab::view)
        .subscription(ShaderLab::subscription)
        .theme(ShaderLab::theme)
        .window_size((1280.0, 720.0))
        .run_with(ShaderLab::new)
}
