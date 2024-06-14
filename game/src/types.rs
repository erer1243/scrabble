use serde::{Deserialize, Serialize};

#[rustfmt::skip]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Tile {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, Blank,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub enum Modifier {
    DoubleLetter,
    TripleLetter,
    DoubleWord,
    TripleWord,
}

pub type Position = (usize, usize);

#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq)]
pub struct Board(pub [[Option<Tile>; 15]; 15]);

/// An attempt to play a word which may or may not be valid
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Move {
    pub tiles: Vec<(Position, Tile)>,
}

#[derive(Clone, Debug, Serialize)]
pub struct InvalidMove {
    pub explanation: String,
    pub positions: Vec<Position>,
}

/// A move that a player previously made, that produced some number of new words on the board
/// and was worth some number of points.
#[derive(Clone, Debug, Serialize)]
pub struct PlayedMove {
    pub positions: Vec<Position>,
    pub words: Vec<String>,
    pub value: u32,
}

#[derive(Default, Clone, Debug, Serialize)]
pub struct Player {
    pub name: String,
    pub tiles: Vec<Tile>,
    pub moves: Vec<PlayedMove>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Game {
    pub board: Board,
    pub tile_bag: Vec<Tile>,
    pub players: Vec<Player>,
    pub whose_turn: usize,
}
