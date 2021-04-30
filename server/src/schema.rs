table! {
    game_state (id) {
        id -> Text,
        board -> Nullable<Text>,
        user_1 -> Nullable<Text>,
        user_2 -> Nullable<Text>,
        winner -> Bool,
        last_user_id -> Nullable<Text>,
        last_user_color -> Nullable<Text>,
        ended -> Bool,
    }
}

table! {
    user (id) {
        id -> Text,
        user_name -> Text,
        user_color -> Text,
    }
}

allow_tables_to_appear_in_same_query!(game_state, user,);
