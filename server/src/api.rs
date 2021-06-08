use actix_session::Session;
use actix_web::{web, Error, HttpRequest, HttpResponse};

use serde_json::json;
use std::result::Result;

pub use crate::db;
pub use crate::game;
pub use crate::models;
pub use crate::schema;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::SqliteConnection;
use std::ops::Deref;
use uuid::Uuid;

const SESSION_ID_KEY: &str = "session_id";
const USER_ID_KEY: &str = "user_id";
const USER_COLOR_KEY: &str = "user_color";

fn get_db_connection(
    req: HttpRequest,
) -> Result<PooledConnection<ConnectionManager<SqliteConnection>>, Error> {
    if let Some(pool) = req.app_data::<Pool<ConnectionManager<SqliteConnection>>>() {
        match pool.get() {
            Ok(conn) => Ok(conn),
            Err(error) => Err(Error::from(
                HttpResponse::BadGateway().body(error.to_string()),
            )),
        }
    } else {
        Err(Error::from(HttpResponse::BadGateway().body(
            "[api][get_db_connection] Can't get db connection".to_string(),
        )))
    }
}

pub async fn register(
    web::Path((user_name, user_color)): web::Path<(String, String)>,
    session: Session,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    println!("REQ: {:?}", req);
    println!("User Name: {:?}", user_name);
    println!("User Color: {:?}", user_color);

    let conn = get_db_connection(req)?;
    let color = user_color;
    let name = user_name;
    match db::create_new_user(&name, &color, conn.deref()) {
        Ok(user_id) => {
            session.set(USER_ID_KEY, user_id.to_string())?;
            session.set(USER_COLOR_KEY, color.clone())?;
            let user = models::User {
                id: user_id.to_string(),
                user_name: name,
                user_color: color,
            };
            Ok(HttpResponse::Ok().body(json!(user)))
        }
        Err(error) => {
            return Err(Error::from(
                HttpResponse::BadGateway().body(format!("Cant register new user: {}", error)),
            ))
        }
    }
}

pub async fn new_game(session: Session, req: HttpRequest) -> Result<HttpResponse, Error> {
    println!("NEW GAME REQ: {:?}", req);

    let conn = get_db_connection(req)?;

    if let Some(user_id) = session.get::<Uuid>(USER_ID_KEY)? {
        match db::create_new_session(&user_id, conn.deref()) {
            Ok(session_id) => {
                session.set(SESSION_ID_KEY, session_id.to_string())?;
                Ok(HttpResponse::Ok().body(json!({ SESSION_ID_KEY: session_id.to_string() })))
            }
            Err(error) => {
                return Err(Error::from(
                    HttpResponse::BadGateway().body(format!("Cant create new session: {}", error)),
                ))
            }
        }
    } else {
        Err(Error::from(
            HttpResponse::BadGateway().body("Can't find the current user ID in session object"),
        ))
    }
}

pub async fn find(session: Session, req: HttpRequest) -> Result<HttpResponse, Error> {
    println!("REQ: {:?}", req);
    let conn = get_db_connection(req)?;
    match db::find_existing_game_session(conn.deref()) {
        Some(game_state) => {
            session.set(SESSION_ID_KEY, game_state.id.to_owned())?;
            Ok(HttpResponse::Ok().body(game_state.id))
        }
        None => Err(Error::from(
            HttpResponse::NotFound().body("No existing session found"),
        )),
    }
}

pub async fn join(
    game_session_id: web::Path<Uuid>,
    session: Session,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    println!("REQ: {:?}", req);
    let conn = get_db_connection(req)?;
    let game_id = game_session_id.into_inner();
    if let Some(user_2_id) = session.get::<Uuid>(USER_ID_KEY)? {
        match db::join_game_session(&game_id, &user_2_id, conn.deref()) {
            Ok(0) => Err(Error::from(
                HttpResponse::NotFound().body(format!("No waiting sessions with id {}", &game_id)),
            )),
            Ok(1) => Ok(HttpResponse::Ok().body("OK")),
            Ok(_) => Err(Error::from(
                HttpResponse::BadGateway().body("Multiple sessions updated"),
            )),
            Err(error) => Err(Error::from(
                HttpResponse::BadGateway().body(format!("Cant join session: {}", error)),
            )),
        }
    } else {
        Err(Error::from(
            HttpResponse::BadGateway().body("Can't find the current user ID in session object"),
        ))
    }
}

#[deprecated]
pub async fn board(session: Session, req: HttpRequest) -> Result<HttpResponse, Error> {
    println!("REQ: {:?}", req);
    let conn = get_db_connection(req)?;
    if let Some(session_id) = session.get::<Uuid>(SESSION_ID_KEY)? {
        println!("API: board, session_id: {:?}", session_id);
        session.set(SESSION_ID_KEY, session_id)?;

        let res = db::get_board(&session_id, conn.deref());
        match res {
            Ok(board_str) => Ok(HttpResponse::Ok().body(board_str)),
            _ => Err(Error::from(
                HttpResponse::InternalServerError()
                    .body(format!("Can't find game with session id {}", session_id)),
            )),
        }
    } else {
        Err(Error::from(
            HttpResponse::InternalServerError().body("[board] Can't find game session!"),
        ))
    }
}

pub async fn game_state(session: Session, req: HttpRequest) -> Result<HttpResponse, Error> {
    println!("REQ: {:?}", req);
    let conn = get_db_connection(req)?;
    if let Some(session_id) = session.get::<Uuid>(SESSION_ID_KEY)? {
        println!("API: board, session_id: {:?}", session_id);
        session.set(SESSION_ID_KEY, session_id)?;

        // let id = session_id.into_inner();
        let res = db::get_game_state(&session_id, conn.deref());
        match res {
            Ok(game_state) => Ok(HttpResponse::Ok().body(json!(game_state))),
            _ => Err(Error::from(
                HttpResponse::InternalServerError()
                    .body(format!("Can't find game with session id {}", session_id)),
            )),
        }
    } else {
        Err(Error::from(
            HttpResponse::InternalServerError().body("[board] Can't find game session!"),
        ))
    }
}

pub async fn make_move(
    web::Path(column): web::Path<u32>,
    session: Session,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    println!("REQ: {:?}", req);
    let conn = get_db_connection(req)?;
    if let (Some(session_id), Some(user_id)) = (
        session.get::<Uuid>(SESSION_ID_KEY)?,
        session.get::<Uuid>(USER_ID_KEY)?,
    ) {
        let res = game::user_move(session_id, user_id, column as usize, conn.deref());
        match res {
            Ok(game_state) => {
                println!("API make_move returns: {:?}", game_state);
                Ok(HttpResponse::Ok().json(game_state))
            }
            Err(msg) => Err(Error::from(HttpResponse::InternalServerError().body(msg))),
        }
    } else {
        Err(Error::from(
            HttpResponse::InternalServerError().body("[user_move] No session info!"),
        ))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use actix_session::UserSession;
    use actix_web::body::Body;
    use actix_web::http;
    use actix_web::http::Method;
    use actix_web::test::TestRequest;
    use mocktopus::mocking::*;
    use std::panic;

    use crate::utils;
    use db::create_conn_pool;
    use uuid::Uuid;

    fn get_body_str(response_body: &Body) -> String {
        match response_body {
            Body::Bytes(ref body_bytes) => {
                let body_str = std::str::from_utf8(body_bytes).unwrap();
                body_str.to_owned()
            }
            _ => panic!("Response body is empty"),
        }
    }

    fn mock_db_create_new_session(test_session_id: Uuid) {
        db::create_new_session
            .mock_safe(move |_user, _conn| MockResult::Return(Result::Ok(test_session_id)));
    }

    fn mock_db_find_existing_game_session(test_session_id: Uuid, user_1_id: Uuid) {
        db::find_existing_game_session.mock_safe(move |_conn| {
            let game_state = models::GameState {
                id: test_session_id.to_string(),
                board: Some("------------------------------------------------------".to_owned()),
                user_1: Some(user_1_id.to_string()),
                user_2: None,
                winner: false,
                last_user_id: Some(user_1_id.to_string()),
                last_user_color: Some("X".to_string()),
                ended: false,
            };
            MockResult::Return(Some(game_state))
        });
    }

    fn mock_db_create_new_user(user_id: Uuid) {
        db::create_new_user
            .mock_safe(move |_name, _color, _conn| MockResult::Return(Result::Ok(user_id)));
    }

    fn mock_db_get_board(board: &'static str) {
        db::get_board
            .mock_safe(move |_sess, _conn| MockResult::Return(Result::Ok(board.to_owned())));
    }

    fn mock_db_get_game_state(
        test_session_id: Uuid,
        user_1_id: Uuid,
        user_2_id: Uuid,
        target_board: &'static str,
    ) {
        db::get_game_state.mock_safe(move |_sess, _conn| {
            let game_state = models::GameState {
                id: test_session_id.to_string(),
                board: Some(target_board.to_owned()),
                user_1: Some(user_1_id.to_string()),
                user_2: Some(user_2_id.to_string()),
                winner: false,
                last_user_id: Some(user_1_id.to_string()),
                last_user_color: Some("X".to_string()),
                ended: false,
            };
            MockResult::Return(Result::Ok(game_state))
        });
    }

    fn mock_db_join_game_session() {
        db::join_game_session
            .mock_safe(move |_sess, _user, _conn| MockResult::Return(Result::Ok(1)));
    }

    fn mock_game_user_move(test_session_id: Uuid, user_1_id: Uuid, board: String) {
        game::user_move.mock_safe(move |_sess, _user_id, _col_num, _conn| {
            let game_state = models::GameState {
                id: test_session_id.to_string(),
                board: Some(board.to_owned()),
                user_1: Some(user_1_id.to_string()),
                user_2: None,
                winner: false,
                last_user_id: Some(user_1_id.to_string()),
                last_user_color: Some("X".to_string()),
                ended: false,
            };
            MockResult::Return(Result::Ok(game_state))
        });
    }

    fn create_user_session(test_session_id: Uuid, test_user_id: Uuid) -> Session {
        let mut srv_req = TestRequest::post().to_srv_request();
        Session::set_session(
            vec![
                (
                    SESSION_ID_KEY.to_string(),
                    serde_json::to_string(&test_session_id).unwrap(),
                ),
                (
                    USER_ID_KEY.to_string(),
                    serde_json::to_string(&test_user_id).unwrap(),
                ),
                (
                    USER_COLOR_KEY.to_string(),
                    serde_json::to_string("X").unwrap(),
                ),
            ]
            .into_iter(),
            &mut srv_req,
        );

        srv_req.get_session()
    }

    #[actix_rt::test]
    async fn test_register_user_post() {
        let pool = create_conn_pool();
        let req = TestRequest::with_header("content-type", "application/json")
            .method(Method::POST)
            .app_data(pool)
            .to_http_request();

        let user_name = "test_user_name".to_string();
        let user_color = "X".to_string();

        let test_session_id = Uuid::new_v4();
        let user_1 = Uuid::new_v4();
        mock_db_create_new_session(test_session_id);
        mock_db_create_new_user(user_1);

        let session = create_user_session(test_session_id, user_1);

        let response = register(
            web::Path::from((user_name.clone(), user_color.clone())),
            session,
            req,
        )
        .await
        .unwrap();

        assert!(response.status().is_success());
        println!("response: {:#?}", response);
        let response_body = &response.body().as_ref().unwrap();
        match response_body {
            Body::Bytes(ref body_bytes) => {
                let user: models::User = serde_json::from_slice(body_bytes).unwrap();
                assert_eq!(user.user_name, user_name);
                assert_eq!(user.user_color, user_color);
                assert_eq!(user.id, user_1.to_string());
            }
            _ => panic!("Response body is empty"),
        }
    }

    #[actix_rt::test]
    async fn test_new_post() {
        let pool = create_conn_pool();
        let req = TestRequest::with_header("content-type", "application/json")
            .method(Method::POST)
            .app_data(pool)
            .to_http_request();

        let test_session_id = Uuid::new_v4();
        mock_db_create_new_session(test_session_id);
        let user_1 = Uuid::new_v4();
        let session = create_user_session(test_session_id, user_1);

        let response = new_game(session, req).await.unwrap();
        assert!(response.status().is_success());

        // read response
        let body = response.body().as_ref().unwrap();
        match body {
            Body::Bytes(ref body_bytes) => {
                let body_json: serde_json::Value =
                    serde_json::from_slice(body_bytes.as_ref()).unwrap();
                println!("response body: {:#?}", body_json);
                assert_eq!(body_json[SESSION_ID_KEY], test_session_id.to_string());
            }
            _ => panic!("Response body is empty"),
        }
    }

    #[actix_rt::test]
    async fn test_find_get() {
        let pool = create_conn_pool();
        let req = TestRequest::with_header("content-type", "application/json")
            .method(Method::GET)
            .app_data(pool.clone())
            .to_http_request();

        let user_1 = Uuid::new_v4();
        let new_session_id = Uuid::new_v4();
        mock_db_create_new_session(new_session_id);
        mock_db_find_existing_game_session(new_session_id, user_1);

        let session = create_user_session(new_session_id, user_1);

        let response = find(session, req).await.unwrap();
        assert!(response.status().is_success());
        println!("response: {:?}", response);

        assert_eq!(
            get_body_str(&response.body().as_ref().unwrap()),
            new_session_id.to_string()
        );
    }

    #[actix_rt::test]
    async fn test_join_post() {
        let pool = create_conn_pool();
        let req = TestRequest::with_header("content-type", "application/json")
            .method(Method::POST)
            .app_data(pool.clone())
            .to_http_request();

        let test_session_id = Uuid::new_v4();
        mock_db_create_new_session(test_session_id);
        mock_db_join_game_session();

        let user_2 = Uuid::new_v4();
        let session = create_user_session(test_session_id, user_2);

        let resp = join(web::Path::from(user_2), session, req).await;
        println!("response: {:?}", resp);
        assert_eq!(resp.unwrap().status(), http::StatusCode::OK);
    }

    #[actix_rt::test]
    async fn test_board_get() {
        let pool = create_conn_pool();
        let req = TestRequest::with_header("content-type", "application/json")
            .method(Method::GET)
            .app_data(pool.clone())
            .to_http_request();

        let user_1 = Uuid::new_v4();
        let new_session_id = Uuid::new_v4();
        let target_board = "------------------------------------------------------";
        mock_db_create_new_session(new_session_id);
        mock_db_get_board(target_board);

        let session = create_user_session(new_session_id, user_1);

        let response = board(session, req).await.unwrap();

        assert!(response.status().is_success());
        println!("response: {:?}", response);
        assert_eq!(
            get_body_str(&response.body().as_ref().unwrap()),
            target_board
        );
    }

    #[actix_rt::test]
    async fn test_game_state() {
        let pool = create_conn_pool();
        let req = TestRequest::with_header("content-type", "application/json")
            .method(Method::GET)
            .app_data(pool.clone())
            .to_http_request();

        let user_1 = Uuid::new_v4();
        let user_2 = Uuid::new_v4();
        let new_session_id = Uuid::new_v4();
        let target_board = "----------------------------------------------------OXX";
        mock_db_create_new_session(new_session_id);
        mock_db_get_game_state(new_session_id, user_1, user_2, target_board);

        let session = create_user_session(new_session_id, user_1);

        let response = game_state(session, req).await.unwrap();

        assert!(response.status().is_success());
        println!("response: {:?}", response);
        let response_body = &response.body().as_ref().unwrap();
        match response_body {
            Body::Bytes(ref body_bytes) => {
                let game_state: models::GameState = serde_json::from_slice(body_bytes).unwrap();
                assert_eq!(game_state.board.unwrap(), target_board);
            }
            _ => panic!("Response body is empty"),
        }
    }

    #[actix_rt::test]
    async fn test_make_move() {
        let pool = create_conn_pool();
        let req = TestRequest::with_header("content-type", "application/json")
            .method(Method::POST)
            .app_data(pool.clone())
            .to_http_request();

        let test_session_id = Uuid::new_v4();
        let user_1 = Uuid::new_v4();
        let column = 5;

        let board = vec![
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', 'X', 'X', 'X', '-', '-', '-'],
        ];

        mock_db_create_new_session(test_session_id);
        mock_db_join_game_session();
        mock_game_user_move(test_session_id, user_1, utils::arr_to_str(&board));

        let session = create_user_session(test_session_id, user_1);

        let response = make_move(web::Path::from(column), session, req)
            .await
            .unwrap();

        assert!(response.status().is_success());
        println!("response: {:?}", response);

        let body = response.body().as_ref().unwrap();
        match body {
            Body::Bytes(ref body_bytes) => {
                let game_state: models::GameState = serde_json::from_slice(body_bytes).unwrap();
                assert_eq!(game_state.id, test_session_id.to_string());
                assert_eq!(game_state.board.unwrap(), utils::arr_to_str(&board));
            }
            _ => panic!("Response body is empty"),
        }
    }
}
