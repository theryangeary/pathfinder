use crate::db::models::*;
use crate::game::{conversion::SerializableBoard, Board, BoardGenerator, GameEngine, Trie};
#[cfg(feature = "database-tests")]
use crate::http_api::ApiState;

#[cfg(feature = "database-tests")]
use axum::Router;
use chrono::Utc;

use tempfile::NamedTempFile;
use uuid::Uuid;

/// Creates a test user with default values
pub fn create_test_user() -> DbUser {
    DbUser {
        id: Uuid::new_v4().to_string(),
        cookie_token: format!("test_cookie_{}", Uuid::new_v4()),
        created_at: Utc::now(),
        last_seen: Utc::now(),
    }
}

/// Creates a new user for database insertion
pub fn create_new_test_user() -> NewUser {
    NewUser {
        cookie_token: format!("test_cookie_{}", Uuid::new_v4()),
    }
}

/// Creates a test game with default values
pub fn create_test_game() -> DbGame {
    let board = create_default_test_board();
    let serializable: SerializableBoard = (&board).into();
    DbGame {
        id: Uuid::new_v4().to_string(),
        date: "2024-01-01".to_string(),
        board_data: serde_json::to_string(&serializable).unwrap(),
        threshold_score: 100,
        sequence_number: 1,
        created_at: Utc::now(),
    }
}

/// Creates a new game for database insertion
pub fn create_new_test_game() -> NewGame {
    let board = create_default_test_board();
    let serializable: crate::game::conversion::SerializableBoard = (&board).into();
    NewGame {
        date: "2024-01-01".to_string(),
        board_data: serde_json::to_string(&serializable).unwrap(),
        threshold_score: 100,
        sequence_number: 1,
    }
}

pub fn create_test_board(letters: &str) -> Board {
    let mut board = Board::new();

    let letters = [
        [
            letters.chars().next(),
            letters.chars().nth(1),
            letters.chars().nth(2),
            letters.chars().nth(3),
        ],
        [
            letters.chars().nth(4),
            letters.chars().nth(5),
            letters.chars().nth(6),
            letters.chars().nth(7),
        ],
        [
            letters.chars().nth(8),
            letters.chars().nth(9),
            letters.chars().nth(10),
            letters.chars().nth(11),
        ],
        [
            letters.chars().nth(12),
            letters.chars().nth(13),
            letters.chars().nth(14),
            letters.chars().nth(15),
        ],
    ];

    for (i, row) in letters.iter().enumerate() {
        for (j, tile) in row.iter().enumerate() {
            board.set_tile(i, j, tile.unwrap(), 1, tile.unwrap() == '*');
        }
    }

    board
}

/// Creates a simple 4x4 test board with known letters
pub fn create_default_test_board() -> Board {
    create_test_board("testh*ngar*astop")
}

#[cfg(feature = "database-tests")]
pub async fn setup_app(pool: sqlx::Pool<sqlx::Postgres>) -> (ApiState, Router) {
    use crate::{http_api::create_secure_router, security::SecurityConfig};

    let repository = crate::db::Repository::new(pool);

    // Create a test game engine using test_utils
    let (game_engine, _temp_file) = create_test_game_engine();

    let state = ApiState::new(repository, game_engine);
    let app = create_secure_router(state.clone(), SecurityConfig::default());

    (state, app)
}



pub fn create_test_board_data() -> String {
    // Use test_utils board and serialize it
    let board = create_default_test_board();
    let serializable: SerializableBoard = (&board).into();
    serde_json::to_string(&serializable).unwrap()
}

/// Creates a temporary wordlist file for testing
pub fn create_test_wordlist() -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    use std::io::Write;
    writeln!(file, "test").unwrap();
    writeln!(file, "thing").unwrap();
    writeln!(file, "area").unwrap();
    writeln!(file, "stop").unwrap();
    writeln!(file, "the").unwrap();
    writeln!(file, "are").unwrap();
    writeln!(file, "tea").unwrap();
    writeln!(file, "set").unwrap();
    writeln!(file, "sed").unwrap();
    writeln!(file, "silo").unwrap();
    writeln!(file, "seed").unwrap();
    writeln!(file, "sold").unwrap();
    writeln!(file, "does").unwrap();
    writeln!(file, "word").unwrap();
    file
}

/// Creates a test game engine with a temporary wordlist
pub fn create_test_game_engine() -> (GameEngine, NamedTempFile) {
    let wordlist_file = create_test_wordlist();
    let game_engine = GameEngine::new(wordlist_file.path().to_path_buf());
    (game_engine, wordlist_file)
}





/// Helper for creating test HTTP requests
pub fn create_test_request(
    method: axum::http::Method,
    uri: &str,
    body: Option<&str>,
) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder().method(method).uri(uri);
    builder = builder.header("referer", "http://localhost");

    if let Some(_body_content) = body {
        builder = builder.header("content-type", "application/json");
    }

    builder
        .body(axum::body::Body::from(body.unwrap_or("").to_string()))
        .unwrap()
}
