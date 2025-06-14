use crate::game::board::{Board, Row, Tile, constraints::{PathConstraintSet, AnswerGroupConstraintSet}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableBoard {
    pub rows: Vec<SerializableRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableRow {
    pub tiles: Vec<SerializableTile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableTile {
    pub letter: String,
    pub points: i32,
    pub is_wildcard: bool,
    pub row: i32,
    pub col: i32,
}

impl From<&Board> for SerializableBoard {
    fn from(board: &Board) -> Self {
        Self {
            rows: board.rows.iter().map(|row| SerializableRow {
                tiles: row.tiles.iter().map(|tile| SerializableTile {
                    letter: tile.letter.clone(),
                    points: tile.points,
                    is_wildcard: tile.is_wildcard,
                    row: tile.row,
                    col: tile.col,
                }).collect()
            }).collect()
        }
    }
}

impl From<SerializableBoard> for Board {
    fn from(board: SerializableBoard) -> Self {
        Self {
            rows: board.rows.into_iter().map(|row| Row {
                tiles: row.tiles.into_iter().map(|tile| Tile {
                    letter: tile.letter,
                    points: tile.points,
                    is_wildcard: tile.is_wildcard,
                    row: tile.row,
                    col: tile.col,
                }).collect()
            }).collect()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializablePath {
    pub tiles: Vec<SerializableGameTile>,
    pub constraints: SerializablePathConstraintSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableGameTile {
    pub letter: String,
    pub points: i32,
    pub is_wildcard: bool,
    pub row: i32,
    pub col: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializablePathConstraintSet {
    Unconstrainted,
    FirstDecided(char),
    SecondDecided(char),
    BothDecided(char, char),
}

impl From<&crate::game::board::path::Path> for SerializablePath {
    fn from(path: &crate::game::board::path::Path) -> Self {
        Self {
            tiles: path.tiles.iter().map(|tile| SerializableGameTile {
                letter: tile.letter.clone(),
                points: tile.points,
                is_wildcard: tile.is_wildcard,
                row: tile.row,
                col: tile.col,
            }).collect(),
            constraints: SerializablePathConstraintSet::from(&path.constraints),
        }
    }
}

impl From<&PathConstraintSet> for SerializablePathConstraintSet {
    fn from(constraints: &PathConstraintSet) -> Self {
        match constraints {
            PathConstraintSet::Unconstrainted => SerializablePathConstraintSet::Unconstrainted,
            PathConstraintSet::FirstDecided(c) => SerializablePathConstraintSet::FirstDecided(*c),
            PathConstraintSet::SecondDecided(c) => SerializablePathConstraintSet::SecondDecided(*c),
            PathConstraintSet::BothDecided(c1, c2) => SerializablePathConstraintSet::BothDecided(*c1, *c2),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableAnswerGroupConstraintSet {
    pub path_constraint_sets: Vec<SerializablePathConstraintSet>,
}

impl From<&AnswerGroupConstraintSet> for SerializableAnswerGroupConstraintSet {
    fn from(constraints: &AnswerGroupConstraintSet) -> Self {
        Self {
            path_constraint_sets: constraints.path_constraint_sets.iter()
                .map(|pcs| SerializablePathConstraintSet::from(pcs))
                .collect(),
        }
    }
}