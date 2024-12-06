use iced::{Event, Size, Subscription, Task, event, keyboard, window};

#[derive(Debug, Clone)]
pub enum Message {
    WindowResized(Size),
    WindowFullscreen(window::Mode),
}

pub struct Global {
    size: Size,
}

impl Global {
    pub fn new() -> Self {
        Self { size: Size::ZERO }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowResized(size) => {
                self.size = size;
                window::get_latest().and_then(move |window| window::resize(window, size))
            }
            Message::WindowFullscreen(mode) => {
                window::get_latest().and_then(move |window| window::change_mode(window, mode))
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _, _| match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => match key {
                keyboard::Key::Named(key) => match (key, modifiers) {
                    (keyboard::key::Named::ArrowUp, keyboard::Modifiers::SHIFT) => {
                        Some(Message::WindowFullscreen(window::Mode::Fullscreen))
                    }
                    (keyboard::key::Named::ArrowDown, keyboard::Modifiers::SHIFT) => {
                        Some(Message::WindowFullscreen(window::Mode::Windowed))
                    }
                    _ => None,
                },
                _ => None,
            },
            Event::Window(window::Event::Resized(size)) => Some(Message::WindowResized(size)),
            _ => None,
        })
    }
}
