#![allow(dead_code)]

//! Sources:
//! - <https://hasbro-new.custhelp.com/app/answers/detail/a_id/55/~/what-is-the-total-face-value-of-all-the-scrabble-tiles%3F>
//! - <https://hasbro-new.custhelp.com/app/answers/detail/a_id/19/related/1>
//! - <https://i.pinimg.com/originals/9a/91/ab/9a91abcf38624a17c3b158a56eaa7e84.jpg>
//! - <https://github.com/dwyl/english-words>
//! - <https://www.hasbro.com/common/instruct/Scrabble_(2003).pdf>

mod std_impls;
mod types;

pub use types::*;

use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

impl Tile {
    pub fn from_u8(n: u8) -> Option<Self> {
        // 26 letters + blank = 27
        if n < 27 {
            unsafe { Some(std::mem::transmute(n)) }
        } else {
            None
        }
    }

    fn as_char(self) -> char {
        if self == Tile::Blank {
            '*'
        } else {
            char::from(b'A' + self as u8)
        }
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

    pub fn iter_game_count() -> impl Iterator<Item = Tile> {
        let mut cur_tile = Tile::A;
        let mut cur_tile_count = cur_tile.number_in_game();
        std::iter::from_fn(move || {
            if cur_tile_count == 0 {
                cur_tile = Tile::from_u8(cur_tile as u8 + 1)?;
                cur_tile_count = cur_tile.number_in_game();
            }
            cur_tile_count -= 1;
            Some(cur_tile)
        })
    }
}

impl Move {
    fn crosses_center(&self) -> bool {
        self.tiles.iter().any(|(p, _)| *p == (7, 7))
    }

    fn positions(&self) -> Vec<Position> {
        self.tiles.iter().map(|(p, _)| *p).collect()
    }

    fn is_horizontal(&self) -> bool {
        let (x0, _) = self.tiles[0].0;
        self.tiles.iter().all(|((x, _), _)| *x == x0)
    }

    fn is_vertical(&self) -> bool {
        let (_, y0) = self.tiles[0].0;
        self.tiles.iter().all(|((_, y), _)| *y == y0)
    }
}

impl InvalidMove {
    fn new(explanation: impl Into<String>, relevant_squares: Vec<Position>) -> Self {
        Self {
            explanation: explanation.into(),
            positions: relevant_squares,
        }
    }
}

/// Includes the center as a double word modifier.
pub static MODIFIERS: Lazy<HashMap<Position, Modifier>> = Lazy::new(|| {
    type Positions = &'static [Position];
    #[rustfmt::skip] const TRIPLE_WORDS: Positions = &[(0, 0), (0, 7), (0, 14), (7, 0), (7, 14), (14, 0), (14, 7), (14, 14)];
    #[rustfmt::skip] const DOUBLE_WORDS: Positions = &[(7, 7), (1, 1), (2, 2), (3, 3), (4, 4), (10, 10), (11, 11), (12, 12), (13, 13), (1, 13), (2, 12), (3, 11), (4, 10), (13, 1), (12, 2), (11, 3), (10, 4)];
    #[rustfmt::skip] const TRIPLE_LETTERS: Positions = &[(1, 5), (1, 9), (5, 1), (5, 5), (5, 9), (5, 13), (9, 1), (9, 5), (9, 9), (9, 13), (13, 5), (13, 9)];
    #[rustfmt::skip] const DOUBLE_LETTERS: Positions = &[(0, 3), (0, 11), (3, 0), (11, 0), (14, 3), (14, 11), (3, 14), (11, 14), (2, 6), (2, 8), (3, 7), (6, 2), (8, 2), (7, 3), (6, 12), (8, 12), (7, 11), (12, 6), (12, 8), (11, 7), (6, 6), (6, 8), (8, 8), (8, 6)];

    fn map(ps: Positions, m: Modifier) -> impl Iterator<Item = ((usize, usize), Modifier)> {
        ps.iter().map(move |p| (*p, m))
    }

    let iter = map(TRIPLE_WORDS, Modifier::TripleWord)
        .chain(map(DOUBLE_WORDS, Modifier::DoubleWord))
        .chain(map(TRIPLE_LETTERS, Modifier::TripleLetter))
        .chain(map(DOUBLE_LETTERS, Modifier::DoubleLetter));

    HashMap::from_iter(iter)
});

impl Board {
    fn validate_and_score_move(&self, m: &Move) -> Result<PlayedMove, InvalidMove> {
        if m.tiles.is_empty() {
            return Err(InvalidMove::new("Empty move", vec![]));
        }

        if m.tiles.len() > 7 {
            return Err(InvalidMove::new(
                "More than 7 tiles played (impossible)",
                vec![],
            ));
        }

        if !(m.is_horizontal() || m.is_vertical()) {
            return Err(InvalidMove::new(
                "That move is not a straight line",
                m.positions(),
            ));
        }

        if self.is_empty() {
            if !m.crosses_center() {
                return Err(InvalidMove::new(
                    "The first move must play through the center",
                    m.positions(),
                ));
            }

            if m.tiles.len() == 1 {
                return Err(InvalidMove::new(
                    "The first move must be at least two letters",
                    m.positions(),
                ));
            }
        }

        // TODO
        Ok(PlayedMove {
            positions: vec![],
            words: vec![],
            value: 6,
        })
    }

    fn find_words(&self, m: &Move) -> Vec<Vec<Tile>> {
        Vec::new()
    }

    /// Precondition: all positions used in `m` are free (`None`) in `self`.
    fn apply_move(&mut self, m: &Move) {
        let mut new_board = *self;
        for ((x, y), tile) in m.tiles.iter().copied() {
            let t = &mut new_board[x][y];
            assert!(t.is_none());
            *t = Some(tile);
        }
        *self = new_board;
    }

    fn is_empty(&self) -> bool {
        *self == Board([[None; 15]; 15])
    }
}

impl Player {
    fn has_tiles_to_play_move(&self, m: &Move) -> bool {
        let mut player_tiles_count: HashMap<Tile, u8> = HashMap::new();
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
        self.moves.iter().map(|pm| pm.value).sum()
    }

    fn refill_tiles_from(&mut self, tile_bag: &mut Vec<Tile>) {
        while self.tiles.len() < 7 && !tile_bag.is_empty() {
            self.tiles.push(tile_bag.pop().unwrap());
        }
    }
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

        let played_move = self.board.validate_and_score_move(m)?;
        player.moves.push(played_move);
        player.remove_played_tiles(m);
        player.refill_tiles_from(&mut self.tile_bag);
        self.board.apply_move(m);
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
        // self.players.len() >= 2
        true
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

pub static WORDLIST: Lazy<Vec<String>> = Lazy::new(|| {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("words.txt");
    let f = File::open(path).unwrap();
    let rdr = BufReader::new(f);
    let mut words = rdr.lines().collect::<io::Result<Vec<String>>>().unwrap();

    // This should be O(n) because the word list is already sorted, but just in case
    // there is an error somewhere.
    words.sort_unstable();

    words
});

pub fn is_word(s: &String) -> bool {
    WORDLIST.binary_search(s).is_ok()
}

#[test]
fn known_game_constants() {
    assert_eq!(Tile::iter_game_count().count(), 100);
    assert_eq!(
        Tile::iter_game_count().map(Tile::point_value).sum::<u32>(),
        187
    );
    assert_eq!(Board::default().0.len(), 15);
    assert_eq!(Board::default()[0].len(), 15);
}
