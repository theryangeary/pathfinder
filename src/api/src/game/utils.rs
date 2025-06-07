use super::{Board, board::{answer::Answer, path::Path}};
use crate::game::board::proto::{Position, Answer as ProtoAnswer};
use std::collections::HashMap;

/// Utility functions for converting between internal game types and protobuf types
pub struct GameUtils;

impl GameUtils {
    /// Convert internal Answer to protobuf Answer
    pub fn answer_to_proto(answer: &Answer) -> ProtoAnswer {
        // Choose the best path (first one after filtering)
        let best_path = answer.paths.first();
        
        match best_path {
            Some(path) => {
                let positions: Vec<Position> = path.tiles.iter()
                    .map(|tile| Position {
                        row: tile.row,
                        col: tile.col,
                    })
                    .collect();

                let wildcard_constraints: HashMap<String, String> = path.constraints.0
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect();

                ProtoAnswer {
                    word: answer.word.clone(),
                    score: 0, // Will be calculated by scorer
                    path: positions,
                    wildcard_constraints,
                }
            }
            None => ProtoAnswer {
                word: answer.word.clone(),
                score: 0,
                path: vec![],
                wildcard_constraints: HashMap::new(),
            }
        }
    }

    /// Calculate score for a path using the scorer
    pub fn calculate_path_score(path: &Path, word: &str, scorer: &crate::game::Scorer) -> i32 {
        // For wildcards, we use the constraints to determine actual letters
        let mut actual_word = String::new();
        
        for tile in &path.tiles {
            if tile.is_wildcard {
                // Get the letter from constraints
                if let Some(letter) = path.constraints.0.get(&tile.id()) {
                    actual_word.push(*letter);
                } else {
                    // This shouldn't happen in a valid path
                    actual_word.push('*');
                }
            } else {
                actual_word.push_str(&tile.letter);
            }
        }
        
        // Score the word (should match the input word if constraints are correct)
        if actual_word.to_lowercase() == word.to_lowercase() {
            scorer.score(&actual_word.to_lowercase()) as i32
        } else {
            0 // Invalid path
        }
    }

    /// Validate that wildcard constraints are consistent across multiple answers
    pub fn validate_wildcard_consistency(answers: &[&Answer]) -> Result<(), String> {
        let mut global_constraints: HashMap<String, char> = HashMap::new();
        
        for answer in answers {
            for path in &answer.paths {
                for (wildcard_id, required_letter) in &path.constraints.0 {
                    if let Some(existing_letter) = global_constraints.get(wildcard_id) {
                        if *existing_letter != *required_letter {
                            return Err(format!(
                                "Wildcard constraint conflict: {} needs to be both '{}' and '{}'",
                                wildcard_id, existing_letter, required_letter
                            ));
                        }
                    } else {
                        global_constraints.insert(wildcard_id.clone(), *required_letter);
                    }
                }
            }
        }
        
        Ok(())
    }
}