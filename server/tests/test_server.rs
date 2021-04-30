#[cfg(test)]
pub mod tests {
    use actix_session::CookieSession;
    use actix_web::{test, web, App, HttpMessage, ResponseError};
    use connect5_rust::models::{GameState, User};
    use connect5_rust::{api, db};
    use serde_json::Value;

    const USER_ID_KEY: &str = "user_id";
    const USER_NAME_KEY: &str = "user_name";
    const SESSION_ID_KEY: &str = "session_id";

    #[actix_rt::test]
    async fn test_register_user_api() {
        let srv = test::start(|| {
            App::new()
                .app_data(db::create_conn_pool())
                .wrap(CookieSession::signed(&[0; 32]).secure(false))
                .service(
                    web::scope("/api").service(
                        web::resource("/register/{user_name}/{user_color}")
                            .route(web::post().to(api::register)),
                    ),
                )
        });

        let mut response = srv
            .post("/api/register/test_user_name/X")
            .send()
            .await
            .unwrap();
        println!("response: {:#?}", response);
        assert!(response.status().is_success());

        // read response
        let body_bytes = response.body().await.unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(body_bytes.as_ref()).unwrap();
        println!("response body: {:#?}", body_json);
        assert_eq!(body_json[USER_NAME_KEY], "test_user_name");
        assert_ne!(body_json[USER_ID_KEY].to_string(), "Null");
    }

    #[actix_rt::test]
    async fn test_game_state_api() {
        let srv = test::start(|| {
            App::new()
                .app_data(db::create_conn_pool())
                .wrap(CookieSession::signed(&[0; 32]).path("/").secure(false))
                .service(
                    web::scope("/api")
                        .service(
                            web::resource("/register/{user_name}/{user_color}")
                                .route(web::post().to(api::register)),
                        )
                        .service(web::resource("/new").route(web::get().to(api::new_game)))
                        .service(
                            web::resource("/game-state").route(web::get().to(api::game_state)),
                        ),
                )
        });

        let mut response = srv
            .post("/api/register/test_user_name/X")
            .send()
            .await
            .unwrap();
        println!("response: {:#?}", response);
        assert!(response.status().is_success());
        // read response
        let body_bytes = response.body().await.unwrap();
        let user_1: User = serde_json::from_slice(body_bytes.as_ref()).unwrap();

        //get the session cookie returned in the response
        let mut cookie = response.cookie("actix-session").unwrap();

        response = srv.get("/api/new").cookie(cookie).send().await.unwrap();

        println!("response: {:#?}", response);
        assert!(response.status().is_success());

        let body_bytes = response.body().await.unwrap();
        let session_id_json: Value = serde_json::from_slice(body_bytes.as_ref()).unwrap();

        cookie = response.cookie("actix-session").unwrap();
        response = srv
            .get("/api/game-state")
            .cookie(cookie)
            .send()
            .await
            .unwrap();
        println!("response: {:#?}", response);
        assert!(response.status().is_success());

        // read response
        let body_bytes = response.body().await.unwrap();
        let game_state: GameState = serde_json::from_slice(body_bytes.as_ref()).unwrap();
        println!("response body: {:#?}", game_state);
        assert_eq!(game_state.user_1.unwrap(), user_1.id);
        assert_eq!(
            game_state.board.unwrap(),
            "------------------------------------------------------".to_string()
        );
        assert_eq!(game_state.id, session_id_json[SESSION_ID_KEY]);
    }

    #[actix_rt::test]
    async fn test_new_api() {
        todo!()
    }

    #[actix_rt::test]
    async fn test_find_api() {
        todo!()
    }

    #[actix_rt::test]
    async fn test_join_api() {
        todo!()
    }

    #[actix_rt::test]
    async fn test_make_move_api() {
        let srv = test::start(|| {
            App::new()
                .app_data(db::create_conn_pool())
                .wrap(CookieSession::signed(&[0; 32]).path("/").secure(false))
                .service(
                    web::scope("/api")
                        .service(
                            web::resource("/register/{user_name}/{user_color}")
                                .route(web::post().to(api::register)),
                        )
                        .service(web::resource("/new").route(web::get().to(api::new_game)))
                        .service(web::resource("/game-state").route(web::get().to(api::game_state)))
                        .service(
                            web::resource("/make-move/{column}")
                                .route(web::post().to(api::make_move)),
                        ),
                )
        });

        let mut response = srv
            .post("/api/register/test_user_name/X")
            .send()
            .await
            .unwrap();
        println!("response: {:#?}", response);
        assert!(response.status().is_success());
        // read response
        let body_bytes = response.body().await.unwrap();
        let user_1: User = serde_json::from_slice(body_bytes.as_ref()).unwrap();

        //get the session cookie returned in the response
        let mut cookie = response.cookie("actix-session").unwrap();

        response = srv.get("/api/new").cookie(cookie).send().await.unwrap();

        println!("response: {:#?}", response);
        assert!(response.status().is_success());

        let body_bytes = response.body().await.unwrap();
        let session_id_json: Value = serde_json::from_slice(body_bytes.as_ref()).unwrap();

        cookie = response.cookie("actix-session").unwrap();
        response = srv
            .get("/api/game-state")
            .cookie(cookie.clone())
            .send()
            .await
            .unwrap();
        println!("response: {:#?}", response);
        assert!(response.status().is_success());

        // read response
        let body_bytes = response.body().await.unwrap();
        let game_state: GameState = serde_json::from_slice(body_bytes.as_ref()).unwrap();
        println!("response body: {:#?}", game_state);
        assert_eq!(game_state.user_1.unwrap(), user_1.id);
        assert_eq!(
            game_state.board.unwrap(),
            "------------------------------------------------------".to_string()
        );
        assert_eq!(game_state.id, session_id_json[SESSION_ID_KEY]);

        response = srv
            .post("/api/make-move/7")
            .cookie(cookie)
            .send()
            .await
            .unwrap();
        println!("response: {:#?}", response);
        let body_bytes = response.body().await;

        match &body_bytes {
            Ok(bytes) => {
                println!("response content: {:#?}", bytes);
            }
            Err(err) => {
                println!("response Error: {:#?}", err.error_response());
            }
        }

        assert!(response.status().is_success());
        let game_state: GameState = serde_json::from_slice(body_bytes.unwrap().as_ref()).unwrap();
        assert_eq!(
            game_state.board.unwrap(),
            "---------------------------------------------------X--".to_string()
        );
    }

    #[actix_rt::test]
    async fn test_board_api() {
        todo!()
    }
}
