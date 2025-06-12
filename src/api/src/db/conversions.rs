use crate::db::storage_types::{DbStoredAnswers, DbAnswer, DbPosition};
use crate::http_api::{ApiAnswer, ApiPosition};

/// Conversion functions between HTTP API types and stable database types.
/// These provide a compatibility layer that allows API types to evolve
/// while maintaining backwards compatibility with stored data.

impl From<ApiAnswer> for DbAnswer {
    fn from(api: ApiAnswer) -> Self {
        Self {
            word: api.word,
            score: api.score,
        }
    }
}

impl From<DbAnswer> for ApiAnswer {
    fn from(db: DbAnswer) -> Self {
        Self {
            word: db.word,
            score: db.score,
        }
    }
}

impl From<ApiPosition> for DbPosition {
    fn from(api: ApiPosition) -> Self {
        Self {
            row: api.row,
            col: api.col,
        }
    }
}

impl From<DbPosition> for ApiPosition {
    fn from(db: DbPosition) -> Self {
        Self {
            row: db.row,
            col: db.col,
        }
    }
}

impl From<Vec<ApiAnswer>> for DbStoredAnswers {
    fn from(api_answers: Vec<ApiAnswer>) -> Self {
        let db_answers = api_answers.into_iter().map(DbAnswer::from).collect();
        DbStoredAnswers::new(db_answers)
    }
}

impl From<DbStoredAnswers> for Vec<ApiAnswer> {
    fn from(stored: DbStoredAnswers) -> Self {
        stored.answers.into_iter().map(ApiAnswer::from).collect()
    }
}

/// Helper functions for working with answer data in the database
pub struct AnswerStorage;

impl AnswerStorage {
    /// Serialize API answers to JSON string for database storage
    pub fn serialize_api_answers(answers: &[ApiAnswer]) -> Result<String, serde_json::Error> {
        let stored = DbStoredAnswers::from(answers.to_vec());
        stored.to_json()
    }

    /// Deserialize JSON string from database to API answers
    pub fn deserialize_to_api_answers(json: &str) -> Result<Vec<ApiAnswer>, Box<dyn std::error::Error>> {
        let stored = DbStoredAnswers::from_json(json)?;
        Ok(Vec::<ApiAnswer>::from(stored))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_api_to_db_conversion() {
        let api_answer = ApiAnswer {
            word: "test".to_string(),
            score: 15,
        };

        let db_answer = DbAnswer::from(api_answer.clone());
        let back_to_api = ApiAnswer::from(db_answer);

        assert_eq!(api_answer.word, back_to_api.word);
        assert_eq!(api_answer.score, back_to_api.score);
    }

    #[test]
    fn test_answer_storage_roundtrip() {
        let api_answers = vec![
            ApiAnswer {
                word: "hello".to_string(),
                score: 10,
            },
            ApiAnswer {
                word: "world".to_string(),
                score: 20,
            },
        ];

        let json = AnswerStorage::serialize_api_answers(&api_answers).unwrap();
        let deserialized = AnswerStorage::deserialize_to_api_answers(&json).unwrap();

        assert_eq!(api_answers.len(), deserialized.len());
        assert_eq!(api_answers[0].word, deserialized[0].word);
    }

    #[test]
    fn test_versioned_storage() {
        let api_answers = vec![
            ApiAnswer {
                word: "test".to_string(),
                score: 5,
            }
        ];

        let json = AnswerStorage::serialize_api_answers(&api_answers).unwrap();
        
        // Verify the JSON contains version information
        assert!(json.contains(r#""version":"1.0""#));
        
        let deserialized = AnswerStorage::deserialize_to_api_answers(&json).unwrap();
        assert_eq!(api_answers[0].word, deserialized[0].word);
    }
}
