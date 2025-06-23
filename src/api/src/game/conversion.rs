use crate::game::board::{Board, Row, Tile};
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
            rows: board
                .rows
                .iter()
                .map(|row| SerializableRow {
                    tiles: row
                        .tiles
                        .iter()
                        .map(|tile| SerializableTile {
                            letter: tile.letter.clone(),
                            points: tile.points,
                            is_wildcard: tile.is_wildcard,
                            row: tile.row,
                            col: tile.col,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

impl From<SerializableBoard> for Board {
    fn from(board: SerializableBoard) -> Self {
        Self {
            rows: board
                .rows
                .into_iter()
                .map(|row| Row {
                    tiles: row
                        .tiles
                        .into_iter()
                        .map(|tile| Tile {
                            letter: tile.letter,
                            points: tile.points,
                            is_wildcard: tile.is_wildcard,
                            row: tile.row,
                            col: tile.col,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}
