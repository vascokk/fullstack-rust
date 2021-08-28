#![cfg_attr(test, feature(proc_macro_hygiene))]
#[macro_use]
extern crate diesel;
extern crate dotenv;

use actix_files as fs;
use actix_session::CookieSession;
use actix_web::{web, App, HttpServer};

pub mod api;
pub mod db;
pub mod game;
pub mod models;
pub mod schema;
pub mod utils;

#[actix_web::main]
pub async fn start_server() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(db::create_conn_pool())
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            .service(
                web::scope("/api")
                    .service(
                        web::resource("/register/{user_name}/{user_color}")
                            .route(web::post().to(api::register)),
                    )
                    .service(web::resource("/new").route(web::get().to(api::new_game)))
                    .service(web::resource("/find").route(web::get().to(api::find)))
                    .service(
                        web::resource("/join/{game_session_id}").route(web::post().to(api::join)),
                    )
                    .service(web::resource("/game-state").route(web::get().to(api::game_state)))
                    .service(
                        web::resource("/make-move/{column}").route(web::post().to(api::make_move)),
                    ),
            )
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}
