use iced::Size;
use serde::{Deserialize, Serialize};

use crate::transport::Transport;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub size: Size,
    pub user: Option<User>,
    pub current: Page,
    pub transport: Transport,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Page {
    NotebookList,
    NotebookDetail { id: String },
    ShaderEditor { id: String },
    ShaderGraph { id: String },
}

#[derive(Debug, Clone)]
pub enum Message {
    // UI Messages
    ShowLogin,
    HideLogin,
    Login { username: String, password: String },
    Logout,
    Navigate(Page),
    
    // WebSocket Messages
    WsConnected,
    WsDisconnected,
    WsMessage(String),
    
    // Notebook Messages
    CreateNotebook { title: String },
    UpdateNotebook { id: String, content: String },
    DeleteNotebook { id: String },
    
    // Shader Messages
    CreateShader { notebook_id: String, name: String, code: String },
    UpdateShader { id: String, code: String },
    DeleteShader { id: String },
    
    // ShaderGraph Messages
    CreateShaderGraph { notebook_id: String, name: String, graph_data: String },
    UpdateShaderGraph { id: String, graph_data: String },
    DeleteShaderGraph { id: String },
}
