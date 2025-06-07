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

use crate::db::Repository;
use crate::game::GameEngine;
use crate::game_generator::GameGenerator;
use crate::service::WordGameServiceImpl;
use crate::serialization::{SerializableAnswer, SerializablePosition};

// HTTP API types (simpler than protobuf for frontend)
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiGame {
    pub id: String,
    pub date: String,
    pub board: ApiBoard,
    pub threshold_score: i32,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiAnswer {
    pub word: String,
    pub score: i32,
    pub path: Vec<ApiPosition>,
    pub wildcard_constraints: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
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
        .route("/api/daily-game", get(get_daily_game))
        .route("/api/daily-game/:date", get(get_game_by_date))
        .route("/api/validate", post(validate_answer))
        .route("/api/submit", post(submit_answers))
        .route("/api/user", post(create_user))
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
    };

    Ok(Json(api_game))
}

async fn validate_answer(
    State(state): State<ApiState>,
    Json(request): Json<ValidateRequest>,
) -> Result<Json<ValidateResponse>, StatusCode> {
    // For now, return a simple validation
    // TODO: Implement full validation logic using game engine
    
    let response = ValidateResponse {
        is_valid: request.word.len() >= 3,
        score: if request.word.len() >= 3 { request.word.len() as i32 * 2 } else { 0 },
        path: vec![], // TODO: Calculate actual path
        wildcard_constraints: HashMap::new(),
        error_message: if request.word.len() < 3 { 
            "Word must be at least 3 letters".to_string() 
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
    // Create or get user
    let user = if let Some(user_id) = request.user_id {
        // Try to find existing user
        // For now, create a new one
        create_new_user(&state).await?
    } else {
        create_new_user(&state).await?
    };

    let total_score: i32 = request.answers.iter().map(|a| a.score).sum();

    // Mock stats for now
    let stats = ApiGameStats {
        total_players: 1,
        user_rank: 1,
        percentile: 100.0,
        average_score: total_score,
        highest_score: total_score,
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

async fn create_new_user(state: &ApiState) -> Result<crate::db::models::DbUser, StatusCode> {
    let new_user = crate::db::models::NewUser {
        cookie_token: uuid::Uuid::new_v4().to_string(),
    };
    
    state.repository.create_user(new_user).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}