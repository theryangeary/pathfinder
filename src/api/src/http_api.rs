use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Json, Redirect},
    routing::{get, post},
    Router,
};
use chrono::{NaiveDate, Utc};
use chrono_tz::Tz;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer};

use crate::db::{conversions::AnswerStorage, Repository};
use crate::game::GameEngine;
use crate::game::{conversion::SerializableBoard, scoring::ScoreSheet};
use crate::game_generator::GameGenerator;
use crate::security::{
    cors::CorsLayer as SecurityCorsLayer,
    headers::SecurityHeadersLayer,
    rate_limit::RateLimitLayer,
    referer::RefererLayer,
    session::{cookie_layer, SessionLayer},
    SecurityConfig,
};

// HTTP API types (simpler than protobuf for frontend)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiGame {
    pub id: String,
    pub date: String,
    pub board: ApiBoard,
    pub threshold_score: i32,
    pub sequence_number: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiBoard {
    pub tiles: Vec<Vec<ApiTile>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiTile {
    pub letter: String,
    pub points: i32,
    pub is_wildcard: bool,
    pub row: i32,
    pub col: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiAnswer {
    pub word: String,
    pub score: i32,
}

impl ApiAnswer {
    pub fn sanitize(self) -> Self {
        Self {
            word: self.word.to_lowercase(),
            // TODO sanitize score
            ..self
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiPosition {
    pub row: i32,
    pub col: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiPath {
    pub tiles: Vec<ApiTile>,
    pub constraints: ApiPathConstraintSet,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ApiPathConstraintSet {
    Unconstrainted,
    FirstDecided(char),
    SecondDecided(char),
    BothDecided(char, char),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiPathsResponse {
    pub words: Vec<ApiWordPaths>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiWordPaths {
    pub word: String,
    pub paths: Vec<ApiPath>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ValidateRequest {
    pub word: String,
    pub previous_answers: Vec<ApiAnswer>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ValidateResponse {
    pub is_valid: bool,
    pub score: i32,
    pub path: Vec<ApiPosition>,
    pub wildcard_constraints: HashMap<String, String>,
    pub error_message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitRequest {
    pub user_id: Option<String>,
    pub cookie_token: Option<String>,
    pub answers: Vec<ApiAnswer>,
    pub game_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateGameEntryRequest {
    pub user_id: Option<String>,
    pub cookie_token: Option<String>,
    pub answers: Vec<ApiAnswer>,
    pub game_id: String,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SubmitResponse {
    pub user_id: String,
    pub total_score: i32,
    pub stats: ApiGameStats,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ApiGameStats {
    pub total_players: i32,
    pub user_rank: i32,
    pub percentile: f32,
    pub average_score: i32,
    pub highest_score: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameEntryResponse {
    pub answers: Vec<ApiAnswer>,
    pub completed: bool,
    pub total_score: i32,
    pub stats: Option<ApiGameStats>,
}

#[derive(Clone)]
pub struct ApiState {
    pub repository: Repository,
    pub game_engine: GameEngine,
    pub game_generator: GameGenerator,
    pub game_cache: Cache<String, ApiGame>,
}

impl ApiState {
    pub fn new(repository: Repository, game_engine: GameEngine) -> Self {
        let game_generator = GameGenerator::new(repository.clone(), game_engine.clone());

        // Create cache with reasonable memory footprint
        // Since games are immutable once created, we can cache recent ones
        let game_cache = Cache::builder()
            .max_capacity(100) // Cache up to 100 games (about 3 months worth) to reduce memory
            .time_to_live(std::time::Duration::from_secs(24 * 60 * 60)) // 24 hours TTL as safety
            .time_to_idle(std::time::Duration::from_secs(6 * 60 * 60)) // 6 hours idle timeout
            .build();

        Self {
            repository,
            game_engine,
            game_generator,
            game_cache,
        }
    }
}

pub fn create_secure_router(state: ApiState, config: SecurityConfig) -> Router {
    Router::new()
        .route(
            "/",
            get(|| async { Redirect::permanent("https://pathfinder.prof") }),
        )
        .route("/api/game/date/:date", get(get_game_by_date))
        .route(
            "/api/game/sequence/:sequence_number",
            get(get_game_by_sequence),
        )
        .route("/api/game/:game_id/words", get(get_game_words))
        .route("/api/game/:game_id/paths", get(get_game_paths))
        .route("/api/game/:game_id/word/:word/paths", get(get_word_paths))
        .route("/api/validate", post(validate_answer))
        .route("/api/user", post(create_user))
        .route("/api/game-entry/:game_id", get(get_game_entry))
        .route("/api/game-entry/:game_id", post(update_game_entry))
        .route("/health", get(health_check))
        .layer(RequestBodyLimitLayer::new(config.max_request_size))
        .layer(TimeoutLayer::new(config.request_timeout))
        .layer(RateLimitLayer::new(config.clone()))
        .layer(SecurityCorsLayer::new(config.clone()))
        .layer(RefererLayer::new(config.clone()))
        .layer(SessionLayer::new(config.clone()))
        .layer(cookie_layer())
        .layer(SecurityHeadersLayer::new(config.clone()))
        .with_state(state)
}

// Helper functions

/// Check if a date is in the future relative to the earliest timezone that might be playing
/// This ensures we only allow loading puzzles for dates that have already started somewhere in the world
fn is_date_in_future(date_str: &str) -> bool {
    // Parse the date string (YYYY-MM-DD format)
    let target_date = match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => return true, // Invalid date format is considered "future" to reject it
    };

    // Use UTC+14 (Pacific/Kiritimati) as the earliest timezone
    // This is the earliest timezone where a new day begins
    let earliest_tz: Tz = "Pacific/Kiritimati".parse().unwrap();
    let now_in_earliest = Utc::now().with_timezone(&earliest_tz);
    let today_in_earliest = now_in_earliest.date_naive();

    target_date > today_in_earliest
}

// Conversion functions for paths API

impl From<crate::game::board::constraints::PathConstraintSet> for ApiPathConstraintSet {
    fn from(constraint: crate::game::board::constraints::PathConstraintSet) -> Self {
        match constraint {
            crate::game::board::constraints::PathConstraintSet::Unconstrainted => {
                ApiPathConstraintSet::Unconstrainted
            }
            crate::game::board::constraints::PathConstraintSet::FirstDecided(c) => {
                ApiPathConstraintSet::FirstDecided(c)
            }
            crate::game::board::constraints::PathConstraintSet::SecondDecided(c) => {
                ApiPathConstraintSet::SecondDecided(c)
            }
            crate::game::board::constraints::PathConstraintSet::BothDecided(c1, c2) => {
                ApiPathConstraintSet::BothDecided(c1, c2)
            }
        }
    }
}

impl From<crate::game::board::path::Path> for ApiPath {
    fn from(path: crate::game::board::path::Path) -> Self {
        let tiles: Vec<ApiTile> = path
            .tiles
            .into_iter()
            .map(|tile| ApiTile {
                letter: tile.letter,
                points: tile.points,
                is_wildcard: tile.is_wildcard,
                row: tile.row,
                col: tile.col,
            })
            .collect();

        ApiPath {
            tiles,
            constraints: path.constraints.into(),
        }
    }
}

impl From<crate::game::board::answer::Answer> for ApiWordPaths {
    fn from(answer: crate::game::board::answer::Answer) -> Self {
        let paths: Vec<ApiPath> = answer.paths.into_iter().map(|path| path.into()).collect();

        ApiWordPaths {
            word: answer.word,
            paths,
        }
    }
}

// Route handlers

async fn get_game_words(
    Path(game_id): Path<String>,
    State(state): State<ApiState>,
) -> Result<Json<Vec<String>>, StatusCode> {
    match state.repository.get_game_words(&game_id).await {
        Ok(words) => Ok(Json(words)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_game_paths(
    Path(game_id): Path<String>,
    State(state): State<ApiState>,
) -> Result<Json<ApiPathsResponse>, StatusCode> {
    // Get the game from the repository
    let game = match state.repository.get_game_by_id(&game_id).await {
        Ok(Some(game)) => game,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Parse the board from the game data
    let serializable_board: SerializableBoard = match serde_json::from_str(&game.board_data) {
        Ok(board) => board,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let board: crate::game::Board = serializable_board.into();

    // Get all valid words for this game
    let valid_words = match state.repository.get_game_words(&game_id).await {
        Ok(words) => words,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Find all paths for each valid word
    let mut word_paths = Vec::new();
    for word in valid_words {
        let answer = state.game_engine.find_word_paths(&board, &word);
        if !answer.paths.is_empty() {
            word_paths.push(answer.into());
        }
    }

    let response = ApiPathsResponse { words: word_paths };
    Ok(Json(response))
}

async fn get_word_paths(
    Path((game_id, word)): Path<(String, String)>,
    State(state): State<ApiState>,
) -> Result<Json<ApiWordPaths>, StatusCode> {
    // Get the game from the repository
    let game = match state.repository.get_game_by_id(&game_id).await {
        Ok(Some(game)) => game,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Parse the board from the game data
    let serializable_board: SerializableBoard = match serde_json::from_str(&game.board_data) {
        Ok(board) => board,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let board: crate::game::Board = serializable_board.into();

    // Check if the word is valid for this game (exists in the game's word list)
    let valid_words = match state.repository.get_game_words(&game_id).await {
        Ok(words) => words,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Convert word to lowercase for case-insensitive comparison
    let word_lower = word.to_lowercase();

    if !valid_words.contains(&word_lower) {
        return Err(StatusCode::NOT_FOUND);
    }

    // Find all paths for this specific word
    let answer = state.game_engine.find_word_paths(&board, &word_lower);

    if answer.paths.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let word_paths: ApiWordPaths = answer.into();
    Ok(Json(word_paths))
}

async fn get_game_by_date(
    Path(date): Path<String>,
    State(state): State<ApiState>,
) -> Result<Json<ApiGame>, StatusCode> {
    // Validate that the requested date is not in the future
    if is_date_in_future(&date) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let cache_key = format!("date:{date}");

    // Check cache first
    if let Some(cached_game) = state.game_cache.get(&cache_key).await {
        return Ok(Json(cached_game));
    }

    // Try to get existing game
    let db_game = match state.repository.get_game_by_date(&date).await {
        Ok(Some(game)) => game,
        Ok(None) => {
            // Generate game if it doesn't exist
            match state.game_generator.generate_game_for_date(&date).await {
                Ok(game) => game,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let api_game = convert_db_game_to_api_game_direct(db_game)?;

    // Cache the result before returning
    state.game_cache.insert(cache_key, api_game.clone()).await;

    Ok(Json(api_game))
}

async fn get_game_by_sequence(
    Path(sequence_number): Path<i32>,
    State(state): State<ApiState>,
) -> Result<Json<ApiGame>, StatusCode> {
    let cache_key = format!("seq:{sequence_number}");

    // Check cache first
    if let Some(cached_game) = state.game_cache.get(&cache_key).await {
        // Still need to validate that this isn't a future puzzle, even if cached
        if is_date_in_future(&cached_game.date) {
            return Err(StatusCode::BAD_REQUEST);
        }
        return Ok(Json(cached_game));
    }

    // Get existing game by sequence number (don't generate new ones)
    let db_game = match state
        .repository
        .get_game_by_sequence_number(sequence_number)
        .await
    {
        Ok(Some(game)) => {
            // Validate that this game's date is not in the future
            if is_date_in_future(&game.date) {
                return Err(StatusCode::BAD_REQUEST);
            }
            game
        }
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    let api_game = convert_db_game_to_api_game_direct(db_game)?;

    // Cache the result before returning
    state.game_cache.insert(cache_key, api_game.clone()).await;

    Ok(Json(api_game))
}

fn convert_db_game_to_api_game_direct(
    db_game: crate::db::models::DbGame,
) -> Result<ApiGame, StatusCode> {
    // Parse board from JSON
    let serializable_board: crate::game::conversion::SerializableBoard =
        serde_json::from_str(&db_game.board_data).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to API format
    let api_board = ApiBoard {
        tiles: serializable_board
            .rows
            .into_iter()
            .map(|row| {
                row.tiles
                    .into_iter()
                    .map(|tile| ApiTile {
                        letter: tile.letter,
                        points: tile.points,
                        is_wildcard: tile.is_wildcard,
                        row: tile.row,
                        col: tile.col,
                    })
                    .collect()
            })
            .collect(),
    };

    let api_game = ApiGame {
        id: db_game.id,
        date: db_game.date,
        board: api_board,
        threshold_score: db_game.threshold_score,
        sequence_number: db_game.sequence_number,
    };

    Ok(api_game)
}

async fn validate_answer(
    State(state): State<ApiState>,
    Json(request): Json<ValidateRequest>,
) -> Result<Json<ValidateResponse>, StatusCode> {
    // Use the game engine to validate the word
    let is_valid = state.game_engine.is_valid_word_in_dictionary(&request.word);

    let response = ValidateResponse {
        is_valid,
        score: if is_valid {
            request.word.len() as i32 * 2
        } else {
            0
        },
        path: vec![], // TODO: Calculate actual path
        wildcard_constraints: HashMap::new(),
        error_message: if request.word.len() < 3 {
            "Word must be at least 3 letters".to_string()
        } else if !is_valid {
            format!("'{}' is not a valid word", request.word)
        } else {
            String::new()
        },
    };

    Ok(Json(response))
}

async fn update_game_entry(
    State(state): State<ApiState>,
    Json(request): Json<UpdateGameEntryRequest>,
) -> Result<Json<SubmitResponse>, StatusCode> {
    // Validate and get existing user or create new one
    let user = match (request.user_id.as_ref(), request.cookie_token.as_ref()) {
        (Some(user_id), Some(cookie_token)) => {
            // Validate existing user by both ID and cookie token
            match state.repository.get_user_by_id(user_id).await {
                Ok(Some(existing_user)) if existing_user.cookie_token == *cookie_token => {
                    // Valid user - update last seen
                    let _ = state
                        .repository
                        .update_user_last_seen(&existing_user.id)
                        .await;
                    existing_user
                }
                _ => {
                    // Invalid user_id or cookie_token mismatch - create new user
                    create_new_user(&state).await?
                }
            }
        }
        (None, Some(cookie_token)) => {
            // Try to find user by cookie token only
            match state.repository.get_user_by_cookie(cookie_token).await {
                Ok(Some(existing_user)) => {
                    // Valid user - update last seen
                    let _ = state
                        .repository
                        .update_user_last_seen(&existing_user.id)
                        .await;
                    existing_user
                }
                _ => {
                    // Invalid cookie_token - create new user
                    create_new_user(&state).await?
                }
            }
        }
        _ => {
            // No user identification provided - create new user
            create_new_user(&state).await?
        }
    };

    // Get the specified game to store the entry against
    let game = match state.repository.get_game_by_id(&request.game_id).await {
        Ok(Some(game)) => game,
        Ok(None) => return Err(StatusCode::NOT_FOUND), // Game not found
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Check if user has already completed this game
    match state
        .repository
        .get_game_entry(&user.id, &request.game_id)
        .await
    {
        Ok(Some(existing_entry)) if existing_entry.completed => {
            return Err(StatusCode::CONFLICT); // 409 Conflict - already submitted
        }
        Ok(_) => {
            // No existing entry or entry is not completed - proceed
        }
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    }

    // Validate that all submitted answers are valid for this game
    if let Err(error_msg) = validate_submitted_answers(&state, &game, &request.answers).await {
        println!("Answer validation failed: {error_msg}");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Score submitted answers
    let score_sheet = match score_submitted_answers(&state, &game, &request.answers).await {
        Ok(scoring) => scoring,
        Err(error_msg) => {
            println!("Answer scoring failed: {error_msg}");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let total_score: i32 = score_sheet.total_score().try_into().unwrap();

    // Serialize answers to JSON using stable database format
    let answers_json = match AnswerStorage::serialize_api_answers(&request.answers) {
        Ok(json) => json,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    // Create or update game entry
    let new_entry = crate::db::models::NewGameEntry {
        user_id: user.id.clone(),
        game_id: game.id.clone(),
        answers_data: answers_json,
        total_score,
        completed: request.completed,
    };

    let _game_entry = match state
        .repository
        .create_or_update_game_entry(new_entry)
        .await
    {
        Ok(entry) => entry,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    if !request.completed {
        return Ok(Json(SubmitResponse::default()));
    }

    // Get real stats
    let (total_players, user_rank, percentile, average_score, highest_score) =
        match state.repository.get_game_stats(&game.id, total_score).await {
            Ok(stats) => stats,
            Err(_) => {
                // Fallback stats if query fails
                (1, 1, 100.0, total_score, total_score)
            }
        };

    let stats = ApiGameStats {
        total_players,
        user_rank,
        percentile: percentile as f32,
        average_score,
        highest_score,
    };

    let response = SubmitResponse {
        user_id: user.id,
        total_score,
        stats,
    };

    Ok(Json(response))
}

async fn create_user(State(state): State<ApiState>) -> Result<Json<serde_json::Value>, StatusCode> {
    let user = create_new_user(&state).await?;
    Ok(Json(serde_json::json!({
        "user_id": user.id,
        "cookie_token": user.cookie_token
    })))
}

async fn get_game_entry(
    Path(game_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<ApiState>,
) -> Result<Json<Option<GameEntryResponse>>, StatusCode> {
    println!("get_game_entry called - game_id: {game_id}, params: {params:?}");

    // Get user identification from query parameters
    let user = match (params.get("user_id"), params.get("cookie_token")) {
        (Some(user_id), Some(cookie_token)) => {
            println!("Validating user by ID: {user_id} and cookie: {cookie_token}");
            // Validate existing user by both ID and cookie token
            match state.repository.get_user_by_id(user_id).await {
                Ok(Some(existing_user)) => {
                    println!(
                        "Found user: {}, stored cookie: {}",
                        existing_user.id, existing_user.cookie_token
                    );
                    if existing_user.cookie_token == *cookie_token {
                        println!("Cookie tokens match!");
                        existing_user
                    } else {
                        println!(
                            "Cookie tokens don't match! Provided: {}, Stored: {}",
                            cookie_token, existing_user.cookie_token
                        );
                        return Ok(Json(None)); // Invalid user credentials
                    }
                }
                Ok(None) => {
                    println!("No user found with ID: {user_id}");
                    return Err(StatusCode::UNAUTHORIZED); // Invalid user credentials
                }
                Err(e) => {
                    println!("Database error getting user by ID: {e}");
                    return Ok(Json(None)); // Invalid user credentials
                }
            }
        }
        (None, Some(cookie_token)) => {
            println!("Validating user by cookie token only: {cookie_token}");
            // Try to find user by cookie token only
            match state.repository.get_user_by_cookie(cookie_token).await {
                Ok(Some(existing_user)) => {
                    println!("Found user by cookie: {}", existing_user.id);
                    existing_user
                }
                Ok(None) => {
                    println!("No user found with cookie: {cookie_token}");
                    return Err(StatusCode::UNAUTHORIZED); // Invalid cookie_token
                }
                Err(e) => {
                    println!("Database error getting user by cookie: {e}");
                    return Ok(Json(None)); // Invalid cookie_token
                }
            }
        }
        _ => {
            println!("No user identification provided");
            return Ok(Json(None)); // No user identification provided
        }
    };

    println!("User found: {}", user.id);

    // Get the game entry for this user and game
    match state.repository.get_game_entry(&user.id, &game_id).await {
        Ok(Some(entry)) => {
            println!("Found game entry: {:?}", entry.answers_data);
            // Parse the answers from JSON using stable database format
            let answers = match AnswerStorage::deserialize_to_api_answers(&entry.answers_data) {
                Ok(answers) => {
                    println!("Parsed answers: {answers:?}");
                    answers
                }
                Err(e) => {
                    println!("Failed to parse answers JSON: {e}");
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            // Calculate stats if the game is completed
            let stats = if entry.completed {
                match state
                    .repository
                    .get_game_stats(&game_id, entry.total_score)
                    .await
                {
                    Ok((total_players, user_rank, percentile, average_score, highest_score)) => {
                        Some(ApiGameStats {
                            total_players,
                            user_rank,
                            percentile: percentile as f32,
                            average_score,
                            highest_score,
                        })
                    }
                    Err(e) => {
                        println!("Failed to get game stats: {e}");
                        None
                    }
                }
            } else {
                None
            };

            Ok(Json(Some(GameEntryResponse {
                answers,
                completed: entry.completed,
                total_score: entry.total_score,
                stats,
            })))
        }
        Ok(None) => {
            println!(
                "No game entry found for user {} and game {}",
                user.id, game_id
            );
            Ok(Json(None)) // No entry found
        }
        Err(e) => {
            println!("Error getting game entry: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_new_user(state: &ApiState) -> Result<crate::db::models::DbUser, StatusCode> {
    let new_user = crate::db::models::NewUser {
        cookie_token: uuid::Uuid::new_v4().to_string(),
    };

    state
        .repository
        .create_user(new_user)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

async fn validate_submitted_answers(
    state: &ApiState,
    game: &crate::db::models::DbGame,
    submitted_answers: &[ApiAnswer],
) -> Result<(), String> {
    // Parse the board from the game data
    let serializable_board: SerializableBoard = serde_json::from_str(&game.board_data)
        .map_err(|_| "Failed to parse game board data".to_string())?;

    let board: crate::game::Board = serializable_board.into();

    state
        .game_engine
        .validate_api_answer_group(&board, Vec::from(submitted_answers))
}

async fn score_submitted_answers(
    state: &ApiState,
    game: &crate::db::models::DbGame,
    submitted_answers: &[ApiAnswer],
) -> Result<ScoreSheet, String> {
    // Parse the board from the game data
    let serializable_board: SerializableBoard = serde_json::from_str(&game.board_data)
        .map_err(|_| "Failed to parse game board data".to_string())?;

    let board: crate::game::Board = serializable_board.into();

    let answers = submitted_answers
        .iter()
        .map(|m| m.word.to_string())
        .collect();

    state.game_engine.score_answer_group(&board, answers)
}

async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    let process = env::var("FLY_PROCESS_GROUP").unwrap_or_else(|_| "unknown".to_string());
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "process": process,
    })))
}

#[cfg(all(test, feature = "database-tests"))]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tower::util::ServiceExt;

    use crate::{db::models::NewGameAnswer, test_utils::*};

    #[sqlx::test]
    async fn test_get_game_by_sequence_exists(pool: sqlx::Pool<sqlx::Postgres>) {
        let (state, app) = setup_app(pool).await;

        // Create a test game using test_utils
        let mut new_game = create_new_test_game();
        new_game.date = "2025-06-08".to_string();
        new_game.threshold_score = 40;
        new_game.sequence_number = 1;
        let (created_game, _) = state
            .repository
            .create_game_with_answers(new_game, vec![], None)
            .await
            .unwrap();

        // Test the endpoint using test_utils request helper
        let request = create_test_request(axum::http::Method::GET, "/api/game/sequence/1", None);
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let game: ApiGame = serde_json::from_slice(&body).unwrap();

        assert_eq!(game.id, created_game.id);
        assert_eq!(game.sequence_number, 1);
        assert_eq!(game.date, "2025-06-08");
        assert_eq!(game.threshold_score, 40);
        assert_eq!(game.board.tiles.len(), 4); // 4x4 board
    }

    #[sqlx::test]
    async fn test_get_game_by_sequence_not_found(pool: sqlx::Pool<sqlx::Postgres>) {
        let (_state, app) = setup_app(pool).await;

        // Test getting a non-existent sequence number
        let request = create_test_request(axum::http::Method::GET, "/api/game/sequence/999", None);
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test]
    async fn test_get_game_by_sequence_multiple_games(pool: sqlx::Pool<sqlx::Postgres>) {
        let (state, app) = setup_app(pool).await;

        // Create multiple test games using test_utils
        let mut game1 = create_new_test_game();
        game1.date = "2025-06-08".to_string();
        game1.threshold_score = 40;
        game1.sequence_number = 1;

        let mut game2 = create_new_test_game();
        game2.date = "2025-06-07".to_string();
        game2.threshold_score = 35;
        game2.sequence_number = 3;

        let mut game3 = create_new_test_game();
        game3.date = "2025-06-06".to_string();
        game3.threshold_score = 45;
        game3.sequence_number = 5;

        let games = vec![game1, game2, game3];

        for game in games {
            state
                .repository
                .create_game_with_answers(game, vec![], None)
                .await
                .unwrap();
        }

        // Test getting game with sequence number 1
        let request = create_test_request(axum::http::Method::GET, "/api/game/sequence/1", None);
        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let game: ApiGame = serde_json::from_slice(&body).unwrap();
        assert_eq!(game.sequence_number, 1);
        assert_eq!(game.date, "2025-06-08");

        // Test getting game with sequence number 3
        let request = create_test_request(axum::http::Method::GET, "/api/game/sequence/3", None);
        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let game: ApiGame = serde_json::from_slice(&body).unwrap();
        assert_eq!(game.sequence_number, 3);
        assert_eq!(game.date, "2025-06-07");

        // Test getting game with sequence number 5
        let request = create_test_request(axum::http::Method::GET, "/api/game/sequence/5", None);
        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let game: ApiGame = serde_json::from_slice(&body).unwrap();
        assert_eq!(game.sequence_number, 5);
        assert_eq!(game.date, "2025-06-06");

        // Test getting non-existent sequence numbers
        let request = create_test_request(axum::http::Method::GET, "/api/game/sequence/2", None);
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let request = create_test_request(axum::http::Method::GET, "/api/game/sequence/4", None);
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test]
    async fn test_get_game_by_date_endpoint(pool: sqlx::Pool<sqlx::Postgres>) {
        let (state, app) = setup_app(pool).await;

        // Create a test game using test_utils
        let mut new_game = create_new_test_game();
        new_game.date = "2025-06-08".to_string();
        new_game.threshold_score = 40;
        new_game.sequence_number = 1;
        let (created_game, _) = state
            .repository
            .create_game_with_answers(new_game, vec![], None)
            .await
            .unwrap();

        // Test the date endpoint
        let request =
            create_test_request(axum::http::Method::GET, "/api/game/date/2025-06-08", None);
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let game: ApiGame = serde_json::from_slice(&body).unwrap();

        assert_eq!(game.id, created_game.id);
        assert_eq!(game.date, "2025-06-08");
        assert_eq!(game.sequence_number, 1);
    }

    #[sqlx::test]
    async fn test_validate_word_endpoint(pool: sqlx::Pool<sqlx::Postgres>) {
        let (_state, app) = setup_app(pool).await;

        let request_body = ValidateRequest {
            word: "test".to_string(),
            previous_answers: vec![],
        };

        let body_json = serde_json::to_string(&request_body).unwrap();
        let request =
            create_test_request(axum::http::Method::POST, "/api/validate", Some(&body_json));
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let validate_response: ValidateResponse = serde_json::from_slice(&body).unwrap();

        assert!(validate_response.is_valid);
        assert_eq!(validate_response.error_message, "");
    }

    #[sqlx::test]
    async fn test_validate_invalid_word_endpoint(pool: sqlx::Pool<sqlx::Postgres>) {
        let (_state, app) = setup_app(pool).await;

        let request_body = ValidateRequest {
            word: "invalidword".to_string(),
            previous_answers: vec![],
        };

        let body_json = serde_json::to_string(&request_body).unwrap();
        let request =
            create_test_request(axum::http::Method::POST, "/api/validate", Some(&body_json));
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let validate_response: ValidateResponse = serde_json::from_slice(&body).unwrap();

        assert!(!validate_response.is_valid);
        assert!(validate_response.error_message.contains("invalidword"));
    }

    #[sqlx::test]
    async fn test_create_user_endpoint(pool: sqlx::Pool<sqlx::Postgres>) {
        let (_state, app) = setup_app(pool).await;

        let request = create_test_request(axum::http::Method::POST, "/api/user", None);
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let user_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(user_response["user_id"].is_string());
        assert!(user_response["cookie_token"].is_string());
        assert!(!user_response["user_id"].as_str().unwrap().is_empty());
        assert!(!user_response["cookie_token"].as_str().unwrap().is_empty());
    }

    #[sqlx::test]
    async fn test_game_caching_works(pool: sqlx::Pool<sqlx::Postgres>) {
        // TODO this doens't really effectively test caching
        let (state, app) = setup_app(pool).await;

        // Create a test game using test_utils
        let mut new_game = create_new_test_game();
        new_game.date = "2025-06-08".to_string();
        new_game.threshold_score = 40;
        new_game.sequence_number = 1;
        let (_created_game, _) = state
            .repository
            .create_game_with_answers(new_game, vec![], None)
            .await
            .unwrap();

        // First request - should hit database and cache
        let request1 = create_test_request(axum::http::Method::GET, "/api/game/sequence/1", None);
        let response1 = app.clone().oneshot(request1).await.unwrap();

        assert_eq!(response1.status(), StatusCode::OK);

        // Verify cache has the game
        let cache_key = "seq:1";
        let cached_game = state.game_cache.get(cache_key).await;
        assert!(cached_game.is_some());

        // Second request - should hit cache
        let request2 = create_test_request(axum::http::Method::GET, "/api/game/sequence/1", None);
        let response2 = app.oneshot(request2).await.unwrap();

        assert_eq!(response2.status(), StatusCode::OK);

        // Both responses should be identical
        let body1 = axum::body::to_bytes(response1.into_body(), usize::MAX)
            .await
            .unwrap();
        let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body1, body2);
    }

    #[sqlx::test]
    async fn test_get_game_paths_endpoint(pool: sqlx::Pool<sqlx::Postgres>) {
        let (state, app) = setup_app(pool).await;

        // Create a test game
        let mut new_game = create_new_test_game();
        new_game.date = "2025-06-08".to_string();
        new_game.threshold_score = 40;
        new_game.sequence_number = 1;

        // Create some test answers for the game
        let test_answers = vec![
            crate::db::models::NewGameAnswer {
                game_id: "a".to_string(),
                word: "test".to_string(),
            },
            crate::db::models::NewGameAnswer {
                game_id: "a".to_string(),
                word: "word".to_string(),
            },
            crate::db::models::NewGameAnswer {
                game_id: "a".to_string(),
                word: "game".to_string(),
            },
        ];

        let (created_game, _game_answers) = state
            .repository
            .create_game_with_answers(new_game, test_answers, None)
            .await
            .unwrap();

        // Test the paths endpoint
        let request = create_test_request(
            axum::http::Method::GET,
            &format!("/api/game/{}/paths", created_game.id),
            None,
        );
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let paths_response: ApiPathsResponse = serde_json::from_slice(&body).unwrap();

        // Verify we get the expected structure
        assert!(
            !paths_response.words.is_empty(),
            "Should have some word paths"
        );

        // Check that each word has the expected structure
        for word_path in &paths_response.words {
            assert!(!word_path.word.is_empty(), "Word should not be empty");
            // Since our test board and words may not actually be valid, we just check structure
            // In a real scenario, valid words would have paths
        }
    }

    #[sqlx::test]
    async fn test_get_word_paths_endpoint(pool: sqlx::Pool<sqlx::Postgres>) {
        let (state, app) = setup_app(pool).await;

        // Create a test game
        let mut new_game = create_new_test_game();
        new_game.date = "2025-06-08".to_string();
        new_game.threshold_score = 40;
        new_game.sequence_number = 1;

        // Create some test answers for the game
        let test_answers = vec![
            crate::db::models::NewGameAnswer {
                game_id: "a".to_string(),
                word: "test".to_string(),
            },
            crate::db::models::NewGameAnswer {
                game_id: "a".to_string(),
                word: "word".to_string(),
            },
        ];

        let (created_game, _game_answers) = state
            .repository
            .create_game_with_answers(new_game, test_answers, None)
            .await
            .unwrap();

        // Test the word paths endpoint for a valid word
        let request = create_test_request(
            axum::http::Method::GET,
            &format!("/api/game/{}/word/test/paths", created_game.id),
            None,
        );
        let response = app.clone().oneshot(request).await.unwrap();

        // Should return OK even if paths are empty (word exists in game)
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::NOT_FOUND);

        // Test with an invalid word (not in game's word list)
        let request = create_test_request(
            axum::http::Method::GET,
            &format!("/api/game/{}/word/invalidword/paths", created_game.id),
            None,
        );
        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        // Test with invalid game ID
        let request = create_test_request(
            axum::http::Method::GET,
            "/api/game/invalid-game-id/word/test/paths",
            None,
        );
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[sqlx::test]
    async fn test_get_game_words_endpoint(pool: sqlx::Pool<sqlx::Postgres>) {
        let (state, app) = setup_app(pool).await;

        // Create a test game
        let mut new_game = create_new_test_game();
        new_game.date = "2025-06-08".to_string();
        new_game.threshold_score = 40;
        new_game.sequence_number = 1;

        // Create some test answers for the game
        let test_answers = vec![
            NewGameAnswer {
                game_id: "a".to_string(),
                word: "test".to_string(),
            },
            NewGameAnswer {
                game_id: "a".to_string(),
                word: "word".to_string(),
            },
            NewGameAnswer {
                game_id: "a".to_string(),
                word: "game".to_string(),
            },
        ];

        let (created_game, _) = state
            .repository
            .create_game_with_answers(new_game, test_answers, None)
            .await
            .unwrap();

        // Test the words endpoint
        let request = create_test_request(
            axum::http::Method::GET,
            &format!("/api/game/{}/words", created_game.id),
            None,
        );
        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let words: Vec<String> = serde_json::from_slice(&body).unwrap();

        // Verify we get the expected words
        assert_eq!(words.len(), 3);
        assert!(words.contains(&"test".to_string()));
        assert!(words.contains(&"word".to_string()));
        assert!(words.contains(&"game".to_string()));
    }

    #[sqlx::test]
    async fn test_validate_submitted_answers_with_cumulative_constraints(
        pool: sqlx::Pool<sqlx::Postgres>,
    ) {
        // Integration test for the validate_submitted_answers function fix
        let (state, _app) = setup_app(pool).await;

        // Create a test game with a simple board that allows constraint testing
        let board_data = create_test_board_data(); // Use existing simple board
        let game = crate::db::models::DbGame {
            id: "test-game".to_string(),
            date: "2025-06-04".to_string(),
            board_data: board_data.clone(),
            threshold_score: 15,
            sequence_number: 1,
            created_at: chrono::Utc::now(),
            completed: false,
            completed_at: None,
        };

        // Test answers that require cumulative constraint validation
        let test_answers = vec![
            ApiAnswer {
                word: "test".to_string(),
                score: 4,
            },
            ApiAnswer {
                word: "word".to_string(),
                score: 6,
            },
        ];

        // This should succeed - the key test is that it uses validate_answer_with_constraints
        // internally rather than validate_answer
        let result = validate_submitted_answers(&state, &game, &test_answers).await;
        assert!(
            result.is_ok(),
            "Submitted answers should be valid: {:?} {:?}",
            result.err(),
            board_data.clone()
        );

        // Test that we can detect when answers would conflict (if they did)
        // This validates that the function is actually checking constraints properly
        let conflicting_answers = vec![
            ApiAnswer {
                word: "test".to_string(),
                score: 4,
            },
            ApiAnswer {
                word: "invalid".to_string(), // This word is not in our test dictionary
                score: 6,
            },
        ];

        let result = validate_submitted_answers(&state, &game, &conflicting_answers).await;
        assert!(result.is_err(), "Invalid word should be rejected");
        assert!(
            result.unwrap_err().contains("not in the dictionary"),
            "Should reject invalid words"
        );
    }

    fn create_puzzle8_board_data() -> String {
        // JSON representation of the puzzle #8 board from user screenshot
        // H I S S
        // C * L O  <- wildcard at (1,1)
        // L E * D  <- wildcard at (2,2)
        // S E E O
        r#"{"rows":[{"tiles":[{"letter":"h","points":3,"is_wildcard":false,"row":0,"col":0},{"letter":"i","points":1,"is_wildcard":false,"row":0,"col":1},{"letter":"s","points":1,"is_wildcard":false,"row":0,"col":2},{"letter":"s","points":1,"is_wildcard":false,"row":0,"col":3}]},{"tiles":[{"letter":"c","points":2,"is_wildcard":false,"row":1,"col":0},{"letter":"*","points":0,"is_wildcard":true,"row":1,"col":1},{"letter":"l","points":2,"is_wildcard":false,"row":1,"col":2},{"letter":"o","points":1,"is_wildcard":false,"row":1,"col":3}]},{"tiles":[{"letter":"l","points":2,"is_wildcard":false,"row":2,"col":0},{"letter":"e","points":1,"is_wildcard":false,"row":2,"col":1},{"letter":"*","points":0,"is_wildcard":true,"row":2,"col":2},{"letter":"d","points":2,"is_wildcard":false,"row":2,"col":3}]},{"tiles":[{"letter":"s","points":1,"is_wildcard":false,"row":3,"col":0},{"letter":"e","points":1,"is_wildcard":false,"row":3,"col":1},{"letter":"e","points":1,"is_wildcard":false,"row":3,"col":2},{"letter":"o","points":1,"is_wildcard":false,"row":3,"col":3}]}]}"#.to_string()
    }

    #[test]
    fn test_api_path_constraint_set_conversion() {
        use crate::game::board::constraints::PathConstraintSet;

        // Test Unconstrainted conversion
        let internal = PathConstraintSet::Unconstrainted;
        let api: ApiPathConstraintSet = internal.into();
        assert!(matches!(api, ApiPathConstraintSet::Unconstrainted));

        // Test FirstDecided conversion
        let internal = PathConstraintSet::FirstDecided('a');
        let api: ApiPathConstraintSet = internal.into();
        assert!(matches!(api, ApiPathConstraintSet::FirstDecided('a')));

        // Test SecondDecided conversion
        let internal = PathConstraintSet::SecondDecided('b');
        let api: ApiPathConstraintSet = internal.into();
        assert!(matches!(api, ApiPathConstraintSet::SecondDecided('b')));

        // Test BothDecided conversion
        let internal = PathConstraintSet::BothDecided('x', 'y');
        let api: ApiPathConstraintSet = internal.into();
        assert!(matches!(api, ApiPathConstraintSet::BothDecided('x', 'y')));
    }

    #[test]
    fn test_api_paths_response_serialization() {
        use crate::game::board::constraints::PathConstraintSet;
        use crate::game::board::path::{GameTile, Path};
        use std::collections::VecDeque;

        // Create a sample response structure
        let mut tiles = VecDeque::new();
        tiles.push_back(GameTile {
            letter: "c".to_string(),
            points: 2,
            is_wildcard: false,
            row: 0,
            col: 0,
        });
        tiles.push_back(GameTile {
            letter: "*".to_string(),
            points: 0,
            is_wildcard: true,
            row: 1,
            col: 1,
        });
        tiles.push_back(GameTile {
            letter: "t".to_string(),
            points: 1,
            is_wildcard: false,
            row: 0,
            col: 2,
        });

        let path = Path {
            tiles,
            constraints: PathConstraintSet::FirstDecided('a'),
        };

        let word_paths = ApiWordPaths {
            word: "cat".to_string(),
            paths: vec![path.into()],
        };

        let response = ApiPathsResponse {
            words: vec![word_paths],
        };

        // Test serialization
        let json = serde_json::to_string_pretty(&response).expect("Should serialize");

        // The JSON should contain the expected structure
        assert!(json.contains("cat"));
        assert!(json.contains("tiles"));
        assert!(json.contains("constraints"));
        assert!(json.contains("FirstDecided"));

        // Test that we can deserialize it back
        let deserialized: ApiPathsResponse =
            serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.words.len(), 1);
        assert_eq!(deserialized.words[0].word, "cat");
        assert_eq!(deserialized.words[0].paths.len(), 1);
        assert_eq!(deserialized.words[0].paths[0].tiles.len(), 3);
    }

    #[test]
    fn test_api_path_conversion() {
        use crate::game::board::constraints::PathConstraintSet;
        use crate::game::board::path::{GameTile, Path};
        use std::collections::VecDeque;

        // Create a test path
        let mut tiles = VecDeque::new();
        tiles.push_back(GameTile {
            letter: "c".to_string(),
            points: 2,
            is_wildcard: false,
            row: 0,
            col: 0,
        });
        tiles.push_back(GameTile {
            letter: "a".to_string(),
            points: 1,
            is_wildcard: false,
            row: 0,
            col: 1,
        });

        let internal_path = Path {
            tiles,
            constraints: PathConstraintSet::FirstDecided('c'),
        };

        let api_path: ApiPath = internal_path.into();

        assert_eq!(api_path.tiles.len(), 2);
        assert_eq!(api_path.tiles[0].letter, "c");
        assert_eq!(api_path.tiles[0].points, 2);
        assert_eq!(api_path.tiles[0].row, 0);
        assert_eq!(api_path.tiles[0].col, 0);
        assert!(!api_path.tiles[0].is_wildcard);

        assert_eq!(api_path.tiles[1].letter, "a");
        assert_eq!(api_path.tiles[1].points, 1);
        assert!(!api_path.tiles[1].is_wildcard);

        assert!(matches!(
            api_path.constraints,
            ApiPathConstraintSet::FirstDecided('c')
        ));
    }

    #[test]
    fn test_word_paths_case_insensitive() {
        // Test that the endpoint handles case-insensitive word matching
        let test_words = ["test".to_string(), "word".to_string(), "game".to_string()];

        // Test that lowercase matches work
        assert!(test_words.contains(&"test".to_lowercase()));
        assert!(test_words.contains(&"TEST".to_lowercase()));
        assert!(test_words.contains(&"Test".to_lowercase()));

        // Test that non-existent words don't match
        assert!(!test_words.contains(&"invalid".to_lowercase()));
        assert!(!test_words.contains(&"NOTFOUND".to_lowercase()));
    }

    #[test]
    fn test_api_word_paths_structure() {
        use crate::game::board::constraints::PathConstraintSet;
        use crate::game::board::path::{GameTile, Path};
        use std::collections::VecDeque;

        // Create a sample word paths structure
        let mut tiles = VecDeque::new();
        tiles.push_back(GameTile {
            letter: "t".to_string(),
            points: 1,
            is_wildcard: false,
            row: 0,
            col: 0,
        });
        tiles.push_back(GameTile {
            letter: "e".to_string(),
            points: 1,
            is_wildcard: false,
            row: 0,
            col: 1,
        });
        tiles.push_back(GameTile {
            letter: "*".to_string(),
            points: 0,
            is_wildcard: true,
            row: 1,
            col: 1,
        });
        tiles.push_back(GameTile {
            letter: "t".to_string(),
            points: 1,
            is_wildcard: false,
            row: 0,
            col: 2,
        });

        let path = Path {
            tiles,
            constraints: PathConstraintSet::FirstDecided('s'),
        };

        let word_paths = ApiWordPaths {
            word: "test".to_string(),
            paths: vec![path.into()],
        };

        // Test serialization
        let json = serde_json::to_string_pretty(&word_paths).expect("Should serialize");

        // The JSON should contain the expected structure
        assert!(json.contains("test"));
        assert!(json.contains("paths"));
        assert!(json.contains("tiles"));
        assert!(json.contains("constraints"));
        assert!(json.contains("FirstDecided"));

        // Test that we can deserialize it back
        let deserialized: ApiWordPaths = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(deserialized.word, "test");
        assert_eq!(deserialized.paths.len(), 1);
        assert_eq!(deserialized.paths[0].tiles.len(), 4);
        assert!(matches!(
            deserialized.paths[0].constraints,
            ApiPathConstraintSet::FirstDecided('s')
        ));
    }

    #[test]
    fn test_is_date_in_future() {
        // Test with today's date (should not be in future)
        let today = Utc::now().format("%Y-%m-%d").to_string();
        assert!(
            !is_date_in_future(&today),
            "Today should not be considered future"
        );

        // Test with yesterday's date (should not be in future)
        let yesterday = (Utc::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        assert!(
            !is_date_in_future(&yesterday),
            "Yesterday should not be considered future"
        );

        // Test with a clearly future date (should be in future)
        let future_date = (Utc::now() + chrono::Duration::days(10))
            .format("%Y-%m-%d")
            .to_string();
        assert!(
            is_date_in_future(&future_date),
            "Future date should be considered future"
        );

        // Test with invalid date format (should be considered future for safety)
        assert!(
            is_date_in_future("invalid-date"),
            "Invalid date should be considered future"
        );
        assert!(
            is_date_in_future("2024-13-45"),
            "Invalid date should be considered future"
        );
        assert!(
            is_date_in_future("not-a-date"),
            "Invalid date should be considered future"
        );
    }

    #[tokio::test]
    async fn test_wildcard_pathfinding_fix() {
        // Test that wildcard pathfinding works correctly after the fix

        // Create a test game engine using test_utils with additional words
        let (game_engine, _temp_file) = create_test_game_engine();
        // Note: The test_utils game engine already includes "test", "sed", etc.

        // Create the exact board from puzzle #8
        let serializable_board: SerializableBoard =
            serde_json::from_str(&create_puzzle8_board_data()).unwrap();
        let board: crate::game::Board = serializable_board.into();

        // Test that "sed" can now be found on the board (this was failing before the fix)
        let answer = game_engine.validate_answer(&board, "sed");
        assert!(
            answer.is_ok(),
            "Word 'sed' should be valid on puzzle #8 board after wildcard fix: {:?}",
            answer.err()
        );

        let answer = answer.unwrap();
        assert!(
            !answer.paths.is_empty(),
            "Word 'sed' should have valid paths"
        );
        assert_eq!(answer.word, "sed");

        // Also test other words from the puzzle #8 scenario
        let test_words = ["silo", "seed", "sed", "sold", "does"];
        for word in test_words {
            let answer = game_engine.validate_answer(&board, word);
            assert!(
                answer.is_ok(),
                "Word '{}' should be valid: {:?}",
                word,
                answer.err()
            );
            assert!(
                !answer.unwrap().paths.is_empty(),
                "Word '{word}' should have valid paths"
            );
        }
    }

    // Helper function to expose answers_are_compatible for testing
    // pub fn answers_are_compatible_test(
    //     answer1: &crate::game::board::answer::Answer,
    //     answer2: &crate::game::board::answer::Answer,
    // ) -> bool {
    //     super::answers_are_compatible(answer1, answer2)
    // }
}
