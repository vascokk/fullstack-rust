use crate::models::{ClientState, GameState, User, USER_INFO_KEY};
use crate::pages::game_state_worker::{ClientRequest, GameWorker, ServerResponse};
use std::cmp;
use yew::format::Json;
use yew::prelude::*;
use yew::services::storage::Area;
use yew::services::{ConsoleService, DialogService, StorageService};

// const ROWS: u32 = 6; //TODO: make these parameters
const COLUMNS: u32 = 9;

pub struct Game {
    link: ComponentLink<Self>,
    selected_column: Option<u32>,
    hover_column: Option<u32>,
    game_state: Option<GameState>,
    game_state_worker: Box<dyn Bridge<GameWorker>>,
    client_state: ClientState,
    this_user: User,
}

pub enum Msg {
    Initialize,
    SelectColumn(u32),
    MouseOver(u32),
    MouseOut(u32),
    DataReceived(ServerResponse),
    MakeMoveClick,
}

impl Component for Game {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(Msg::DataReceived);
        let storage = StorageService::new(Area::Session).expect("storage was disabled by the user");
        let this_user = get_user_info(&storage).expect("User not registered"); //this must have value, after user registration, panic otherwise
        let game_state_worker = GameWorker::bridge(callback);
        link.send_message(Msg::Initialize);
        Self {
            link,
            selected_column: None,
            hover_column: None,
            game_state: None,
            game_state_worker,
            client_state: ClientState::WaitingForThisUserTurn,
            this_user,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Initialize => {
                self.game_state_worker.send(ClientRequest::InitializeBoard);
                true
            }
            Msg::SelectColumn(column) => {
                self.selected_column = Some(column);
                true
            }
            Msg::MouseOver(column) => {
                self.hover_column = Some(column);
                true
            }
            Msg::MouseOut(_column) => {
                self.hover_column = None;
                true
            }
            Msg::DataReceived(data) => {
                ConsoleService::info("===============> Msg::DataReceived: ");
                self.process_response_data(data);
                true
            }
            Msg::MakeMoveClick => {
                match &self.client_state {
                    ClientState::WaitingForThisUserTurn => {
                        self.game_state_worker.send(ClientRequest::MakeMoveRequest(
                            self.selected_column.unwrap(),
                        ));
                    }
                    ClientState::WaitingForOtherUserTurn => {
                        DialogService::alert("Please, wait for the other user turn to finish!");
                    }
                    ClientState::GameOver(winner) => {
                        DialogService::alert(&format!("GameOver! User {} won.", winner));
                    }
                }
                self.selected_column = None;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <div>
                    <h1 class="title is-4">{["Status: ", self.get_status_msg().as_str()].concat() }</h1>
                </div>
                <div>
                    <table class="table is-bordered">
                    { (0..6).map(|row| self.view_row(row)).collect::<Html>() }
                    </table>
                </div>
                <div>{format!("Selected column:{}", Game::print_selected_column(self.selected_column)) }</div>
                <button onclick={self.link.callback(|_| Msg::MakeMoveClick)}>{ "Make Move" }</button>
            </div>
        }
    }
}

impl Game {
    fn get_status_msg(&self) -> String {
        if let ClientState::GameOver(winner) = &self.client_state {
            return format!("GameOver. User {} won,", winner);
        }
        match &self.game_state {
            Some(game_state) => match &game_state.last_user_id {
                Some(last_user_id) => {
                    if last_user_id == &self.this_user.id {
                        "Other user turn".to_string()
                    } else {
                        "Your turn".to_string()
                    }
                }
                None => "Un-initialized".to_string(),
            },
            None => "Un-initialized".to_string(),
        }
    }

    fn get_square_class(&self, row: u32, column: u32) -> &'static str {
        let mut board = "------------------------------------------------------"; //....I know, needs refactoring.....TODO
        if let Some(game_state) = self.game_state.as_ref() {
            board = game_state.board.as_ref().unwrap();
        }

        match self.selected_column {
            Some(x) if x == column => "square_red",
            _ => match self.hover_column {
                Some(x) if x == column => "col_grey",
                _ => {
                    let idx = (cmp::max(0, row) * COLUMNS + column) as usize;
                    match &board[idx..idx + 1] {
                        "X" => "X",
                        "O" => "O",
                        _ => "square_blue",
                    }
                }
            },
        }
    }

    fn view_square(&self, row: u32, column: u32) -> Html {
        html! {
            <td class={self.get_square_class(row, column)}
                onclick={self.link.callback(move |_| Msg::SelectColumn(column))}
                onmouseover={self.link.callback(move |_| Msg::MouseOver(column))}
                onmouseout={self.link.callback(move |_| Msg::MouseOut(column))}>
            </td>
        }
    }

    fn view_row(&self, row: u32) -> Html {
        html! {
            <tr>
                {for (0..9).map(|column| {
                    self.view_square(row, column)
                })}
            </tr>
        }
    }

    fn print_selected_column(colum: Option<u32>) -> String {
        match colum {
            Some(x) => (x + 1).to_string(),
            None => "".to_owned(),
        }
    }

    pub(crate) fn process_response_data(&mut self, data: ServerResponse) {
        match data {
            ServerResponse::DataFetched(event_data) => {
                ConsoleService::info(&format!(
                    "process_response_data DataFetched:{:#?}",
                    event_data
                ));
            }
            ServerResponse::MakeMoveResponse(event_data) => {
                match event_data {
                    Ok(game_state) => {
                        self.game_state = Some(game_state);
                        self.update_client_state();
                    }
                    Err(err) => DialogService::alert(&err.err),
                };
            }
            ServerResponse::GetGameStateResponse(event_data) => match event_data {
                Ok(game_state) => {
                    self.game_state = Some(game_state);
                    self.update_client_state();
                }
                Err(err) => DialogService::alert(&err.err),
            },
            ServerResponse::GameOver(winner) => {
                self.client_state = ClientState::GameOver(winner);
            }
        }
    }

    fn update_client_state(&mut self) {
        if self.game_state.as_ref().unwrap().winner {
            self.client_state = ClientState::GameOver(
                self.game_state
                    .as_ref()
                    .unwrap()
                    .last_user_id
                    .as_ref()
                    .unwrap()
                    .to_owned(),
            );
        } else {
            match &self.game_state {
                Some(game_state) => match &game_state.last_user_id {
                    Some(last_user_id) => {
                        if last_user_id == &self.this_user.id {
                            self.client_state = ClientState::WaitingForOtherUserTurn;
                        } else {
                            self.client_state = ClientState::WaitingForThisUserTurn;
                        }
                    }
                    None => {
                        self.client_state = ClientState::WaitingForThisUserTurn;
                    }
                },
                None => {
                    self.client_state = ClientState::WaitingForThisUserTurn;
                }
            }
        }
    }
}

fn get_user_info(storage: &StorageService) -> Option<User> {
    let Json(user_info): Json<Result<User, anyhow::Error>> = storage.restore(USER_INFO_KEY);
    match user_info {
        Ok(user) => Some(user),
        _ => None,
    }
}
