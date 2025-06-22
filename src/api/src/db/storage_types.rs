use serde::{Deserialize, Serialize};

/// Stable database types for long-term storage compatibility.
/// These types should remain backwards compatible and only evolve carefully.
///
/// Version 1.0 - Initial stable schema

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DbStoredAnswers {
    pub version: String,
    pub answers: Vec<DbAnswer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DbAnswer {
    pub word: String,
    pub score: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DbPosition {
    pub row: i32,
    pub col: i32,
}

impl DbStoredAnswers {
    pub fn new(answers: Vec<DbAnswer>) -> Self {
        Self {
            version: "1.0".to_string(),
            answers,
        }
    }

    /// Serialize to JSON string for database storage
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON string with version compatibility
    pub fn from_json(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Try to parse as current version first
        match serde_json::from_str::<DbStoredAnswers>(json) {
            Ok(stored) => {
                // Validate version compatibility
                match stored.version.as_str() {
                    "1.0" => Ok(stored),
                    version => Err(format!("Unsupported version: {}", version).into()),
                }
            }
            Err(_) => {
                // Fallback: try to parse as legacy format (direct Vec<DbAnswer>)
                // This maintains compatibility with data stored before versioning
                match serde_json::from_str::<Vec<DbAnswer>>(json) {
                    Ok(answers) => {
                        println!("Migrating legacy answer data to versioned format");
                        Ok(DbStoredAnswers::new(answers))
                    }
                    Err(e) => Err(format!("Failed to parse answers data: {}", e).into()),
                }
            }
        }
    }
}

impl Default for DbStoredAnswers {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let answers = vec![DbAnswer {
            word: "test".to_string(),
            score: 10,
        }];

        let stored = DbStoredAnswers::new(answers.clone());
        let json = stored.to_json().unwrap();
        let deserialized = DbStoredAnswers::from_json(&json).unwrap();

        assert_eq!(stored, deserialized);
        assert_eq!(deserialized.version, "1.0");
        assert_eq!(deserialized.answers, answers);
    }

    #[test]
    fn test_legacy_format_compatibility() {
        // Simulate legacy format (direct Vec<DbAnswer>)
        let legacy_json =
            r#"[{"word":"test","score":10,"path":[{"row":0,"col":0}],"wildcard_constraints":{}}]"#;

        let result = DbStoredAnswers::from_json(legacy_json).unwrap();
        assert_eq!(result.version, "1.0");
        assert_eq!(result.answers.len(), 1);
        assert_eq!(result.answers[0].word, "test");
    }

    #[test]
    fn test_version_validation() {
        let future_version_json = r#"{"version":"2.0","answers":[]}"#;
        let result = DbStoredAnswers::from_json(future_version_json);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported version"));
    }
}
