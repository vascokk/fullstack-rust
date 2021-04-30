use super::schema::game_state;
use super::schema::user;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Queryable, Insertable, Debug, Clone)]
#[table_name = "user"]
pub struct User {
    pub id: String,
    pub user_name: String,
    pub user_color: String,
}

#[derive(Serialize, Deserialize, Queryable, Debug, Clone)]
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

#[derive(Deserialize, Serialize, Insertable)]
#[table_name = "game_state"]
pub struct NewGameState {
    pub id: String,
    pub board: Option<String>,
    pub user_1: Option<String>,
}
