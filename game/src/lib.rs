#![allow(dead_code)]

//! Sources:
//! - <https://hasbro-new.custhelp.com/app/answers/detail/a_id/55/~/what-is-the-total-face-value-of-all-the-scrabble-tiles%3F>
//! - <https://hasbro-new.custhelp.com/app/answers/detail/a_id/19/related/1>
//! - <https://i.pinimg.com/originals/9a/91/ab/9a91abcf38624a17c3b158a56eaa7e84.jpg>
//! - <https://github.com/dwyl/english-words>
//! - <https://www.hasbro.com/common/instruct/Scrabble_(2003).pdf>

mod solve;
mod std_impls;

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, iter};

#[rustfmt::skip]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Tile {
     Blank = b'*', A = b'a', B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
}

impl Tile {
    fn from_ascii(n: u8) -> Option<Self> {
        if !n.is_ascii_lowercase() {
            None
        } else {
            Some(unsafe { std::mem::transmute(n) })
        }
    }

    fn as_ascii(self) -> u8 {
        self as u8
    }

    fn as_char(self) -> char {
        char::from(self.as_ascii())
    }

    fn successor(self) -> Option<Self> {
        Self::from_ascii(self.as_ascii() + 1)
    }

    fn point_value(self) -> u32 {
        use Tile::*;
        match self {
            A | E | I | O | U | L | N | S | T | R => 1,
            D | G => 2,
            B | C | M | P => 3,
            F | H | V | W | Y => 4,
            K => 5,
            J | X => 8,
            Q | Z => 10,
            Blank => 0,
        }
    }

    fn number_in_game(self) -> u32 {
        use Tile::*;
        match self {
            J | K | Q | X | Z => 1,
            B | C | F | H | M | P | V | W | Y | Blank => 2,
            G => 3,
            D | L | S | U => 4,
            N | R | T => 6,
            O => 8,
            A | I => 9,
            E => 12,
        }
    }

    fn iter_alphabet() -> impl Iterator<Item = Tile> {
        (b'a'..=b'z').map(|c| Tile::from_ascii(c).unwrap())
    }

    fn iter_game_count() -> impl Iterator<Item = Tile> {
        let mut cur_tile = Tile::A;
        let mut cur_tile_count = cur_tile.number_in_game();
        std::iter::from_fn(move || {
            if cur_tile_count == 0 {
                cur_tile = if cur_tile == Tile::Z {
                    Tile::Blank
                } else {
                    cur_tile.successor()?
                };
                cur_tile_count = cur_tile.number_in_game();
            }
            cur_tile_count -= 1;
            Some(cur_tile)
        })
    }
}

/// An attempt to play a word which may or may not be valid
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct Move {
    tiles: Vec<(Position, Tile)>,
}

impl Move {
    fn crosses_center(&self) -> bool {
        self.tiles.iter().any(|(p, _)| *p == (7, 7))
    }

    fn positions(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.tiles.iter().map(|(p, _)| *p)
    }

    fn is_horizontal(&self) -> bool {
        let (x0, _) = self.tiles[0].0;
        self.tiles.iter().all(|((x, _), _)| *x == x0)
    }

    fn is_vertical(&self) -> bool {
        let (_, y0) = self.tiles[0].0;
        self.tiles.iter().all(|((_, y), _)| *y == y0)
    }

    fn is_straight_line(&self) -> bool {
        self.is_vertical() || self.is_horizontal()
    }

    /// Precondition: self is a straight line (function won't panic, but the return value will be incorrect & meaningless)
    fn is_contiguous(&self) -> bool {
        fn dist((ax, ay): Position, (bx, by): Position) -> usize {
            (((ax as isize) - (bx as isize)).abs() + ((ay as isize) - (by as isize)).abs()) as usize
        }

        self.sorted()
            .tiles
            .windows(2)
            .all(|ts| dist(ts[0].0, ts[1].0) == 1)
    }

    fn sort(&mut self) {
        self.tiles.sort_unstable_by_key(|t| t.0);
    }

    fn sorted(&self) -> Move {
        let mut m = self.clone();
        m.sort();
        m
    }

    fn to_word(&self) -> String {
        self.sorted()
            .tiles
            .into_iter()
            .map(|(_, t)| t.as_char())
            .collect()
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct InvalidMove {
    pub explanation: String,
    pub positions: Vec<Position>,
}

impl InvalidMove {
    fn new(explanation: impl Into<String>, positions: impl IntoIterator<Item = Position>) -> Self {
        Self {
            explanation: explanation.into(),
            positions: positions.into_iter().collect(),
        }
    }
}

/// A move that a player previously made, that produced some number of new words on the board
/// and was worth some number of points.
#[derive(Clone, Debug, Serialize)]
pub struct PlayedMove {
    original_move: Move,
    word_values: Vec<(String, u32)>,
}

impl PlayedMove {
    fn value(&self) -> u32 {
        self.word_values.iter().map(|(_, value)| value).sum()
    }
}

pub type Position = (usize, usize);

#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq)]
pub struct Board {
    tiles: [[Option<Tile>; 15]; 15],
    blank_fills: [[Option<Tile>; 15]; 15],
}

impl Board {
    fn new() -> Self {
        Self::default()
    }

    fn get_non_blank(&self, (x, y): Position) -> Option<Tile> {
        self.tiles[x][y].map(|t| {
            if t == Tile::Blank {
                self.blank_fills[x][y].unwrap()
            } else {
                t
            }
        })
    }

    /// Precondition: blank_fills.len() == number of blank tiles in m
    fn with_move_applied(&self, m: &Move, blank_fills: &[Tile]) -> Board {
        let mut new_board = *self;
        let mut blank_i = 0;
        for ((x, y), tile) in m.tiles.iter().copied() {
            new_board.tiles[x][y] = Some(tile);

            if tile == Tile::Blank {
                new_board.blank_fills[x][y] = Some(blank_fills[blank_i]);
                blank_i += 1;
            }
        }
        new_board
    }

    fn is_empty(&self) -> bool {
        *self == Board::new()
    }

    fn play_move(&mut self, m: &Move) -> Result<PlayedMove, InvalidMove> {
        solve::validate_move(self, m)?;
        let (expanded_move, crossing_moves) = solve::expand_move(self, m);

        if crossing_moves.is_empty() && !self.is_empty() {
            return Err(InvalidMove::new("That move is disconnected", m.positions()));
        }

        let blank_fills = solve::solve_for_blanks(m, &crossing_moves)?;

        let moves = iter::once(&expanded_move).chain(crossing_moves.iter());

        // We need to score the move before applying it to the board
        // because scoring takes into account which tiles are new and
        // which are from some prior move that we're playing off of
        let scores: Vec<_> = moves.clone().map(|m| solve::score_move(self, m)).collect();

        let new_board = self.with_move_applied(m, &blank_fills);

        // Create strings out of the moves after we apply the move to the board so
        // we can take advantage of Board::get_non_blank
        let move_words: Vec<_> = moves
            .map(|m| {
                (
                    m,
                    m.sorted()
                        .tiles
                        .iter()
                        .map(|(pos, _)| new_board.get_non_blank(*pos).unwrap().as_char())
                        .collect::<String>(),
                )
            })
            .collect();

        for (m, w) in &move_words {
            if !solve::is_word(w) {
                return Err(InvalidMove::new(
                    format!("'{w}' is not a word"),
                    m.positions(),
                ));
            }
        }

        let word_values = move_words.into_iter().map(|(_, w)| w).zip(scores).collect();

        *self = new_board;

        Ok(PlayedMove {
            original_move: m.clone(),
            word_values,
        })
    }
}

#[derive(Default, Clone, Debug, Serialize)]
pub struct Player {
    pub name: String,
    pub tiles: Vec<Tile>,
    pub moves: Vec<PlayedMove>,
}

impl Player {
    fn has_tiles_to_play_move(&self, m: &Move) -> bool {
        let mut player_tiles_count: HashMap<Tile, u8> = HashMap::with_capacity(self.tiles.len());
        for &t in &self.tiles {
            *player_tiles_count.entry(t).or_insert(0) += 1;
        }
        for (_, t) in &m.tiles {
            match player_tiles_count.get_mut(t) {
                Some(0) => return false,
                Some(n) => *n -= 1,
                None => return false,
            }
        }
        true
    }

    fn remove_played_tiles(&mut self, m: &Move) {
        for (_, t) in &m.tiles {
            let i = self.tiles.iter().position(|t_| t == t_).unwrap();
            self.tiles.swap_remove(i);
        }
    }

    fn score(&self) -> u32 {
        self.moves.iter().map(|pm| pm.value()).sum()
    }

    fn refill_tiles_from(&mut self, tile_bag: &mut Vec<Tile>) {
        while self.tiles.len() < 7 && !tile_bag.is_empty() {
            self.tiles.push(tile_bag.pop().unwrap());
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Game {
    pub board: Board,
    pub tile_bag: Vec<Tile>,
    pub players: Vec<Player>,
    pub whose_turn: usize,
}

impl Game {
    pub fn new() -> Self {
        Game {
            players: vec![],
            board: Board::default(),
            tile_bag: Tile::iter_game_count().collect(),
            whose_turn: 0,
        }
    }

    pub fn start_game(&mut self) {
        let rng = &mut rand::thread_rng();
        self.tile_bag.shuffle(rng);
        self.players.shuffle(rng);
        for p in &mut self.players {
            p.refill_tiles_from(&mut self.tile_bag);
        }
    }

    pub fn play_move(&mut self, m: &Move) -> Result<(), InvalidMove> {
        let player = &mut self.players[self.whose_turn];

        if !player.has_tiles_to_play_move(m) {
            return Err(InvalidMove::new(
                "Tiles played that you don't have (impossible)",
                vec![],
            ));
        }

        let played_move = self.board.play_move(m)?;
        player.moves.push(played_move);
        player.remove_played_tiles(m);
        player.refill_tiles_from(&mut self.tile_bag);
        self.whose_turn += 1;
        self.whose_turn %= self.players.len();
        Ok(())
    }

    fn index_of_player(&self, name: &str) -> Option<usize> {
        self.players
            .iter()
            .enumerate()
            .find(|(_, p)| p.name == name)
            .map(|(i, _)| i)
    }

    pub fn is_players_turn(&self, name: &str) -> bool {
        self.index_of_player(name).unwrap() == self.whose_turn
    }

    pub fn ready_to_play(&self) -> bool {
        self.players.len() >= 2
    }

    pub fn has_player(&self, name: &str) -> bool {
        self.index_of_player(name).is_some()
    }

    pub fn add_player(&mut self, name: String) {
        self.players.push(Player {
            name,
            ..Default::default()
        })
    }

    fn game_finished_by_tiles(&self) -> bool {
        self.tile_bag.is_empty() && self.players.iter().any(|p| p.tiles.is_empty())
    }
}
