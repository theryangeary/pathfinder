use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Simple serializable versions of our protobuf types for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableAnswer {
    pub word: String,
    pub score: i32,
    pub path: Vec<SerializablePosition>,
    pub wildcard_constraints: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializablePosition {
    pub row: i32,
    pub col: i32,
}

// Helper functions to convert between protobuf and serializable types
impl SerializableAnswer {
    pub fn from_proto(answer: &crate::service::wordgame::Answer) -> Self {
        Self {
            word: answer.word.clone(),
            score: answer.score,
            path: answer.path.iter().map(|pos| SerializablePosition {
                row: pos.row,
                col: pos.col,
            }).collect(),
            wildcard_constraints: answer.wildcard_constraints.clone(),
        }
    }

    pub fn to_proto(&self) -> crate::service::wordgame::Answer {
        crate::service::wordgame::Answer {
            word: self.word.clone(),
            score: self.score,
            path: self.path.iter().map(|pos| crate::service::wordgame::Position {
                row: pos.row,
                col: pos.col,
            }).collect(),
            wildcard_constraints: self.wildcard_constraints.clone(),
        }
    }
}

pub fn serialize_answers(answers: &[crate::service::wordgame::Answer]) -> Result<String, serde_json::Error> {
    let serializable: Vec<SerializableAnswer> = answers.iter()
        .map(SerializableAnswer::from_proto)
        .collect();
    serde_json::to_string(&serializable)
}

pub fn deserialize_answers(json: &str) -> Result<Vec<crate::service::wordgame::Answer>, serde_json::Error> {
    let serializable: Vec<SerializableAnswer> = serde_json::from_str(json)?;
    Ok(serializable.iter().map(|a| a.to_proto()).collect())
}