use yew::format::Json;
use yew::prelude::*;
use yew_router::agent::{RouteAgentDispatcher, RouteRequest};
use yew_router::prelude::*;
use yew::services::storage::Area;
use yew::services::{ConsoleService, DialogService, StorageService};

use crate::models::{User, USER_INFO_KEY};
use crate::rest_helper;
use crate::AppRoute;

pub enum Msg {
    RegisterUser,
    UpdateNameInputText(String),
    UpdateColorInputText(String),
    RegisterUserResponse(Result<User, rest_helper::RestError>),
    FindGameResponse(Result<String, rest_helper::RestError>),
    JoinGameResponse(Result<String, rest_helper::RestError>),
    NewGameResponse(Result<String, rest_helper::RestError>),
}

pub struct Register {
    link: ComponentLink<Self>,
    user_name: Option<String>,
    user_color: Option<String>,
    user: Option<User>,
    router: RouteAgentDispatcher,
    storage: StorageService,
}

impl Component for Register {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            user_name: None,
            user_color: None,
            user: None,
            router: RouteAgentDispatcher::new(),
            storage: StorageService::new(Area::Session).expect("storage was disabled by the user"),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::RegisterUser => {
                ConsoleService::info("===============> ON_SUBMIT: <===================");
                ConsoleService::info(&format!(
                    "===============> DATA: {:?}, {:#?} <===================",
                    self.user_name, self.user_color
                ));
                self.register_user();
                true
            }
            Msg::UpdateNameInputText(val) => {
                println!("Name Input: {}", val);
                self.user_name = Some(val);
                true
            }
            Msg::UpdateColorInputText(val) => {
                println!("Color Input: {}", val);
                self.user_color = Some(val);
                true
            }
            Msg::RegisterUserResponse(fetched_response) => match fetched_response {
                Ok(result) => {
                    ConsoleService::info(
                        format!(
                            "register::update::Msg::RegisterUserResponse called: {:#?}",
                            result
                        )
                        .as_str(),
                    );
                    self.user = Some(result);
                    self.store_user_info();
                    self.find_game();
                    true
                }
                Err(err) => {
                    DialogService::alert(&err.err);
                    true
                }
            },
            Msg::FindGameResponse(fetched_response) => match fetched_response {
                Ok(game_session_id) => {
                    ConsoleService::info(
                        format!(
                            "register::update::Msg::FindGameResponse called: {}",
                            game_session_id
                        )
                        .as_str(),
                    );
                    if game_session_id == "No existing session found" {
                        self.new_game();
                    } else {
                        self.join_game(game_session_id);
                    }
                    true
                }
                Err(err) => {
                    DialogService::alert(&err.err);
                    self.new_game();
                    true
                }
            },
            Msg::JoinGameResponse(fetched_response) => {
                match fetched_response {
                    Ok(_result) => {
                        ConsoleService::info(
                            format!(
                                "register::update::Msg::JoinGameResponse called: {}",
                                _result
                            )
                            .as_str(),
                        );
                        // Navigate to the "Game" page
                        self.goto_game_page();
                        false
                    }
                    Err(err) => {
                        DialogService::alert(&err.err);
                        self.new_game();
                        true
                    }
                }
            }
            Msg::NewGameResponse(fetched_response) => {
                match fetched_response {
                    Ok(game_session_id) => {
                        ConsoleService::info(
                            format!(
                                "register::update::Msg::NewGameResponse called: {}",
                                game_session_id
                            )
                            .as_str(),
                        );
                        // Navigate to the "Game" page
                        self.goto_game_page();
                        false
                    }
                    Err(err) => {
                        DialogService::alert(&err.err);
                        true
                    }
                }
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div class="tile is-ancestor is-vertical">
                <div class="tile is-child hero">
                    <div class="hero-body container pb-0">
                        <h1 class="title is-1">{ "New Game user registration" }</h1>
                            <div>
                                <label for="user_name">{"User Name:"}</label>
                                <input type="text"
                                     oninput=self.link.callback(|e: InputData| Msg::UpdateNameInputText(e.value))
                                />
                                <label for="user_name">{"User Color:"}</label>
                                <input type="text"
                                     oninput=self.link.callback(|e: InputData| Msg::UpdateColorInputText(e.value))
                                />
                                <button onclick=self.link.callback(|_| Msg::RegisterUser)>
                                    { "Submit" }
                                </button>
                            </div>
                    </div>
                </div>
            </div>
        }
    }
}

impl Register {
    fn goto_game_page(&mut self) {
        let route = Route::from(AppRoute::Game);
        self.router.send(RouteRequest::ChangeRoute(route));
    }

    fn register_user(&mut self) {
        let link = self.link.clone();
        let user_name = self.user_name.clone();
        let user_color = self.user_color.clone();
        let future = async move {
            let rest_response =
                rest_helper::register_user(&user_name.unwrap(), &user_color.unwrap()).await;
            link.send_message(Msg::RegisterUserResponse(rest_response));
        };
        wasm_bindgen_futures::spawn_local(future);
    }

    fn find_game(&mut self) {
        let link = self.link.clone();
        let future = async move {
            let rest_response = rest_helper::find_game().await;
            link.send_message(Msg::FindGameResponse(rest_response));
        };
        wasm_bindgen_futures::spawn_local(future);
    }

    fn join_game(&mut self, game_session_id: String) {
        let link = self.link.clone();
        let future = async move {
            let rest_response = rest_helper::join_game(&game_session_id).await;
            link.send_message(Msg::JoinGameResponse(rest_response));
        };
        wasm_bindgen_futures::spawn_local(future);
    }

    fn new_game(&mut self) {
        let link = self.link.clone();
        let future = async move {
            let rest_response = rest_helper::new_game().await;
            link.send_message(Msg::NewGameResponse(rest_response));
        };
        wasm_bindgen_futures::spawn_local(future);
    }

    fn store_user_info(&mut self) {
        self.storage.store(USER_INFO_KEY, Json(&self.user.clone()));
    }
}
