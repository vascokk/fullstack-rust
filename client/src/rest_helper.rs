use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    str,
};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::models;

#[derive(Debug, Clone, PartialEq)]
pub struct FetchError {
    pub err: JsValue,
}

impl Display for FetchError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}

impl Error for FetchError {}

impl From<JsValue> for FetchError {
    fn from(value: JsValue) -> Self {
        Self { err: value }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RestError {
    pub err: String,
}

impl Display for RestError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.err, f)
    }
}

impl Error for RestError {}

pub async fn do_post(url: &str) -> Result<JsValue, FetchError> {
    send_request("POST", url).await
}

pub async fn do_get(url: &str) -> Result<JsValue, FetchError> {
    send_request("GET", url).await
}

//TODO use the new Yew FetchService
pub async fn send_request<'a>(method: &'a str, url: &'a str) -> Result<JsValue, FetchError> {
    let mut opts = RequestInit::new();
    opts.method(method);
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(url, &opts)?;

    let window = yew::utils::window();

    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into().unwrap();

    let resp_text: JsValue = JsFuture::from(resp.text()?).await?;

    Ok(resp_text)
}

fn get_base_url() -> String {
    let origin = yew::utils::origin().expect("Can't get the origin of the current window!");
    let base_url = format!("{}/api", origin);
    base_url
}

fn return_string(resp_text: Result<JsValue, FetchError>) -> Result<String, RestError> {
    Ok(resp_text.unwrap().as_string().unwrap())
}

fn return_game_state(
    resp_text: Result<JsValue, FetchError>,
) -> Result<models::GameState, RestError> {
    let result =
        serde_json::from_str::<models::GameState>(&resp_text.unwrap().as_string().unwrap());
    match result {
        Ok(game_state) => Ok(game_state),
        Err(err) => Err(RestError {
            err: err.to_string(),
        }),
    }
}

fn return_user(resp_text: Result<JsValue, FetchError>) -> Result<models::User, RestError> {
    let result = serde_json::from_str::<models::User>(&resp_text.unwrap().as_string().unwrap());
    match result {
        Ok(user) => Ok(user),
        Err(err) => Err(RestError {
            err: err.to_string(),
        }),
    }
}

pub async fn register_user(user_name: &str, user_color: &str) -> Result<models::User, RestError> {
    let base_url = get_base_url();
    let url = format!("{}/{}/{}/{}", base_url, "register", user_name, user_color);
    let result = do_post(&url).await;
    return_user(result)
}

pub async fn new_game() -> Result<String, RestError> {
    let base_url = get_base_url();
    let url = format!("{}/{}", base_url, "new");
    let result = do_get(&url).await;
    return_string(result)
}

pub async fn find_game() -> Result<String, RestError> {
    let url = format!("{}/{}", get_base_url(), "find");
    let result = do_get(&url).await;
    return_string(result)
}

pub async fn join_game(game_session_id: &str) -> Result<String, RestError> {
    let url = format!("{}/{}/{}", get_base_url(), "join", game_session_id);
    let result = do_post(&url).await;
    return_string(result)
}

pub async fn get_game_state() -> Result<models::GameState, RestError> {
    let url = format!("{}/{}", get_base_url(), "game-state");
    let result = do_get(&url).await;
    return_game_state(result)
}

pub async fn make_move(column: u32) -> Result<models::GameState, RestError> {
    let url = format!("{}/{}/{}", get_base_url(), "make-move", column + 1);
    let result = do_post(&url).await;
    return_game_state(result)
}
