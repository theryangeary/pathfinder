use core::fmt::Display;
use std::fmt::Debug;
use std::collections::HashMap;

use super::path::Path;
use super::constraints;

#[derive(Clone, Debug, PartialEq)]
pub struct Answer {
    pub word: String,
    pub paths: Vec<Path>,
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

    pub fn filter_paths_by_constraints(&self, constraints: &HashMap<String, char>) -> Answer {
        let filtered_paths: Vec<Path> = self.paths.iter()
            .filter(|path| {
                // Check if this path's constraints are compatible with existing constraints
                for (wildcard_id, existing_letter) in constraints {
                    if let Some(path_letter) = path.constraints.0.get(wildcard_id) {
                        if path_letter != existing_letter {
                            return false;
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();

        Answer {
            word: self.word.clone(),
            paths: filtered_paths,
        }
    }

    pub fn can_coexist_with(&self, other: &Answer) -> bool {
        for path in self.paths.iter() {
            for other_path in other.paths.iter() {
                if !path.constraints.has_collision_with(&other_path.constraints) {
                    return true;
                }
            }
        }
        return false;
    }

    pub fn constraints_intersections(&self, other: &Answer) -> Vec<constraints::Constraints> {
        let mut constraints = vec![];

        for path in self.paths.iter() {
            for other_path in other.paths.iter() {
                let intersection = path.constraints.intersection(&other_path.constraints);
                if let Some(i) = intersection {
                    constraints.push(i);
                }
            }
        }

        return constraints;
    }
}

impl Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", &self.word))
    }
}