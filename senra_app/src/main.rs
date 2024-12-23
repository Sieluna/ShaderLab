mod global;
mod network;
mod pages;
mod storage;
mod widgets;

use iced::widget::center;
use iced::{Element, Subscription, Task, Theme};
use senra_api::Response;

pub use global::{Global, Message as GlobalMessage};
pub use network::{Message as NetworkMessage, Network, Protocol};
pub use pages::{Message as PageMessage, Page};
pub use storage::{Message as StorageMessage, Storage};

#[derive(Debug, Clone)]
pub enum Message {
    ToggleTheme,
    Global(GlobalMessage),
    Network(NetworkMessage),
    Storage(StorageMessage),
    Page(PageMessage),
}

struct ShaderLab {
    dark_mode: bool,
    global: Global,
    network: Network,
    storage: Storage,
    page: Page,
}

impl ShaderLab {
    fn new() -> (Self, Task<Message>) {
        let mut storage = Storage::new();
        let network = Network::new(String::from("http://localhost:3000"));
        let (page, page_task) = Page::new();

        (
            Self {
                dark_mode: false,
                global: Global::new(),
                network: network.clone(),
                storage: storage.clone(),
                page,
            },
            Task::batch([
                storage
                    .update(StorageMessage::Get("auth_token".to_string()))
                    .map(Message::Storage),
                page_task.map(Message::Page),
            ]),
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
                NetworkMessage::Incoming(response) => match response {
                    Response::Auth(auth) => self
                        .network
                        .update(NetworkMessage::AuthToken(auth.token.clone()))
                        .map(Message::Network),
                    Response::Verify(verify) => {
                        if let Some(token) = verify.token {
                            self.network
                                .update(NetworkMessage::AuthToken(token))
                                .map(Message::Network)
                        } else {
                            self.page.update(PageMessage::ShowAuth).map(Message::Page)
                        }
                    }
                    _ => Task::none(),
                },
                _ => Task::none(),
            },
            Message::Storage(event) => match event {
                StorageMessage::GetSuccess(key, value) => {
                    if key == "auth_token" {
                        if let Some(token) = value.and_then(|v| v.as_str().map(String::from)) {
                            return self
                                .network
                                .update(NetworkMessage::AuthToken(token))
                                .map(Message::Network);
                        }
                    }
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::Global(message) => self.global.update(message).map(Message::Global),
            Message::Page(message) => self.page.update(message).map(Message::Page),
        }
    }

    fn view(&self) -> Element<Message> {
        center(self.page.view().map(Message::Page)).into()
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
