mod global;
mod network;
mod pages;
mod storage;

use iced::widget::text;
use iced::{Element, Event, Size, Subscription, Task, Theme, event, keyboard};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub use global::{Global, Message as GlobalMessage};
pub use network::{Network, NetworkMessage};
pub use storage::{Storage, StorageMessage};

#[derive(Debug, Clone)]
pub enum Page {
    Login,
    NotebookList,
    NotebookDetail { id: String },
    ShaderEditor { id: String },
    ShaderGraph { id: String },
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleTheme,
    Global(GlobalMessage),
    Network(NetworkMessage),
    Storage(StorageMessage),
}

struct ShaderLab {
    current_page: Page,
    dark_mode: bool,
    global: Global,
    network: Network,
    storage: Storage,
}

impl ShaderLab {
    fn new() -> (Self, Task<Message>) {
        let storage = Storage::new();
        let transport = Network::new(Arc::from("ws://localhost:3000"));

        (
            Self {
                current_page: Page::NotebookList,
                dark_mode: false,
                global: Global::new(),
                network: transport,
                storage,
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("ShaderLab")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ToggleTheme => {
                self.dark_mode = !self.dark_mode;
                Task::none()
            }
            Message::Network(event) => match event {
                NetworkMessage::Connected(_) => Task::none(),
                NetworkMessage::Disconnected => Task::none(),
                NetworkMessage::Incoming(response) => match response {
                    _ => Task::none(),
                },
                NetworkMessage::Outgoing(_, _) => Task::none(),
                NetworkMessage::Error(e) => {
                    tracing::error!("Transport error: {}", e);
                    Task::none()
                }
            },
            Message::Storage(event) => match event {
                StorageMessage::Error(error) => {
                    tracing::error!("Storage error: {}", error);
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::Global(message) => self.global.update(message).map(Message::Global),
        }
    }

    fn view(&self) -> Element<Message> {
        text("Hello, this is iced!").size(20).into()
    }

    fn theme(&self) -> Theme {
        if self.dark_mode {
            Theme::Dark
        } else {
            Theme::Light
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.global.subscription().map(Message::Global),
            self.network.subscribe().map(Message::Network),
        ])
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
