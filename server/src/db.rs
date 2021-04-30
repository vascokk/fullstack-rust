use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::result::QueryResult;
use diesel::SqliteConnection;
use dotenv::dotenv;
use std::env;
use std::result::Result;
use std::time::Duration;
use uuid::Uuid;

pub use crate::models;
use crate::models::{GameState, NewGameState, User};
pub use crate::schema;
pub use crate::utils;

#[cfg(test)]
use mocktopus::macros::*;

#[derive(Debug)]
pub struct ConnectionOptions {
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
    pub busy_timeout: Option<Duration>,
}

impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
    for ConnectionOptions
{
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        (|| {
            if self.enable_wal {
                conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
            }
            if self.enable_foreign_keys {
                conn.batch_execute("PRAGMA foreign_keys = ON;")?;
            }
            if let Some(d) = self.busy_timeout {
                conn.batch_execute(&format!("PRAGMA busy_timeout = {};", d.as_millis()))?;
            }
            Ok(())
        })()
        .map_err(diesel::r2d2::Error::QueryError)
    }
}

pub fn create_conn_pool() -> Pool<ConnectionManager<SqliteConnection>> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Pool::builder()
        .max_size(16)
        .connection_customizer(Box::new(ConnectionOptions {
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout: Some(Duration::from_secs(30)),
        }))
        .build(ConnectionManager::<SqliteConnection>::new(db_url))
        .unwrap()
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[cfg_attr(test, mockable)]
pub fn create_new_user(name: &str, color: &str, conn: &SqliteConnection) -> Result<Uuid, String> {
    use super::schema::user::dsl::*;

    let new_user_id = Uuid::new_v4();

    let new_user = User {
        id: new_user_id.to_string(),
        user_name: name.to_owned(),
        user_color: color.to_owned(),
    };

    let result = diesel::insert_into(user).values(&new_user).execute(conn);

    match result {
        Ok(_) => Ok(new_user_id),
        Err(e) => Err(format!("Can't create a new user: {:?}", e)),
    }
}

#[cfg_attr(test, mockable)]
pub fn create_new_session(user: &Uuid, conn: &SqliteConnection) -> Result<Uuid, String> {
    use super::schema::game_state::dsl::*;

    let new_session_id = Uuid::new_v4();

    let new_game = NewGameState {
        id: new_session_id.to_string(),
        board: Some("------------------------------------------------------".to_string()), //TODO generate the string given board h&w
        user_1: Some(user.to_string()),
    };

    let result = diesel::insert_into(game_state)
        .values(&new_game)
        .execute(conn);

    match result {
        Ok(_) => Ok(new_session_id),
        Err(e) => Err(format!("Can't create a new session: {:?}", e)),
    }
}

#[cfg_attr(test, mockable)]
pub fn find_existing_game_session(conn: &SqliteConnection) -> Option<models::GameState> {
    use super::schema::game_state::dsl::*;

    let results = game_state
        .filter(user_2.is_null())
        .load::<GameState>(conn)
        .expect("Error loading posts");

    if !results.is_empty() {
        let session = results.into_iter().rev().collect::<Vec<_>>()[0].clone();
        Some(session)
    } else {
        None
    }
}

#[cfg_attr(test, mockable)]
pub fn join_game_session(
    session_id: &Uuid,
    user2_id: &Uuid,
    conn: &SqliteConnection,
) -> QueryResult<usize> {
    use super::schema::game_state::dsl::*;
    diesel::update(game_state)
        .set(user_2.eq(user2_id.to_string()))
        .filter(id.eq(session_id.to_string()))
        .execute(conn)
}

#[cfg_attr(test, mockable)]
pub fn get_board(session_id: &Uuid, conn: &SqliteConnection) -> Result<String, String> {
    use super::schema::game_state::dsl::*;

    let results = game_state
        .filter(id.eq(session_id.to_string()))
        .limit(1)
        .load::<GameState>(conn)
        .expect("Error loading game state");

    let b = results[0].board.as_ref().unwrap();
    Result::Ok(b.to_string())
}

#[cfg_attr(test, mockable)]
pub fn update_game_state(
    session_id: &Uuid,
    user_id: &Uuid,
    board_str: &str,
    is_winner: bool,
    game_over: bool,
    conn: &SqliteConnection,
) -> QueryResult<usize> {
    use super::schema::game_state::dsl::*;
    diesel::update(game_state)
        .filter(id.eq(session_id.to_string()))
        .set((
            last_user_id.eq(user_id.to_string()),
            board.eq(board_str),
            winner.eq(is_winner),
            ended.eq(game_over),
        ))
        .execute(conn)
}

#[cfg_attr(test, mockable)]
pub fn get_user_color(user_id: &Uuid, conn: &SqliteConnection) -> QueryResult<char> {
    use super::schema::user::dsl::*;
    let res = user
        .select(user_color)
        .filter(id.eq(user_id.to_string()))
        .limit(1)
        .load::<String>(conn)
        .into_iter()
        .collect::<Vec<_>>()[0]
        .clone();
    Ok(res[0].chars().collect::<Vec<char>>()[0])
}

#[cfg_attr(test, mockable)]
pub fn get_game_state(
    session_id: &Uuid,
    conn: &SqliteConnection,
) -> QueryResult<models::GameState> {
    use super::schema::game_state::dsl::*;

    let res = game_state
        .filter(id.eq(session_id.to_string()))
        .limit(1)
        .load::<GameState>(conn)
        .expect("Error loading game state");
    Ok(res[0].clone())
}

pub fn clean_db(conn: &SqliteConnection) {
    use super::schema::game_state::dsl::*;
    // use super::schema::user::dsl::*;
    diesel::delete(game_state).execute(conn).unwrap();
    // diesel::delete(user)
    //     .execute(&conn)
    //     .unwrap();
}

#[cfg(test)]
pub mod tests {
    use crate::db::{
        create_conn_pool, create_new_session, create_new_user, find_existing_game_session,
        get_board, get_game_state, get_user_color, join_game_session, update_game_state,
    };
    use std::ops::Deref;
    use uuid::Uuid;

    #[test]
    pub fn test_get_board() {
        let target_board = "------------------------------------------------------".to_owned();
        let conn = create_conn_pool().get().unwrap();
        let session_id = create_new_session(&Uuid::new_v4(), conn.deref()).unwrap();
        let board = get_board(&session_id, conn.deref()).unwrap();
        // println!("{:?}", board.unwrap());
        assert_eq!(board, target_board);
    }

    #[test]
    pub fn test_find_existing_game_session() {
        //clean_db();
        let conn = create_conn_pool().get().unwrap();
        let session_id = create_new_session(&Uuid::new_v4(), conn.deref()).unwrap();
        let gs = find_existing_game_session(conn.deref()).unwrap();
        assert_eq!(session_id.to_string(), gs.id)
    }

    #[test]
    pub fn test_create_new_session() {
        let user = Uuid::new_v4();
        let conn = create_conn_pool().get().unwrap();
        let result = create_new_session(&user, conn.deref());
        match result {
            Ok(session_id) => println!("created new session with id: {}", session_id),
            Err(error) => {
                assert!(false, "{:?}", error);
            }
        }
    }

    #[test]
    pub fn test_join_game_session() {
        let user = Uuid::new_v4();
        let conn = create_conn_pool().get().unwrap();
        let session_id = create_new_session(&Uuid::new_v4(), conn.deref()).unwrap();
        let updated_records = join_game_session(&session_id, &user, conn.deref());
        assert_eq!(updated_records.unwrap(), 1);
        // let _board = get_board(&session_id, conn.deref());
    }

    #[test]
    pub fn test_update_game_state() {
        let user_id = Uuid::new_v4();
        let conn = create_conn_pool().get().unwrap();
        let session_id = create_new_session(&user_id, conn.deref()).unwrap();
        let new_board = "-X----------------------------------------------------".to_owned();
        let updated_records = update_game_state(
            &session_id,
            &user_id,
            &new_board,
            false,
            false,
            conn.deref(),
        );
        assert_eq!(updated_records.unwrap(), 1);
        // let board = get_board(&session_id, conn.deref());
        let board = get_game_state(&session_id, conn.deref()).unwrap().board;
        assert_eq!(board.unwrap(), new_board);

        let new_board = "-X--X-------------------------------------------------".to_owned();
        let updated_records = update_game_state(
            &session_id,
            &user_id,
            &new_board,
            false,
            false,
            conn.deref(),
        );
        assert_eq!(updated_records.unwrap(), 1);
        let board = get_game_state(&session_id, conn.deref()).unwrap().board;
        assert_eq!(board.unwrap(), new_board);
    }

    #[test]
    pub fn test_get_user_color() {
        let conn = create_conn_pool().get().unwrap();

        let new_user_id = create_new_user("test-user", "X", conn.deref()).unwrap();
        let color = get_user_color(&new_user_id, conn.deref()).unwrap();
        assert_eq!(color, 'X');
    }

    #[test]
    pub fn test_get_game_state() {
        let user = Uuid::new_v4();
        let conn = create_conn_pool().get().unwrap();
        let result = create_new_session(&user, conn.deref());
        match result {
            Ok(session_id) => {
                let gs = get_game_state(&session_id, conn.deref()).unwrap();
                assert_eq!(gs.id, session_id.to_string());
                assert_eq!(gs.user_1.unwrap(), user.to_string());
                assert_eq!(gs.user_2, None);
            }
            Err(error) => panic!("{}", error.as_str()),
        }
    }
}
