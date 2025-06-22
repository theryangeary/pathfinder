#[cfg(test)]
pub mod test_utils {
    use crate::db::models::*;
    use crate::game::{conversion::SerializableBoard, Board, BoardGenerator, GameEngine, Trie};

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

    /// Creates a test game entry
    pub fn create_test_game_entry(user_id: &str, game_id: &str) -> DbGameEntry {
        DbGameEntry {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            game_id: game_id.to_string(),
            answers_data: "[]".to_string(),
            total_score: 0,
            completed: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Creates a new game entry for database insertion
    pub fn create_new_test_game_entry(user_id: &str, game_id: &str) -> NewGameEntry {
        NewGameEntry {
            user_id: user_id.to_string(),
            game_id: game_id.to_string(),
            answers_data: "[]".to_string(),
            total_score: 0,
            completed: false,
        }
    }

    pub fn create_test_board(letters: &str) -> Board {
        let mut board = Board::new();

        let letters = [
            [
                letters.chars().nth(0),
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

        for i in 0..4 {
            for j in 0..4 {
                board.set_tile(
                    i,
                    j,
                    letters[i][j].unwrap(),
                    1,
                    letters[i][j].unwrap() == '*',
                );
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
        let repository = crate::db::Repository::new(pool);

        // Create a test game engine using test_utils
        let (game_engine, _temp_file) = create_test_game_engine();

        let state = ApiState::new(repository, game_engine);
        let app = create_router(state.clone());

        (state, app)
    }

    /// Creates a test board generator
    pub fn create_test_board_generator() -> BoardGenerator {
        BoardGenerator::new()
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

    /// Creates a test trie with common words
    pub fn create_test_trie() -> Trie {
        Trie::from(vec![
            "test", "thing", "area", "stop", "the", "are", "tea", "set", "sed", "silo", "seed",
            "sold", "does", "word",
        ])
    }

    /// Creates test HTTP headers for API requests
    pub fn create_test_headers() -> axum::http::HeaderMap {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());
        headers.insert("origin", "http://localhost:3000".parse().unwrap());
        headers
    }

    /// Creates test HTTP headers with session cookie
    pub fn create_test_headers_with_session(session_token: &str) -> axum::http::HeaderMap {
        let mut headers = create_test_headers();
        headers.insert(
            "cookie",
            format!("session_token={}", session_token).parse().unwrap(),
        );
        headers
    }

    /// Helper to create a test date string
    pub fn create_test_date() -> String {
        "2024-01-01".to_string()
    }

    /// Helper to create a test date range
    pub fn create_test_date_range() -> (String, String) {
        ("2024-01-01".to_string(), "2024-01-07".to_string())
    }

    /// Creates a mock answer for testing
    pub fn create_test_answer(word: &str, score: i32) -> serde_json::Value {
        serde_json::json!({
            "word": word,
            "score": score,
            "path": [[0, 0], [0, 1], [0, 2], [0, 3]] // Simple horizontal path
        })
    }

    /// Creates multiple test answers
    pub fn create_test_answers() -> Vec<serde_json::Value> {
        vec![
            create_test_answer("test", 10),
            create_test_answer("thing", 15),
            create_test_answer("area", 12),
        ]
    }

    /// Helper for async database test setup
    #[cfg(feature = "database-tests")]
    pub async fn setup_test_database() -> sqlx::PgPool {
        use crate::db::setup_database;

        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://localhost/pathfinder_test".to_string());

        setup_database(&database_url).await.unwrap()
    }

    /// Helper for creating API test state
    #[cfg(feature = "database-tests")]
    pub async fn create_test_api_state() -> crate::http_api::ApiState {
        use crate::db::Repository;

        let pool = setup_test_database().await;
        let repository = Repository::new(pool);
        let (game_engine, _temp_file) = create_test_game_engine();

        crate::http_api::ApiState::new(repository, game_engine)
    }

    /// Helper for creating test HTTP requests
    pub fn create_test_request(
        method: axum::http::Method,
        uri: &str,
        body: Option<&str>,
    ) -> axum::http::Request<axum::body::Body> {
        let mut builder = axum::http::Request::builder().method(method).uri(uri);

        if let Some(_body_content) = body {
            builder = builder.header("content-type", "application/json");
        }

        builder
            .body(axum::body::Body::from(body.unwrap_or("").to_string()))
            .unwrap()
    }

    /// Helper for asserting JSON responses
    pub fn assert_json_response(
        response: &axum::http::Response<axum::body::Body>,
        expected_status: axum::http::StatusCode,
    ) {
        assert_eq!(response.status(), expected_status);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );
    }

    /// Helper for creating test error responses
    pub fn create_test_error_response(message: &str) -> serde_json::Value {
        serde_json::json!({
            "error": message
        })
    }
}
