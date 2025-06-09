use crate::game::directions;
use core::fmt;
use std::collections::{HashMap, HashSet, VecDeque};

pub mod answer;
pub mod constraints;
pub mod path;

use path::GameTile;

// Native Rust types (replacing protobuf)
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub tiles: Vec<Tile>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tile {
    pub letter: String,
    pub points: i32,
    pub is_wildcard: bool,
    pub row: i32,
    pub col: i32,
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.rows {
            for tile in &row.tiles {
                if !tile.is_wildcard {
                    write!(f, " {} ", &tile.letter.to_uppercase())?;
                } else {
                    write!(f, " * ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl Tile {
    fn id(&self) -> String {
        format!("{}_{}", self.row, self.col)
    }
}

// Convert protobuf Tile to GameTile for internal use
impl From<&Tile> for GameTile {
    fn from(tile: &Tile) -> Self {
        GameTile {
            letter: tile.letter.clone(),
            points: tile.points,
            is_wildcard: tile.is_wildcard,
            row: tile.row,
            col: tile.col,
        }
    }
}

impl Board {
    pub fn new() -> Self {
        Self {
            rows: (0..4).map(|row_idx| {
                Row {
                    tiles: (0..4).map(|col_idx| {
                        Tile {
                            letter: "*".to_string(),
                            points: 0,
                            is_wildcard: false,
                            row: row_idx,
                            col: col_idx,
                        }
                    }).collect()
                }
            }).collect()
        }
    }

    pub fn set_tile(&mut self, row: usize, col: usize, letter: char, points: i32, is_wildcard: bool) {
        if row < 4 && col < 4 {
            self.rows[row].tiles[col] = Tile {
                letter: letter.to_string(),
                points,
                is_wildcard,
                row: row as i32,
                col: col as i32,
            };
        }
    }

    pub fn get_tile(&self, row: usize, col: usize) -> &Tile {
        &self.rows[row].tiles[col]
    }

    pub fn new_answer(&self, word: &str) -> answer::Answer {
        self.paths_for(word)
    }

    pub fn paths_for(&self, word: &str) -> answer::Answer {
        let mut paths = vec![];
        for row in 0..self.rows.len() {
            for column in 0..self.rows[row].tiles.len() {
                let mut new_paths = self.paths_for_word_from_position(word, row, column, &mut HashSet::new());
                paths.append(&mut new_paths);
            }
        }
        
        // Filter out paths that use a superset of wildcards compared to other paths
        let final_paths = Self::filter_minimal_wildcard_paths(paths);
        
        return answer::Answer {
            paths: final_paths,
            word: word.into(),
        };
    }

    pub fn contains(&self, answer: &answer::Answer) -> bool {
        answer.paths.len() > 0
    }

    pub fn paths_for_word_from_position(
        &self,
        word: &str,
        row_number: usize,
        column_number: usize,
        visited: &mut HashSet<(usize, usize)>,
    ) -> Vec<path::Path> {
        let mut result = vec![];
        let current_word_char = word.chars().next();
        if visited.contains(&(row_number, column_number)) || current_word_char.is_none() {
            return result;
        }
        let current_char = current_word_char.unwrap();
        let current_location = &self.rows[row_number].tiles[column_number];
        let current_location_letter = current_location.letter.chars().next();
        
        if current_location_letter != Some(current_char)
            && !current_location.is_wildcard
        {
            return result;
        }
        
        if word.len() == 1 {
            let mut tiles = VecDeque::new();
            tiles.push_back(GameTile::from(current_location));
            let mut constraints = HashMap::new();
            if current_location.is_wildcard {
                let _ = constraints.insert(current_location.id(), current_char);
            }
            let path = path::Path {
                tiles,
                constraints: constraints::Constraints(constraints),
            };
            result.push(path);
            return result;
        }
        
        for direction in directions::DIRECTIONS {
            visited.insert((row_number, column_number));
            let next_row_number = row_number.checked_add_signed(direction.0);
            let next_column_number = column_number.checked_add_signed(direction.1);
            if next_row_number.is_none()
                || next_column_number.is_none()
                || next_row_number.unwrap() >= self.rows.len()
                || next_column_number.unwrap() >= self.rows[next_row_number.unwrap()].tiles.len()
            {
                continue;
            }

            let paths = self.paths_for_word_from_position(
                &word[1..],
                next_row_number.unwrap(),
                next_column_number.unwrap(),
                visited,
            );

            for mut path in paths.into_iter() {
                path.tiles.push_front(GameTile::from(current_location));
                if current_location.is_wildcard {
                    let _ = path
                        .constraints
                        .0
                        .insert(current_location.id(), current_char);
                }
                result.push(path.clone());
            }
            visited.remove(&(row_number, column_number));
        }
        return result;
    }

    fn filter_minimal_wildcard_paths(paths: Vec<path::Path>) -> Vec<path::Path> {
        if paths.is_empty() {
            return paths;
        }

        // Group paths by number of wildcards used
        let mut paths_by_wildcard_count: HashMap<usize, Vec<path::Path>> = HashMap::new();
        for path in paths {
            let wildcard_count = path.constraints.0.len();
            paths_by_wildcard_count.entry(wildcard_count).or_default().push(path);
        }

        // Find the minimum number of wildcards used
        let min_wildcard_count = *paths_by_wildcard_count.keys().min().unwrap();
        
        // Return only paths that use the minimum number of wildcards
        paths_by_wildcard_count.remove(&min_wildcard_count).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn display(answer: &answer::Answer) -> String {
        let mut result = String::new();
        for path in &answer.paths {
            let mut letters = String::new();
            let mut locations = String::new();
            for tile in &path.tiles {
                letters.push(tile.letter.chars().next().unwrap_or('*'));
                locations += &format!(",({},{})", tile.row, tile.col);
            }
            result += &letters;
            result += &format!("[{}],", locations);
            result.push('\n');
        }
        return result;
    }

    fn test_board() -> Board {
        return Board {
            rows: vec![
                Row {
                    tiles: vec![
                        Tile {
                            letter: "a".into(),
                            points: 1,
                            is_wildcard: false,
                            row: 0,
                            col: 0,
                        },
                        Tile {
                            letter: "b".into(),
                            points: 2,
                            is_wildcard: false,
                            row: 0,
                            col: 1,
                        },
                        Tile {
                            letter: "c".into(),
                            points: 3,
                            is_wildcard: false,
                            row: 0,
                            col: 2,
                        },
                        Tile {
                            letter: "d".into(),
                            points: 4,
                            is_wildcard: false,
                            row: 0,
                            col: 3,
                        },
                    ],
                },
                Row {
                    tiles: vec![
                        Tile {
                            letter: "a".into(),
                            points: 1,
                            is_wildcard: false,
                            row: 1,
                            col: 0,
                        },
                        Tile {
                            letter: "b".into(),
                            points: 2,
                            is_wildcard: false,
                            row: 1,
                            col: 1,
                        },
                        Tile {
                            letter: "c".into(),
                            points: 3,
                            is_wildcard: false,
                            row: 1,
                            col: 2,
                        },
                        Tile {
                            letter: "d".into(),
                            points: 4,
                            is_wildcard: false,
                            row: 1,
                            col: 3,
                        },
                    ],
                },
                Row {
                    tiles: vec![
                        Tile {
                            letter: "a".into(),
                            points: 1,
                            is_wildcard: false,
                            row: 2,
                            col: 0,
                        },
                        Tile {
                            letter: "b".into(),
                            points: 2,
                            is_wildcard: false,
                            row: 2,
                            col: 1,
                        },
                        Tile {
                            letter: "".into(),
                            points: 0,
                            is_wildcard: true,
                            row: 2,
                            col: 2,
                        },
                        Tile {
                            letter: "d".into(),
                            points: 4,
                            is_wildcard: false,
                            row: 2,
                            col: 3,
                        },
                    ],
                },
                Row {
                    tiles: vec![
                        Tile {
                            letter: "a".into(),
                            points: 1,
                            is_wildcard: false,
                            row: 3,
                            col: 0,
                        },
                        Tile {
                            letter: "p".into(),
                            points: 2,
                            is_wildcard: false,
                            row: 3,
                            col: 1,
                        },
                        Tile {
                            letter: "p".into(),
                            points: 3,
                            is_wildcard: false,
                            row: 3,
                            col: 2,
                        },
                        Tile {
                            letter: "d".into(),
                            points: 4,
                            is_wildcard: false,
                            row: 3,
                            col: 3,
                        },
                    ],
                },
            ],
        };
    }

    #[test]
    fn test_basic_path_finding() {
        let board = test_board();
        let answer = board.paths_for("app");
        assert!(!answer.paths.is_empty(), "Should find paths for 'app'");
    }

    #[test]
    fn test_ab_paths() {
        let board = test_board();
        let answer = board.paths_for("ab");
        // Should find multiple paths from different 'a' tiles to adjacent 'b' tiles
        assert!(answer.paths.len() > 1, "Should find multiple paths for 'ab'");
    }
}