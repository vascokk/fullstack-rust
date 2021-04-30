-- Your SQL goes here
CREATE TABLE game_state (
    id TEXT PRIMARY KEY NOT NULL ,
    board TEXT,
    user_1 TEXT,
    user_2 TEXT,
    winner BOOLEAN NOT NULL DEFAULT 'f',
    last_user_id TEXT,
    last_user_color TEXT(1),
    ended BOOLEAN NOT NULL DEFAULT 'f'
);

CREATE TABLE user (
    id TEXT PRIMARY KEY NOT NULL ,
    user_name TEXT NOT NULL ,
    user_color TEXT(1) NOT NULL
);

INSERT INTO user VALUES ('1', 'user1', 'X');
INSERT INTO user VALUES ('2', 'user2', 'O');