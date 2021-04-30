pub use crate::db;
pub use crate::models;
pub use crate::utils;
use diesel::SqliteConnection;
use itertools::Itertools;
use uuid::Uuid;

#[cfg(test)]
use mocktopus::macros::*;

//---------- Gameplay functions---------------------------------------------------------------------
pub fn is_winner(board: &[Vec<char>]) -> bool {
    let winning_seq_len = 5;
    let rows = board;
    let cols: &Vec<Vec<char>> = &get_columns(board);
    let diags_left = &get_diagonals_left(board);
    let diags_right = &get_diagonals_right(board);
    let lines = vec![rows, cols, diags_left, diags_right];
    let lines_flatten = lines.iter().flat_map(|it| it.iter());
    for line in lines_flatten {
        for (color, group) in &line.iter().group_by(|elt| **elt) {
            let gr: Vec<&char> = group.collect();
            // println!("{}, {:?}", color, gr);
            if color != '-' && gr.len() >= winning_seq_len {
                return true;
            }
        }
    }
    false
}

fn get_columns(board: &[Vec<char>]) -> Vec<Vec<char>> {
    let mut cols: Vec<Vec<char>> = vec![vec![' '; board.len()]; board[0].len()];
    for (row_idx, row) in board.iter().enumerate() {
        for (col_idx, x) in row.iter().enumerate() {
            cols[col_idx][row_idx] = *x;
        }
    }
    cols
}

fn get_diagonals_left(board: &[Vec<char>]) -> Vec<Vec<char>> {
    let h = board.len();
    let w = board[0].len();
    let mut diags: Vec<Vec<char>> = Vec::new();
    for p in 0..(h + w - 1) {
        let mut d: Vec<char> = Vec::new();
        let lower_bound = (p as i8 - h as i8 + 1).max(0) as usize;
        let higher_bound = (p as i8 + 1).min(w as i8) as usize;
        for col in lower_bound..higher_bound {
            let row = (h as i8 - p as i8 + col as i8 - 1) as usize;
            d.push(board[row][col])
        }
        diags.push(d);
    }
    diags
}

fn get_diagonals_right(board: &[Vec<char>]) -> Vec<Vec<char>> {
    let h = board.len();
    let w = board[0].len();
    let mut diags: Vec<Vec<char>> = Vec::new();
    for p in 0..(h + w - 1) {
        let mut d: Vec<char> = Vec::new();
        let lower_bound = (p as i8 - h as i8 + 1).max(0) as usize;
        let higher_bound = (p as i8 + 1).min(w as i8) as usize;
        for col in lower_bound..higher_bound {
            let row = (p as i8 - col as i8) as usize;
            d.push(board[row][col])
        }
        diags.push(d);
    }
    diags
}

#[cfg_attr(test, mockable)]
pub fn user_move(
    ses_id: Uuid,
    user_id: Uuid,
    col_num: usize,
    conn: &SqliteConnection,
) -> Result<models::GameState, String> {
    //TODO: needs refactoring. should not do db calls
    db::get_board(&ses_id, conn).and_then(|s| {
        let board = utils::str_to_arr(&s);
        do_move(user_id, col_num, &board, conn).and_then(|new_board| {
            let board_arr = utils::arr_to_str(&new_board);
            let is_winner = is_winner(&new_board);
            let game_over = is_winner;
            db::update_game_state(&ses_id, &user_id, &board_arr, is_winner, game_over, conn)
                .map_err(|err| err.to_string())
                .map(|_| {
                    //return the updated game state
                    db::get_game_state(&ses_id, conn).unwrap()
                })
        })
    })
}

fn do_move(
    user_id: Uuid,
    col_num: usize,
    board: &[Vec<char>],
    conn: &SqliteConnection,
) -> Result<Vec<Vec<char>>, String> {
    if col_num < 1 || col_num > board[0].len() {
        return Err(format!(
            "There is no column with this number. Max column is: {}",
            board[0].len()
        ));
    }

    let y_curr = board
        .iter()
        .enumerate()
        .filter(|(_, row)| row[col_num - 1] == '-')
        .map(|(y, _)| y)
        .max();
    match y_curr {
        Some(row_num) => {
            let color: char = db::get_user_color(&user_id, conn).unwrap();
            let mut new_board = board.to_owned();
            new_board[row_num][col_num - 1] = color;
            Ok(new_board)
        }
        None => Err("This column is full. Please, try another move".to_owned()),
    }
}

#[cfg(test)]
pub mod tests {
    use crate::db;
    use crate::game::{do_move, get_diagonals_left, get_diagonals_right, is_winner, user_move};
    use crate::utils;
    use db::create_conn_pool;
    use itertools::Itertools;
    use mocktopus::mocking::*;
    use std::ops::Deref;
    use uuid::Uuid;

    #[test]
    pub fn test_is_winner() {
        let mut board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
        ];
        assert!(!is_winner(&board));

        board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', 'X', 'X', 'X', '-', '-', '-'],
        ];
        assert!(!is_winner(&board));

        board = vec![
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', 'X', 'X', 'X', '-', '-', '-'],
        ];
        assert!(is_winner(&board));

        board = vec![
            vec!['-', '-', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'O', '-', 'X', '-'],
            vec!['-', '-', '-', '-', '-', 'O', 'X', '-', '-'],
            vec!['-', '-', '-', '-', '-', 'X', '-', '-', '-'],
            vec!['-', '-', '-', '-', 'X', 'O', '-', '-', '-'],
            vec!['-', '-', '-', 'X', 'X', 'X', '-', '-', '-'],
        ];
        assert!(is_winner(&board));

        board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', 'X', '-', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', 'X', '-', 'O', '-', '-', '-'],
            vec!['-', '-', '-', '-', 'X', 'O', '-', '-', '-'],
            vec!['-', '-', '-', 'X', 'X', 'X', '-', '-', '-'],
        ];
        assert!(is_winner(&board));

        board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', 'X', 'X', 'X', 'X', 'X', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
        ];
        assert!(is_winner(&board));

        board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['X', 'X', '-', 'X', 'X', 'X', '-', 'X', 'X'],
        ];
        assert!(!is_winner(&board));

        let target_board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', 'X', '-', '-', '-', '-', '-', '-'],
        ];

        assert!(!is_winner(&target_board));
    }

    #[test]
    pub fn test_get_diagonals_left() {
        let board = vec![
            vec!['1', '2', '3', '4'],
            vec!['A', 'B', 'C', '4'],
            vec!['W', 'X', 'Y', 'Z'],
            vec!['9', '8', '7', '6'],
        ];

        let target = vec![
            vec!['9'],
            vec!['W', '8'],
            vec!['A', 'X', '7'],
            vec!['1', 'B', 'Y', '6'],
            vec!['2', 'C', 'Z'],
            vec!['3', '4'],
            vec!['4'],
        ];

        let res = get_diagonals_left(&board);

        assert_eq!(res, target)
    }

    #[test]
    pub fn test_get_diagonals_right() {
        let board = vec![
            vec!['1', '2', '3', '4'],
            vec!['A', 'B', 'C', '4'],
            vec!['W', 'X', 'Y', 'Z'],
            vec!['9', '8', '7', '6'],
        ];

        let target = vec![
            vec!['1'],
            vec!['A', '2'],
            vec!['W', 'B', '3'],
            vec!['9', 'X', 'C', '4'],
            vec!['8', 'Y', '4'],
            vec!['7', 'Z'],
            vec!['6'],
        ];

        let res = get_diagonals_right(&board);

        assert_eq!(res, target)
    }

    #[test]
    pub fn test_group_by() {
        let data = vec![1, 3, -2, -2, 1, 0, 1, 2];
        for (key, group) in &data.into_iter().group_by(|elt| *elt >= 0) {
            let g = group.collect::<Vec<i32>>();
            println!("{}, {:?}", key, g);
        }
    }

    #[test]
    pub fn test_do_move() {
        let user_id = Uuid::new_v4();
        let conn = create_conn_pool().get().unwrap();
        let board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
        ];

        let target_board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', 'X', '-', '-'],
        ];

        db::get_user_color.mock_safe(move |_x, _conn| MockResult::Return(Result::Ok('X')));
        let new_board = do_move(user_id, 7, &board, conn.deref()).unwrap();
        assert_eq!(new_board, target_board);
    }

    #[test]
    pub fn test_do_move_adjacent() {
        let user_id = Uuid::new_v4();
        let conn = create_conn_pool().get().unwrap();
        // let user_id = db::create_new_user("test-user".to_string(), "X".to_string(), conn.deref()).unwrap();
        let board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', 'X', '-', '-', '-', '-', '-', '-'],
        ];

        let target_board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', 'X', '-', '-', '-', '-', '-', '-'],
        ];

        db::get_user_color.mock_safe(move |_x, _conn| MockResult::Return(Result::Ok('X')));

        let new_board = do_move(user_id, 2, &board, conn.deref()).unwrap();
        assert_eq!(new_board, target_board);
    }

    #[test]
    pub fn test_do_move_full() {
        let user_id = Uuid::new_v4();
        let conn = create_conn_pool().get().unwrap();
        let board = vec![
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
        ];

        db::get_user_color.mock_safe(move |_x, _conn| MockResult::Return(Result::Ok('X')));

        let new_board = do_move(user_id, 2, &board, conn.deref());
        // log:debug!("{:?}", new_board);
        assert_eq!(
            new_board,
            Err("This column is full. Please, try another move".to_string())
        );
    }

    #[test]
    pub fn test_user_move() {
        let conn = create_conn_pool().get().unwrap();
        let user_id = db::create_new_user("test-user", "X", conn.deref()).unwrap();
        let new_session_id = db::create_new_session(&user_id, conn.deref()).unwrap();

        let board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', 'X', '-', '-', '-', '-', '-', '-'],
        ];

        let res = db::update_game_state(
            &new_session_id,
            &user_id,
            &utils::arr_to_str(&board),
            false,
            false,
            conn.deref(),
        );

        assert_eq!(res.unwrap(), 1);

        let tmp_board = db::get_board(&new_session_id, conn.deref()).unwrap();
        assert_eq!(tmp_board, utils::arr_to_str(&board));

        let target_board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', 'X', '-', '-', '-', '-', '-', '-'],
        ];

        let new_state = user_move(new_session_id, user_id, 2, conn.deref()).unwrap();
        assert_eq!(new_state.board.unwrap(), utils::arr_to_str(&target_board));
        assert_eq!(new_state.last_user_id.unwrap(), user_id.to_string());
        assert_eq!(new_state.winner, false);
    }

    #[test]
    pub fn test_user_move_win_column() {
        let conn = create_conn_pool().get().unwrap();
        let user_id = db::create_new_user("test-user", "X", conn.deref()).unwrap();
        let new_session_id = db::create_new_session(&user_id, conn.deref()).unwrap();
        let board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
        ];
        let board_str = utils::arr_to_str(&board);
        let _ = db::update_game_state(
            &new_session_id,
            &user_id,
            &board_str,
            false,
            false,
            conn.deref(),
        );
        let target_board = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', 'X', '-', '-', '-', '-', '-', '-', '-'],
        ];

        let new_state = user_move(new_session_id, user_id, 2, conn.deref()).unwrap();
        assert_eq!(new_state.board.unwrap(), utils::arr_to_str(&target_board));
        assert_eq!(new_state.last_user_id.unwrap(), user_id.to_string());
        assert_eq!(new_state.winner, true);
        assert_eq!(new_state.ended, true);
    }
}
