mod config;
mod global;
mod network;
mod pages;
mod storage;
mod styles;
mod widgets;

use config::Config;
use iced::widget::center;
use iced::{Element, Subscription, Task, Theme};
use senra_api::Response;
use tracing::warn;

pub use global::{Global, Message as GlobalMessage};
pub use network::{Message as NetworkMessage, Network, Protocol};
pub use pages::{Message as PageMessage, Page};
pub use storage::{Message as StorageMessage, Storage};

const TOKEN_KEY: &str = "auth_token";

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
        let config = Config::default();

        let mut storage = Storage::new(&config);
        let network = Network::new(&config);
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
                    .update(StorageMessage::GetRequest(TOKEN_KEY.to_string()))
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
                NetworkMessage::MessageRespond(response) => match response {
                    Response::Auth(auth) => Task::batch([
                        self.page
                            .update(PageMessage::Receive(Response::Auth(auth.clone())))
                            .map(Message::Page),
                        self.network
                            .update(NetworkMessage::ConnectRequest(auth.token.clone()))
                            .map(Message::Network),
                    ]),
                    Response::Token(verify) => {
                        if let Some(token) = &verify.token {
                            self.network
                                .update(NetworkMessage::ConnectRequest(token.clone()))
                                .map(Message::Network)
                        } else {
                            self.page
                                .update(PageMessage::ShowAuthRequest)
                                .map(Message::Page)
                        }
                    }
                    _ => self
                        .page
                        .update(PageMessage::Receive(response))
                        .map(Message::Page),
                },
                NetworkMessage::Error(error) => {
                    warn!("Network connection error: {}", error);
                    Task::none()
                }
                _ => Task::none(),
            },
            Message::Storage(event) => match event {
                StorageMessage::GetRespond(key, value) if key == TOKEN_KEY => {
                    if let Some(token) = value.and_then(|v| v.as_str().map(String::from)) {
                        self.network
                            .update(NetworkMessage::ConnectRequest(token))
                            .map(Message::Network)
                    } else {
                        Task::none()
                    }
                }
                _ => Task::none(),
            },
            Message::Global(message) => self.global.update(message).map(Message::Global),
            Message::Page(message) => match message {
                PageMessage::Send(protocol, request) => self
                    .network
                    .update(NetworkMessage::MessageRequest(protocol, request))
                    .map(Message::Network),
                _ => self.page.update(message).map(Message::Page),
            },
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
    {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env().add_directive(
                    format!("{}=debug", env!("CARGO_CRATE_NAME"))
                        .parse()
                        .unwrap(),
                ),
            )
            .init();
    }

    iced::application(ShaderLab::title, ShaderLab::update, ShaderLab::view)
        .subscription(ShaderLab::subscription)
        .theme(ShaderLab::theme)
        .window_size((1280.0, 720.0))
        .run_with(ShaderLab::new)
}
