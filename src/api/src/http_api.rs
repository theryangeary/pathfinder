use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_http::cors::CorsLayer;

use crate::db::{Repository, conversions::AnswerStorage};
use crate::game::GameEngine;
use crate::game_generator::GameGenerator;

// HTTP API types (simpler than protobuf for frontend)
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiGame {
    pub id: String,
    pub date: String,
    pub board: ApiBoard,
    pub threshold_score: i32,
    pub sequence_number: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiBoard {
    pub tiles: Vec<Vec<ApiTile>>,
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub path: Vec<ApiPosition>,
    pub wildcard_constraints: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiPosition {
    pub row: i32,
    pub col: i32,
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
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitResponse {
    pub user_id: String,
    pub total_score: i32,
    pub stats: ApiGameStats,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiGameStats {
    pub total_players: i32,
    pub user_rank: i32,
    pub percentile: f32,
    pub average_score: i32,
    pub highest_score: i32,
}

#[derive(Clone)]
pub struct ApiState {
    pub repository: Repository,
    pub game_engine: GameEngine,
    pub game_generator: GameGenerator,
}

impl ApiState {
    pub fn new(repository: Repository, game_engine: GameEngine) -> Self {
        let game_generator = GameGenerator::new(repository.clone(), game_engine.clone());
        Self {
            repository,
            game_engine,
            game_generator,
        }
    }
}

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/api/game", get(get_daily_game))
        .route("/api/game/:date", get(get_game_by_date))
        .route("/api/validate", post(validate_answer))
        .route("/api/submit", post(submit_answers))
        .route("/api/user", post(create_user))
        .route("/api/game-entry/:game_id", get(get_game_entry))
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        .with_state(state)
}

// Route handlers
async fn get_daily_game(State(state): State<ApiState>) -> Result<Json<ApiGame>, StatusCode> {
    let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();
    get_game_by_date(Path(today), State(state)).await
}

async fn get_game_by_date(
    Path(date): Path<String>,
    State(state): State<ApiState>,
) -> Result<Json<ApiGame>, StatusCode> {
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

    // Parse board from JSON
    let serializable_board: crate::game::conversion::SerializableBoard = 
        serde_json::from_str(&db_game.board_data)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to API format
    let api_board = ApiBoard {
        tiles: serializable_board.rows.into_iter().map(|row| {
            row.tiles.into_iter().map(|tile| ApiTile {
                letter: tile.letter,
                points: tile.points,
                is_wildcard: tile.is_wildcard,
                row: tile.row,
                col: tile.col,
            }).collect()
        }).collect()
    };

    let api_game = ApiGame {
        id: db_game.id,
        date: db_game.date,
        board: api_board,
        threshold_score: db_game.threshold_score,
        sequence_number: db_game.sequence_number,
    };

    Ok(Json(api_game))
}

async fn validate_answer(
    State(state): State<ApiState>,
    Json(request): Json<ValidateRequest>,
) -> Result<Json<ValidateResponse>, StatusCode> {
    // Use the game engine to validate the word
    let is_valid = state.game_engine.validate_word(&request.word);
    
    let response = ValidateResponse {
        is_valid,
        score: if is_valid { request.word.len() as i32 * 2 } else { 0 },
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

async fn submit_answers(
    State(state): State<ApiState>,
    Json(request): Json<SubmitRequest>,
) -> Result<Json<SubmitResponse>, StatusCode> {
    // Validate and get existing user or create new one
    let user = match (request.user_id.as_ref(), request.cookie_token.as_ref()) {
        (Some(user_id), Some(cookie_token)) => {
            // Validate existing user by both ID and cookie token
            match state.repository.get_user_by_id(user_id).await {
                Ok(Some(existing_user)) if existing_user.cookie_token == *cookie_token => {
                    // Valid user - update last seen
                    let _ = state.repository.update_user_last_seen(&existing_user.id).await;
                    existing_user
                },
                _ => {
                    // Invalid user_id or cookie_token mismatch - create new user
                    create_new_user(&state).await?
                }
            }
        },
        (None, Some(cookie_token)) => {
            // Try to find user by cookie token only
            match state.repository.get_user_by_cookie(cookie_token).await {
                Ok(Some(existing_user)) => {
                    // Valid user - update last seen
                    let _ = state.repository.update_user_last_seen(&existing_user.id).await;
                    existing_user
                },
                _ => {
                    // Invalid cookie_token - create new user
                    create_new_user(&state).await?
                }
            }
        },
        _ => {
            // No user identification provided - create new user
            create_new_user(&state).await?
        }
    };

    let total_score: i32 = request.answers.iter().map(|a| a.score).sum();

    // Get today's game to store the entry against
    let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();
    let game = match state.repository.get_game_by_date(&today).await {
        Ok(Some(game)) => game,
        Ok(None) => return Err(StatusCode::NOT_FOUND), // Game should exist by now
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

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
        completed: true, // Assume completed when submitting all answers
    };

    let _game_entry = match state.repository.create_or_update_game_entry(new_entry).await {
        Ok(entry) => entry,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

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
) -> Result<Json<Option<Vec<ApiAnswer>>>, StatusCode> {
    println!("get_game_entry called - game_id: {}, params: {:?}", game_id, params);
    
    // Get user identification from query parameters
    let user = match (params.get("user_id"), params.get("cookie_token")) {
        (Some(user_id), Some(cookie_token)) => {
            println!("Validating user by ID: {} and cookie: {}", user_id, cookie_token);
            // Validate existing user by both ID and cookie token
            match state.repository.get_user_by_id(user_id).await {
                Ok(Some(existing_user)) => {
                    println!("Found user: {}, stored cookie: {}", existing_user.id, existing_user.cookie_token);
                    if existing_user.cookie_token == *cookie_token {
                        println!("Cookie tokens match!");
                        existing_user
                    } else {
                        println!("Cookie tokens don't match! Provided: {}, Stored: {}", cookie_token, existing_user.cookie_token);
                        return Ok(Json(None)) // Invalid user credentials
                    }
                },
                Ok(None) => {
                    println!("No user found with ID: {}", user_id);
                    return Err(StatusCode::UNAUTHORIZED) // Invalid user credentials
                }
                Err(e) => {
                    println!("Database error getting user by ID: {}", e);
                    return Ok(Json(None)) // Invalid user credentials
                }
            }
        },
        (None, Some(cookie_token)) => {
            println!("Validating user by cookie token only: {}", cookie_token);
            // Try to find user by cookie token only
            match state.repository.get_user_by_cookie(cookie_token).await {
                Ok(Some(existing_user)) => {
                    println!("Found user by cookie: {}", existing_user.id);
                    existing_user
                },
                Ok(None) => {
                    println!("No user found with cookie: {}", cookie_token);
                    return Err(StatusCode::UNAUTHORIZED) // Invalid cookie_token
                }
                Err(e) => {
                    println!("Database error getting user by cookie: {}", e);
                    return Ok(Json(None)) // Invalid cookie_token
                }
            }
        },
        _ => {
            println!("No user identification provided");
            return Ok(Json(None)) // No user identification provided
        }
    };

    println!("User found: {}", user.id);

    // Get the game entry for this user and game
    match state.repository.get_game_entry(&user.id, &game_id).await {
        Ok(Some(entry)) => {
            println!("Found game entry: {:?}", entry.answers_data);
            // Parse the answers from JSON using stable database format
            match AnswerStorage::deserialize_to_api_answers(&entry.answers_data) {
                Ok(answers) => {
                    println!("Parsed answers: {:?}", answers);
                    Ok(Json(Some(answers)))
                },
                Err(e) => {
                    println!("Failed to parse answers JSON: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        },
        Ok(None) => {
            println!("No game entry found for user {} and game {}", user.id, game_id);
            Ok(Json(None)) // No entry found
        },
        Err(e) => {
            println!("Error getting game entry: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_new_user(state: &ApiState) -> Result<crate::db::models::DbUser, StatusCode> {
    let new_user = crate::db::models::NewUser {
        cookie_token: uuid::Uuid::new_v4().to_string(),
    };
    
    state.repository.create_user(new_user).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}