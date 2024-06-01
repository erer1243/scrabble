use serde::{Deserialize, Serialize};

#[rustfmt::skip]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Tile {
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, Blank,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Modifier {
    DoubleLetter,
    TripleLetter,
    DoubleWord,
    TripleWord,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Direction {
    Right,
    Down,
}

pub type Point = (usize, usize);

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Board(pub [[Option<Tile>; 15]; 15]);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InvalidMoveReason {
    Disconnected,
    NotAWord,
    Impossible(String),
}

/// An attempt to play a word which may or may not be valid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Move {
    pub letters: Vec<Tile>,
    pub start: Point,
    pub direction: Direction,
}

/// An Nx1 or 1xN rectangle on the board
// #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
// pub struct BoardRegion {
//     pub length: usize,
//     pub start: Point,
//     pub direction: Direction,
// }

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub tiles: Vec<Tile>,
    pub score: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    pub board: Board,
    pub tile_bag: Vec<Tile>,
    pub players: Vec<Player>,
    pub whose_turn: u8,
}
