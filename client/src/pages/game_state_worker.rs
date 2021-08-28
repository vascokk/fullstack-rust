use std::time::Duration;
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
};

use serde::{Deserialize, Serialize};
use yew::agent::{Agent, AgentLink, HandlerId, Job};
use yew::format::Json;
use yew::services::storage::Area;
use yew::services::{IntervalService, StorageService, Task};

use crate::models::{ClientState, GameState, User, USER_INFO_KEY};
use crate::rest_helper;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerError {
    pub err: String,
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}

impl Error for ServerError {}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientRequest {
    InitializeBoard,
    MakeMoveRequest(u32),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerResponse {
    DataFetched(String),
    MakeMoveResponse(Result<GameState, ServerError>),
    GetGameStateResponse(Result<GameState, ServerError>),
    GameOver(String),
}

pub enum Msg {
    InitializeWorker,
    Updating,
    MakeMoveResponse(HandlerId, Result<GameState, rest_helper::RestError>),
    GetGameStateResponse(HandlerId, Result<GameState, rest_helper::RestError>),
}

pub struct GameWorker {
    this_user_id: Option<String>,
    link: AgentLink<GameWorker>,
    _task: Box<dyn Task>,
    input_handler: Option<HandlerId>,
    client_state: ClientState,
    storage: StorageService,
}

impl GameWorker {
    pub fn set_user_id(&mut self) {
        let Json(user_info): Json<Result<User, anyhow::Error>> =
            self.storage.restore(USER_INFO_KEY);
        if let Ok(user) = user_info {
            self.this_user_id = Some(user.id);
        }
    }

    fn get_game_state(&self) {
        let link = self.link.clone();
        let input_handler = self.input_handler.unwrap();
        let future = async move {
            let rest_response = rest_helper::get_game_state().await;
            link.send_message(Msg::GetGameStateResponse(input_handler, rest_response));
        };
        wasm_bindgen_futures::spawn_local(future);
    }
}

impl Agent for GameWorker {
    type Reach = Job<Self>;
    type Message = Msg;
    type Input = ClientRequest;
    type Output = ServerResponse;

    fn create(link: AgentLink<Self>) -> Self {
        let duration = Duration::from_secs(3);
        let state_update_callback = link.callback(|_| Msg::Updating);
        let state_update_task = IntervalService::spawn(duration, state_update_callback);
        let storage = StorageService::new(Area::Session).expect("storage was disabled by the user");

        link.send_message(Msg::InitializeWorker);

        Self {
            this_user_id: None,
            link,
            _task: Box::new(state_update_task),
            input_handler: None,
            client_state: ClientState::WaitingForThisUserTurn,
            storage,
        }
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::InitializeWorker => {
                self.set_user_id();
                yew::services::ConsoleService::info("Game State Initialized!");
            }
            Msg::Updating => {
                yew::services::ConsoleService::info("Updating Game State...");

                if let Some(_input_handler) = self.input_handler {
                    self.get_game_state();
                }
            }
            Msg::MakeMoveResponse(who, fetched_response) => {
                let msg = match fetched_response {
                    Ok(game_state) => {
                        yew::services::ConsoleService::info(
                            format!("update::Msg::MakeMoveResponse called: {:#?}", game_state)
                                .as_str(),
                        );
                        self.client_state = ClientState::WaitingForOtherUserTurn;
                        ServerResponse::MakeMoveResponse(Ok(game_state))
                    }
                    Err(err) => ServerResponse::MakeMoveResponse(Err(ServerError { err: err.err })),
                };
                self.link.respond(who, msg);
            }
            Msg::GetGameStateResponse(who, fetched_response) => {
                match fetched_response {
                    Ok(game_state) => {
                        yew::services::ConsoleService::info(
                            format!("update::Msg::UpdateBoardResponse called: {:#?}", game_state)
                                .as_str(),
                        );
                        if game_state.winner {
                            let winer = game_state.last_user_id.clone();
                            self.client_state = ClientState::GameOver(winer.unwrap())
                        }

                        match &self.client_state {
                            ClientState::WaitingForThisUserTurn => {
                                if game_state.last_user_id == self.this_user_id {
                                    // start waiting for the other user turn
                                    self.client_state = ClientState::WaitingForOtherUserTurn
                                }
                            }
                            ClientState::WaitingForOtherUserTurn => {
                                if game_state.last_user_id != self.this_user_id {
                                    // start waiting for this user turn
                                    self.client_state = ClientState::WaitingForThisUserTurn
                                }
                            }
                            ClientState::GameOver(winner) => {
                                let msg = ServerResponse::GameOver(winner.to_string());
                                self.link.respond(who, msg);
                            }
                        }
                        let msg = ServerResponse::GetGameStateResponse(Ok(game_state));
                        self.link.respond(who, msg);
                    }
                    Err(err) => {
                        yew::services::DialogService::alert(&err.err);
                    }
                };
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, who: HandlerId) {
        yew::services::ConsoleService::info(&format!("Request: {:?}", msg));
        self.input_handler = Some(who);
        let link = self.link.clone();
        let future = async move {
            match msg {
                ClientRequest::InitializeBoard => {
                    let rest_response = rest_helper::get_game_state().await;
                    link.send_message(Msg::GetGameStateResponse(who, rest_response));
                }
                ClientRequest::MakeMoveRequest(column) => {
                    let rest_response = rest_helper::make_move(column).await;
                    link.send_message(Msg::MakeMoveResponse(who, rest_response));
                }
            };
        };
        wasm_bindgen_futures::spawn_local(future);
    }
}
