use core::fmt::Display;
use std::fmt::Debug;
use std::collections::HashMap;

use crate::game::board::constraints::{AnswerGroupConstraintSet, ConstraintsSet};

use super::path::Path;
use super::constraints;

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

    // pub fn filter_paths_by_constraints(&self, constraints: &HashMap<String, char>) -> Answer {
    //     let filtered_paths: Vec<Path> = self.paths.iter()
    //         .filter(|path| {
    //             // Check if this path's constraints are compatible with existing constraints
    //             for (wildcard_id, existing_letter) in constraints {
    //                 if let Some(path_letter) = path.constraints.0.get(wildcard_id) {
    //                     if path_letter != existing_letter {
    //                         return false;
    //                     }
    //                 }
    //             }
    //             true
    //         })
    //         .cloned()
    //         .collect();

    //     Answer {
    //         word: self.word.clone(),
    //         paths: filtered_paths,
    //     }
    // }

    // pub fn can_coexist_with(&self, other: &Answer) -> bool {
    //     for path in self.paths.iter() {
    //         for other_path in other.paths.iter() {
    //             if !path.constraints.has_collision_with(&other_path.constraints) {
    //                 return true;
    //             }
    //         }
    //     }
    //     return false;
    // }

    // pub fn constraints_intersections(&self, other: &Answer) -> Vec<constraints::ConstraintsSet> {
    //     let mut constraints = vec![];

    //     for path in self.paths.iter() {
    //         for other_path in other.paths.iter() {
    //             let intersection = path.constraints.intersection(&other_path.constraints);
    //             if let Some(i) = intersection {
    //                 constraints.push(i);
    //             }
    //         }
    //     }

    //     return constraints;
    // }
}

impl Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", &self.word))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::board::{constraints::{Constraint, PathConstraintSet}, path::{GameTile, Path}};
    use super::constraints::ConstraintsSet;
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

        let path1 = create_test_path(vec![tile1.clone(), tile2.clone(), tile3.clone()], HashMap::new());
        let path2 = create_test_path(vec![tile1, tile2, tile3], HashMap::new());

        let answer = Answer {
            word: "cat".to_string(),
            paths: vec![path1.clone(), path2],
            constraints_set: AnswerGroupConstraintSet{path_constraint_sets:vec![]},
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
            constraints_set: AnswerGroupConstraintSet{path_constraint_sets:vec![]},
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
            constraints_set: AnswerGroupConstraintSet{path_constraint_sets:vec![]},
        };

        // Score should be sum of tile points: 2 + 1 + 1 = 4
        assert_eq!(answer.score(), 4);
    }

    #[test]
    fn test_answer_score_empty_paths() {
        let answer = Answer {
            word: "empty".to_string(),
            paths: vec![],
            constraints_set: AnswerGroupConstraintSet{path_constraint_sets:vec![]},
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
            constraints_set: AnswerGroupConstraintSet{path_constraint_sets:vec![]},
        };

        // Score should be 2 + 0 + 1 = 3 (wildcard contributes 0)
        assert_eq!(answer.score(), 3);
    }

    // #[test]
    // fn test_filter_paths_by_constraints_compatible() {
    //     let mut constraints = HashMap::new();
    //     constraints.insert("1_1".to_string(), 'a');

    //     let mut path1_constraints = HashMap::new();
    //     path1_constraints.insert("1_1".to_string(), 'a'); // Compatible

    //     let mut path2_constraints = HashMap::new();
    //     path2_constraints.insert("1_1".to_string(), 'e'); // Incompatible

    //     let tile = create_test_tile(0, 0, 'c', 2);
    //     let path1 = create_test_path(vec![tile.clone()], path1_constraints);
    //     let path2 = create_test_path(vec![tile], path2_constraints);

    //     let answer = Answer {
    //         word: "cat".to_string(),
    //         paths: vec![path1.clone(), path2],
    //     };

    //     let filtered = answer.filter_paths_by_constraints(&constraints);

    //     // Should only keep path1 (compatible constraint)
    //     assert_eq!(filtered.paths.len(), 1);
    //     assert_eq!(filtered.paths[0], path1);
    //     assert_eq!(filtered.word, "cat");
    // }

    // #[test]
    // fn test_filter_paths_by_constraints_no_conflicts() {
    //     let mut constraints = HashMap::new();
    //     constraints.insert("1_1".to_string(), 'a');

    //     // Path with no constraints on the same wildcard
    //     let path_constraints = HashMap::new();
    //     let tile = create_test_tile(0, 0, 'c', 2);
    //     let path = create_test_path(vec![tile], path_constraints);

    //     let answer = Answer {
    //         word: "cat".to_string(),
    //         paths: vec![path.clone()],
    //     };

    //     let filtered = answer.filter_paths_by_constraints(&constraints);

    //     // Should keep the path since there's no conflict
    //     assert_eq!(filtered.paths.len(), 1);
    //     assert_eq!(filtered.paths[0], path);
    // }

    // #[test]
    // fn test_filter_paths_by_constraints_empty_constraints() {
    //     let constraints = HashMap::new();

    //     let tile = create_test_tile(0, 0, 'c', 2);
    //     let path = create_test_path(vec![tile], HashMap::new());

    //     let answer = Answer {
    //         word: "cat".to_string(),
    //         paths: vec![path.clone()],
    //     };

    //     let filtered = answer.filter_paths_by_constraints(&constraints);

    //     // Should keep all paths when no constraints exist
    //     assert_eq!(filtered.paths.len(), 1);
    //     assert_eq!(filtered.paths[0], path);
    // }

    // #[test]
    // fn test_filter_paths_by_constraints_all_filtered() {
    //     let mut constraints = HashMap::new();
    //     constraints.insert("1_1".to_string(), 'a');

    //     let mut path_constraints = HashMap::new();
    //     path_constraints.insert("1_1".to_string(), 'e'); // Incompatible

    //     let tile = create_test_tile(0, 0, 'c', 2);
    //     let path = create_test_path(vec![tile], path_constraints);

    //     let answer = Answer {
    //         word: "cat".to_string(),
    //         paths: vec![path],
    //     };

    //     let filtered = answer.filter_paths_by_constraints(&constraints);

    //     // Should filter out all paths
    //     assert_eq!(filtered.paths.len(), 0);
    // }

    // #[test]
    // fn test_can_coexist_with_compatible() {
    //     // Create paths with non-conflicting constraints
    //     let mut constraints1 = HashMap::new();
    //     constraints1.insert("1_1".to_string(), 'a');

    //     let mut constraints2 = HashMap::new();
    //     constraints2.insert("2_2".to_string(), 'e'); // Different wildcard

    //     let tile = create_test_tile(0, 0, 'c', 2);
    //     let path1 = create_test_path(vec![tile.clone()], constraints1);
    //     let path2 = create_test_path(vec![tile], constraints2);

    //     let answer1 = Answer {
    //         word: "cat".to_string(),
    //         paths: vec![path1],
    //     };

    //     let answer2 = Answer {
    //         word: "ace".to_string(),
    //         paths: vec![path2],
    //     };

    //     // Should be able to coexist since they don't conflict
    //     assert!(answer1.can_coexist_with(&answer2));
    // }

    // #[test]
    // fn test_can_coexist_with_incompatible() {
    //     // Create paths with conflicting constraints
    //     let mut constraints1 = HashMap::new();
    //     constraints1.insert("1_1".to_string(), 'a');

    //     let mut constraints2 = HashMap::new();
    //     constraints2.insert("1_1".to_string(), 'e'); // Same wildcard, different letter

    //     let tile = create_test_tile(0, 0, 'c', 2);
    //     let path1 = create_test_path(vec![tile.clone()], constraints1);
    //     let path2 = create_test_path(vec![tile], constraints2);

    //     let answer1 = Answer {
    //         word: "cat".to_string(),
    //         paths: vec![path1],
    //     };

    //     let answer2 = Answer {
    //         word: "cet".to_string(),
    //         paths: vec![path2],
    //     };

    //     // Should not be able to coexist due to conflicting constraints
    //     assert!(!answer1.can_coexist_with(&answer2));
    // }

    // #[test]
    // fn test_can_coexist_with_multiple_paths() {
    //     // First answer has two paths: one compatible, one incompatible
    //     let mut constraints1a = HashMap::new();
    //     constraints1a.insert("1_1".to_string(), 'a');

    //     let mut constraints1b = HashMap::new();
    //     constraints1b.insert("1_1".to_string(), 'e');

    //     let mut constraints2 = HashMap::new();
    //     constraints2.insert("1_1".to_string(), 'a'); // Compatible with path1a

    //     let tile = create_test_tile(0, 0, 'c', 2);
    //     let path1a = create_test_path(vec![tile.clone()], constraints1a);
    //     let path1b = create_test_path(vec![tile.clone()], constraints1b);
    //     let path2 = create_test_path(vec![tile], constraints2);

    //     let answer1 = Answer {
    //         word: "cat".to_string(),
    //         paths: vec![path1a, path1b],
    //     };

    //     let answer2 = Answer {
    //         word: "cat".to_string(),
    //         paths: vec![path2],
    //     };

    //     // Should be able to coexist since at least one path combination works
    //     assert!(answer1.can_coexist_with(&answer2));
    // }

    // #[test]
    // fn test_constraints_intersections() {
    //     let mut constraints1 = HashMap::new();
    //     constraints1.insert("1_1".to_string(), 'a');

    //     let mut constraints2 = HashMap::new();
    //     constraints2.insert("1_1".to_string(), 'a'); // Same constraint
    //     constraints2.insert("2_2".to_string(), 'e');

    //     let tile = create_test_tile(0, 0, 'c', 2);
    //     let path1 = create_test_path(vec![tile.clone()], constraints1);
    //     let path2 = create_test_path(vec![tile], constraints2);

    //     let answer1 = Answer {
    //         word: "cat".to_string(),
    //         paths: vec![path1],
    //     };

    //     let answer2 = Answer {
    //         word: "ace".to_string(),
    //         paths: vec![path2],
    //     };

    //     let intersections = answer1.constraints_intersections(&answer2);

    //     // Should find one intersection
    //     assert_eq!(intersections.len(), 1);
    //     assert!(intersections[0].0.contains_key("1_1"));
    //     assert_eq!(intersections[0].0["1_1"], 'a');
    // }

    // #[test]
    // fn test_constraints_intersections_no_overlap() {
    //     let mut constraints1 = HashMap::new();
    //     constraints1.insert("1_1".to_string(), 'a');

    //     let mut constraints2 = HashMap::new();
    //     constraints2.insert("2_2".to_string(), 'e');

    //     let tile = create_test_tile(0, 0, 'c', 2);
    //     let path1 = create_test_path(vec![tile.clone()], constraints1);
    //     let path2 = create_test_path(vec![tile], constraints2);

    //     let answer1 = Answer {
    //         word: "cat".to_string(),
    //         paths: vec![path1],
    //     };

    //     let answer2 = Answer {
    //         word: "ace".to_string(),
    //         paths: vec![path2],
    //     };

    //     let intersections = answer1.constraints_intersections(&answer2);

    //     // The intersection method merges non-conflicting constraints, so we get one result
    //     assert_eq!(intersections.len(), 1);
    //     assert!(intersections[0].0.contains_key("1_1"));
    //     assert!(intersections[0].0.contains_key("2_2"));
    //     assert_eq!(intersections[0].0["1_1"], 'a');
    //     assert_eq!(intersections[0].0["2_2"], 'e');
    // }

    #[test]
    fn test_display_trait() {
        let answer = Answer {
            word: "test".to_string(),
            paths: vec![],
            constraints_set: AnswerGroupConstraintSet{path_constraint_sets:vec![]},
        };

        assert_eq!(format!("{}", answer), "test");
    }

    #[test]
    fn test_debug_trait() {
        let answer = Answer {
            word: "test".to_string(),
            paths: vec![],
            constraints_set: AnswerGroupConstraintSet{path_constraint_sets:vec![]},
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
            constraints_set: AnswerGroupConstraintSet{path_constraint_sets:vec![]},
        };

        let answer2 = answer1.clone();
        assert_eq!(answer1, answer2);

        let answer3 = Answer {
            word: "dog".to_string(),
            paths: vec![path],
            constraints_set: AnswerGroupConstraintSet{path_constraint_sets:vec![]},
        };

        assert_ne!(answer1, answer3);
    }
}
