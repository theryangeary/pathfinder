use tonic::{Request, Response, Status};
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;

use crate::db::{Repository, models::{NewUser, NewGameEntry}};
use crate::game::GameEngine;
use crate::game_generator::GameGenerator;

// Import generated protobuf types
pub mod wordgame {
    tonic::include_proto!("wordgame");
}

use wordgame::{
    word_game_service_server::WordGameService,
    *,
};

pub struct WordGameServiceImpl {
    repository: Repository,
    game_engine: GameEngine,
    game_generator: GameGenerator,
}

impl WordGameServiceImpl {
    pub fn new(repository: Repository, game_engine: GameEngine) -> Self {
        let game_generator = GameGenerator::new(repository.clone(), game_engine.clone());
        Self {
            repository,
            game_engine,
            game_generator,
        }
    }
}

#[tonic::async_trait]
impl WordGameService for WordGameServiceImpl {
    async fn get_daily_game(
        &self,
        request: Request<GetDailyGameRequest>,
    ) -> Result<Response<GetDailyGameResponse>, Status> {
        let req = request.into_inner();
        
        // Use provided date or default to today
        let date = if req.date.is_empty() {
            Utc::now().date_naive().format("%Y-%m-%d").to_string()
        } else {
            req.date
        };

        // Try to get existing game
        let db_game = match self.repository.get_game_by_date(&date).await {
            Ok(Some(game)) => game,
            Ok(None) => {
                // Generate game if it doesn't exist
                match self.game_generator.generate_game_for_date(&date).await {
                    Ok(game) => game,
                    Err(e) => {
                        tracing::error!("Failed to generate game for date {}: {}", date, e);
                        return Err(Status::internal("Failed to generate daily game"));
                    }
                }
            }
            Err(e) => {
                tracing::error!("Database error getting game for date {}: {}", date, e);
                return Err(Status::internal("Database error"));
            }
        };

        // Convert to protobuf
        let game = convert_db_game_to_proto(db_game)?;
        
        Ok(Response::new(GetDailyGameResponse { game: Some(game) }))
    }

    async fn get_historical_game(
        &self,
        request: Request<GetHistoricalGameRequest>,
    ) -> Result<Response<GetHistoricalGameResponse>, Status> {
        let req = request.into_inner();
        
        let db_game = match req.selector {
            Some(get_historical_game_request::Selector::GameId(game_id)) => {
                self.repository.get_game_by_id(&game_id).await
                    .map_err(|e| {
                        tracing::error!("Database error getting game by ID {}: {}", game_id, e);
                        Status::internal("Database error")
                    })?
            }
            Some(get_historical_game_request::Selector::Date(date)) => {
                self.repository.get_game_by_date(&date).await
                    .map_err(|e| {
                        tracing::error!("Database error getting game by date {}: {}", date, e);
                        Status::internal("Database error")
                    })?
            }
            Some(get_historical_game_request::Selector::Random(_)) => {
                self.repository.get_random_historical_game().await
                    .map_err(|e| {
                        tracing::error!("Database error getting random game: {}", e);
                        Status::internal("Database error")
                    })?
            }
            None => {
                return Err(Status::invalid_argument("Must specify selector"));
            }
        };

        let game = match db_game {
            Some(game) => convert_db_game_to_proto(game)?,
            None => return Err(Status::not_found("Game not found")),
        };
        
        Ok(Response::new(GetHistoricalGameResponse { game: Some(game) }))
    }

    async fn validate_answer(
        &self,
        request: Request<ValidateAnswerRequest>,
    ) -> Result<Response<ValidateAnswerResponse>, Status> {
        let req = request.into_inner();
        
        // Get the game
        let db_game = self.repository.get_game_by_id(&req.game_id).await
            .map_err(|e| {
                tracing::error!("Database error getting game {}: {}", req.game_id, e);
                Status::internal("Database error")
            })?
            .ok_or_else(|| Status::not_found("Game not found"))?;

        // Parse board from JSON  
        let serializable_board: crate::game::conversion::SerializableBoard = serde_json::from_str(&db_game.board_data)
            .map_err(|e| {
                tracing::error!("Failed to parse board data: {}", e);
                Status::internal("Invalid board data")
            })?;
        let board: crate::game::Board = serializable_board.into();

        // Convert previous answers to internal format
        let mut previous_constraints = HashMap::new();
        for answer in req.previous_answers {
            for (wildcard_id, letter) in answer.wildcard_constraints {
                previous_constraints.insert(wildcard_id, letter.chars().next().unwrap_or('*'));
            }
        }

        // Validate the word
        match self.game_engine.validate_word_with_constraints(&board, &req.word, &previous_constraints).await {
            Ok(Some(answer)) => {
                let path: Vec<Position> = answer.best_path().tiles.iter()
                    .map(|tile| Position { row: tile.row as i32, col: tile.col as i32 })
                    .collect();

                let wildcard_constraints: HashMap<String, String> = answer.best_path().constraints.0.iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect();

                Ok(Response::new(ValidateAnswerResponse {
                    is_valid: true,
                    score: answer.score(),
                    path,
                    wildcard_constraints,
                    error_message: String::new(),
                }))
            }
            Ok(None) => {
                Ok(Response::new(ValidateAnswerResponse {
                    is_valid: false,
                    score: 0,
                    path: vec![],
                    wildcard_constraints: HashMap::new(),
                    error_message: "Invalid word or no valid path found".to_string(),
                }))
            }
            Err(e) => {
                tracing::error!("Error validating word {}: {}", req.word, e);
                Ok(Response::new(ValidateAnswerResponse {
                    is_valid: false,
                    score: 0,
                    path: vec![],
                    wildcard_constraints: HashMap::new(),
                    error_message: format!("Validation error: {}", e),
                }))
            }
        }
    }

    async fn submit_game_entry(
        &self,
        request: Request<SubmitGameEntryRequest>,
    ) -> Result<Response<SubmitGameEntryResponse>, Status> {
        let req = request.into_inner();
        
        // Verify user exists
        let _user = self.repository.get_user_by_cookie(&req.user_id).await
            .map_err(|e| {
                tracing::error!("Database error getting user {}: {}", req.user_id, e);
                Status::internal("Database error")
            })?
            .ok_or_else(|| Status::not_found("User not found"))?;

        // Verify game exists
        let _game = self.repository.get_game_by_id(&req.game_id).await
            .map_err(|e| {
                tracing::error!("Database error getting game {}: {}", req.game_id, e);
                Status::internal("Database error")
            })?
            .ok_or_else(|| Status::not_found("Game not found"))?;

        // Calculate total score
        let total_score: i32 = req.answers.iter().map(|a| a.score).sum();
        let completed = req.answers.len() == 5;

        // Serialize answers to JSON
        let answers_json = crate::serialization::serialize_answers(&req.answers)
            .map_err(|e| {
                tracing::error!("Failed to serialize answers: {}", e);
                Status::internal("Serialization error")
            })?;

        // Create or update game entry
        let new_entry = NewGameEntry {
            user_id: req.user_id,
            game_id: req.game_id.clone(),
            answers_data: answers_json,
            total_score,
            completed,
        };

        let db_entry = self.repository.create_or_update_game_entry(new_entry).await
            .map_err(|e| {
                tracing::error!("Failed to save game entry: {}", e);
                Status::internal("Failed to save game entry")
            })?;

        // Get game stats
        let (total_players, user_rank, percentile, average_score, highest_score) = 
            self.repository.get_game_stats(&req.game_id, total_score).await
                .map_err(|e| {
                    tracing::error!("Failed to get game stats: {}", e);
                    Status::internal("Failed to get game stats")
                })?;

        let stats = GameStats {
            total_players,
            user_rank,
            percentile: percentile as f32,
            average_score,
            highest_score,
        };

        // Convert to protobuf
        let game_entry = convert_db_game_entry_to_proto(db_entry, req.answers)?;

        Ok(Response::new(SubmitGameEntryResponse {
            game_entry: Some(game_entry),
            stats: Some(stats),
        }))
    }

    async fn get_game_stats(
        &self,
        request: Request<GetGameStatsRequest>,
    ) -> Result<Response<GetGameStatsResponse>, Status> {
        let req = request.into_inner();
        
        let (total_players, user_rank, percentile, average_score, highest_score) = 
            self.repository.get_game_stats(&req.game_id, req.user_score).await
                .map_err(|e| {
                    tracing::error!("Failed to get game stats: {}", e);
                    Status::internal("Failed to get game stats")
                })?;

        let stats = GameStats {
            total_players,
            user_rank,
            percentile: percentile as f32,
            average_score,
            highest_score,
        };

        Ok(Response::new(GetGameStatsResponse {
            stats: Some(stats),
        }))
    }

    async fn register_user(
        &self,
        _request: Request<RegisterUserRequest>,
    ) -> Result<Response<RegisterUserResponse>, Status> {
        let cookie_token = Uuid::new_v4().to_string();
        
        let new_user = NewUser { cookie_token };
        let db_user = self.repository.create_user(new_user).await
            .map_err(|e| {
                tracing::error!("Failed to create user: {}", e);
                Status::internal("Failed to create user")
            })?;

        let user = User {
            id: db_user.id,
            cookie_token: db_user.cookie_token,
            created_at: Some(prost_types::Timestamp {
                seconds: db_user.created_at.timestamp(),
                nanos: db_user.created_at.timestamp_subsec_nanos() as i32,
            }),
            last_seen: Some(prost_types::Timestamp {
                seconds: db_user.last_seen.timestamp(),
                nanos: db_user.last_seen.timestamp_subsec_nanos() as i32,
            }),
        };

        Ok(Response::new(RegisterUserResponse {
            user: Some(user),
        }))
    }
}

// Helper functions to convert between database models and protobuf

fn convert_db_game_to_proto(db_game: crate::db::models::DbGame) -> Result<Game, Status> {
    let serializable_board: crate::game::conversion::SerializableBoard = serde_json::from_str(&db_game.board_data)
        .map_err(|e| {
            tracing::error!("Failed to parse board data: {}", e);
            Status::internal("Invalid board data")
        })?;
    let board: crate::game::Board = serializable_board.into();

    let proto_board = convert_board_to_proto(board);

    Ok(Game {
        id: db_game.id,
        date: db_game.date,
        board: Some(proto_board),
        threshold_score: db_game.threshold_score,
        created_at: Some(prost_types::Timestamp {
            seconds: db_game.created_at.timestamp(),
            nanos: db_game.created_at.timestamp_subsec_nanos() as i32,
        }),
    })
}

fn convert_board_to_proto(board: crate::game::Board) -> Board {
    let rows: Vec<Row> = (0..4)
        .map(|row_idx| {
            let tiles: Vec<Tile> = (0..4)
                .map(|col_idx| {
                    let tile = board.get_tile(row_idx, col_idx);
                    Tile {
                        letter: tile.letter.to_string(),
                        points: tile.points,
                        is_wildcard: tile.is_wildcard,
                        row: row_idx as i32,
                        col: col_idx as i32,
                    }
                })
                .collect();
            Row { tiles }
        })
        .collect();

    Board { rows }
}

fn convert_db_game_entry_to_proto(
    db_entry: crate::db::models::DbGameEntry,
    answers: Vec<Answer>,
) -> Result<GameEntry, Status> {
    Ok(GameEntry {
        id: db_entry.id,
        user_id: db_entry.user_id,
        game_id: db_entry.game_id,
        answers,
        total_score: db_entry.total_score,
        created_at: Some(prost_types::Timestamp {
            seconds: db_entry.created_at.timestamp(),
            nanos: db_entry.created_at.timestamp_subsec_nanos() as i32,
        }),
        updated_at: Some(prost_types::Timestamp {
            seconds: db_entry.updated_at.timestamp(),
            nanos: db_entry.updated_at.timestamp_subsec_nanos() as i32,
        }),
        completed: db_entry.completed,
    })
}