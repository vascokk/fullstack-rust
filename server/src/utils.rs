pub fn str_to_arr(board_str: &str) -> Vec<Vec<char>> {
    const ROWS: usize = 6; //TODO: make them parameters
    const COLUMNS: usize = 9;

    let mut board_arr: Vec<Vec<char>> = Vec::new();

    for y in 0..ROWS {
        let first_idx = y * COLUMNS;
        let row = board_str[first_idx..(first_idx + COLUMNS)]
            .chars()
            .collect::<Vec<char>>();
        board_arr.push(row);
    }
    board_arr
}

pub fn arr_to_str(board: &[Vec<char>]) -> String {
    board.iter().flatten().collect::<String>()
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    pub fn test_str_to_arr() {
        let s = String::from("-----------------------------------------------X------");
        let arr = str_to_arr(s.as_str());

        let target = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', 'X', '-', '-', '-', '-', '-', '-'],
        ];
        assert_eq!(arr, target)
    }

    #[test]
    pub fn test_str_to_arr_2() {
        let s = String::from("---------------------------------------------X--------");
        let arr = str_to_arr(s.as_str());

        let target = vec![
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['-', '-', '-', '-', '-', '-', '-', '-', '-'],
            vec!['X', '-', '-', '-', '-', '-', '-', '-', '-'],
        ];
        assert_eq!(arr, target)
    }

    #[test]
    pub fn test_arr_to_str() {
        let s = String::from("123456789123456789123456789123456789123456789123456789");
        assert_eq!(arr_to_str(&str_to_arr(&s)), s);
    }
}
