use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbUser {
    pub id: String,
    pub cookie_token: String,
    pub created_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbGame {
    pub id: String,
    pub date: String,       // YYYY-MM-DD format
    pub board_data: String, // JSON serialized board
    pub threshold_score: i32,
    pub sequence_number: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbGameEntry {
    pub id: String,
    pub user_id: String,
    pub game_id: String,
    pub answers_data: String, // JSON serialized answers
    pub total_score: i32,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbGameAnswer {
    pub id: String,
    pub game_id: String,
    pub word: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbOptimalSolution {
    pub id: String,
    pub game_id: String,
    pub words_and_scores: String, // JSON blob containing optimal words and their scores
    pub total_score: i32,
    pub created_at: DateTime<Utc>,
}

// Helper structs for creating new entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub cookie_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGame {
    pub date: String,
    pub board_data: String,
    pub threshold_score: i32,
    pub sequence_number: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGameEntry {
    pub user_id: String,
    pub game_id: String,
    pub answers_data: String,
    pub total_score: i32,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGameAnswer {
    pub game_id: String,
    pub word: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOptimalSolution {
    pub game_id: String,
    pub words_and_scores: String,
    pub total_score: i32,
}

impl DbUser {
    pub fn new(cookie_token: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            cookie_token,
            created_at: now,
            last_seen: now,
        }
    }
}

impl DbGame {
    pub fn new(
        date: String,
        board_data: String,
        threshold_score: i32,
        sequence_number: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            date,
            board_data,
            threshold_score,
            sequence_number,
            created_at: Utc::now(),
        }
    }
}

impl DbGameEntry {
    pub fn new(
        user_id: String,
        game_id: String,
        answers_data: String,
        total_score: i32,
        completed: bool,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            game_id,
            answers_data,
            total_score,
            completed,
            created_at: now,
            updated_at: now,
        }
    }
}

impl DbGameAnswer {
    pub fn new(game_id: String, word: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            game_id,
            word,
            created_at: Utc::now(),
        }
    }
}

impl DbOptimalSolution {
    pub fn new(game_id: String, words_and_scores: String, total_score: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            game_id,
            words_and_scores,
            total_score,
            created_at: Utc::now(),
        }
    }
}
