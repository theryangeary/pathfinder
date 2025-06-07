use std::collections::VecDeque;
use std::fmt::Debug;

use super::constraints;

// Since we're using protobuf, we'll work with the generated Tile type
// and create a compatibility layer
#[derive(Clone, Debug, PartialEq)]
pub struct GameTile {
    pub letter: String,
    pub points: i32,
    pub is_wildcard: bool,
    pub row: i32,
    pub col: i32,
}

impl GameTile {
    pub fn id(&self) -> String {
        format!("{}_{}", self.row, self.col)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    pub tiles: VecDeque<GameTile>,
    pub constraints: constraints::Constraints,
}