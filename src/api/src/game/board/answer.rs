use core::fmt::Display;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::game::board::constraints::{AnswerGroupConstraintSet, ConstraintsSet};

use super::constraints;
use super::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub struct Answer {
    pub word: String,
    pub paths: Vec<Path>,
    pub constraints_set: AnswerGroupConstraintSet,
}

impl Answer {
    pub fn best_path(&self) -> &Path {
        // Return the first path if available, or the one with least wildcards
        self.paths.first().unwrap()
    }

    pub fn score(&self) -> i32 {
        if let Some(path) = self.paths.first() {
            path.tiles.iter().map(|tile| tile.points).sum()
        } else {
            0
        }
    }
}

impl Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", &self.word))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::board::{
        constraints::{Constraint, PathConstraintSet},
        path::{GameTile, Path},
    };

    use std::collections::VecDeque;

    fn create_test_tile(row: i32, col: i32, letter: char, points: i32) -> GameTile {
        GameTile {
            row,
            col,
            letter: letter.to_string(),
            points,
            is_wildcard: false,
        }
    }

    fn create_wildcard_tile(row: i32, col: i32, points: i32) -> GameTile {
        GameTile {
            row,
            col,
            letter: "*".to_string(),
            points,
            is_wildcard: true,
        }
    }

    fn create_test_path(tiles: Vec<GameTile>, constraints: HashMap<String, Constraint>) -> Path {
        let mut tile_deque = VecDeque::new();
        for tile in tiles {
            tile_deque.push_back(tile);
        }
        Path {
            tiles: tile_deque,
            constraints: PathConstraintSet::Unconstrainted,
        }
    }

    #[test]
    fn test_answer_best_path() {
        let tile1 = create_test_tile(0, 0, 'c', 2);
        let tile2 = create_test_tile(0, 1, 'a', 1);
        let tile3 = create_test_tile(0, 2, 't', 1);

        let path1 = create_test_path(
            vec![tile1.clone(), tile2.clone(), tile3.clone()],
            HashMap::new(),
        );
        let path2 = create_test_path(vec![tile1, tile2, tile3], HashMap::new());

        let answer = Answer {
            word: "cat".to_string(),
            paths: vec![path1.clone(), path2],
            constraints_set: AnswerGroupConstraintSet {
                path_constraint_sets: vec![],
            },
        };

        // Should return the first path
        assert_eq!(answer.best_path(), &path1);
    }

    #[test]
    #[should_panic]
    fn test_answer_best_path_empty() {
        let answer = Answer {
            word: "empty".to_string(),
            paths: vec![],
            constraints_set: AnswerGroupConstraintSet {
                path_constraint_sets: vec![],
            },
        };

        // Should panic when no paths are available
        answer.best_path();
    }

    #[test]
    fn test_answer_score_with_paths() {
        let tile1 = create_test_tile(0, 0, 'c', 2);
        let tile2 = create_test_tile(0, 1, 'a', 1);
        let tile3 = create_test_tile(0, 2, 't', 1);

        let path = create_test_path(vec![tile1, tile2, tile3], HashMap::new());

        let answer = Answer {
            word: "cat".to_string(),
            paths: vec![path],
            constraints_set: AnswerGroupConstraintSet {
                path_constraint_sets: vec![],
            },
        };

        // Score should be sum of tile points: 2 + 1 + 1 = 4
        assert_eq!(answer.score(), 4);
    }

    #[test]
    fn test_answer_score_empty_paths() {
        let answer = Answer {
            word: "empty".to_string(),
            paths: vec![],
            constraints_set: AnswerGroupConstraintSet {
                path_constraint_sets: vec![],
            },
        };

        // Score should be 0 when no paths exist
        assert_eq!(answer.score(), 0);
    }

    #[test]
    fn test_answer_score_with_wildcards() {
        let tile1 = create_test_tile(0, 0, 'c', 2);
        let tile2 = create_wildcard_tile(1, 1, 0); // Wildcard with 0 points
        let tile3 = create_test_tile(0, 2, 't', 1);

        let path = create_test_path(vec![tile1, tile2, tile3], HashMap::new());

        let answer = Answer {
            word: "cat".to_string(),
            paths: vec![path],
            constraints_set: AnswerGroupConstraintSet {
                path_constraint_sets: vec![],
            },
        };

        // Score should be 2 + 0 + 1 = 3 (wildcard contributes 0)
        assert_eq!(answer.score(), 3);
    }

    #[test]
    fn test_display_trait() {
        let answer = Answer {
            word: "test".to_string(),
            paths: vec![],
            constraints_set: AnswerGroupConstraintSet {
                path_constraint_sets: vec![],
            },
        };

        assert_eq!(format!("{}", answer), "test");
    }

    #[test]
    fn test_debug_trait() {
        let answer = Answer {
            word: "test".to_string(),
            paths: vec![],
            constraints_set: AnswerGroupConstraintSet {
                path_constraint_sets: vec![],
            },
        };

        let debug_string = format!("{:?}", answer);
        assert!(debug_string.contains("test"));
        assert!(debug_string.contains("Answer"));
    }

    #[test]
    fn test_clone_and_partialeq() {
        let tile = create_test_tile(0, 0, 'c', 2);
        let path = create_test_path(vec![tile], HashMap::new());

        let answer1 = Answer {
            word: "cat".to_string(),
            paths: vec![path.clone()],
            constraints_set: AnswerGroupConstraintSet {
                path_constraint_sets: vec![],
            },
        };

        let answer2 = answer1.clone();
        assert_eq!(answer1, answer2);

        let answer3 = Answer {
            word: "dog".to_string(),
            paths: vec![path],
            constraints_set: AnswerGroupConstraintSet {
                path_constraint_sets: vec![],
            },
        };

        assert_ne!(answer1, answer3);
    }
}
