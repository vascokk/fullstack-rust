use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub id: String,
    pub board: Option<String>,
    pub user_1: Option<String>,
    pub user_2: Option<String>,
    pub winner: bool,
    pub last_user_id: Option<String>,
    pub last_user_color: Option<String>,
    pub ended: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub user_name: String,
    pub user_color: String,
}

pub enum ClientState {
    WaitingForThisUserTurn,
    WaitingForOtherUserTurn,
    GameOver(String),
}

pub const USER_INFO_KEY: &str = "user_info";
