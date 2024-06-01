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

/// Includes the center as a double word modifier.
pub static MODIFIERS: Lazy<HashMap<Point, Modifier>> = Lazy::new(|| {
    type Points = &'static [Point];

    #[rustfmt::skip]
    const TRIPLE_WORDS: Points = &[
        (0, 0), (0, 7), (0, 14), (7, 0), (7, 14), (14, 0), (14, 7), (14, 14),
    ];

    #[rustfmt::skip]
    const DOUBLE_WORDS: Points = &[
        (7, 7), // Center
        (1, 1), (2, 2), (3, 3), (4, 4), (10, 10), (11, 11), (12, 12), (13, 13), (1, 13), (2, 12), (3, 11), (4, 10), (13, 1), (12, 2), (11, 3), (10, 4),
    ];

    #[rustfmt::skip]
    const TRIPLE_LETTERS: Points = &[
        (1, 5), (1, 9), (5, 1), (5, 5), (5, 9), (5, 13), (9, 1), (9, 5), (9, 9), (9, 13), (13, 5), (13, 9),
    ];

    #[rustfmt::skip]
    const DOUBLE_LETTERS: Points = &[
        (0, 3), (0, 11), (3, 0), (11, 0), (14, 3), (14, 11), (3, 14), (11, 14), (2, 6), (2, 8), (3, 7), (6, 2), (8, 2), (7, 3), (6, 12), (8, 12), (7, 11), (12, 6), (12, 8), (11, 7), (6, 6), (6, 8), (8, 8), (8, 6),
    ];

    fn map(ps: Points, m: Modifier) -> impl Iterator<Item = ((usize, usize), Modifier)> {
        ps.iter().map(move |p| (*p, m))
    }

    let iter = map(TRIPLE_WORDS, Modifier::TripleWord)
        .chain(map(DOUBLE_WORDS, Modifier::DoubleWord))
        .chain(map(TRIPLE_LETTERS, Modifier::TripleLetter))
        .chain(map(DOUBLE_LETTERS, Modifier::DoubleLetter));

    HashMap::from_iter(iter)
});

impl Board {
    pub fn validate_and_score_move(m: &Move) -> Result<u32, InvalidMoveReason> {
        _ = m;
        // TODO
        Ok(0)
    }
}

impl Game {
    pub fn new(num_players: usize) -> Self {
        assert!(num_players > 1 && num_players <= 4);

        let mut game = Game {
            players: vec![Player::default(); num_players as usize],
            board: Board::default(),
            tile_bag: Tile::iter_game_count().collect(),
            whose_turn: 0,
        };

        game.mix_tile_bag();
        for i in 0..num_players {
            game.refill_player_tiles(i);
            game.set_player_name(i, format!("Player {}", i + 1));
        }

        game
    }

    pub fn set_player_name(&mut self, player_index: usize, name: String) {
        self.players[player_index].name = name;
    }

    fn mix_tile_bag(&mut self) {
        use rand::seq::SliceRandom;
        self.tile_bag.shuffle(&mut rand::thread_rng());
    }

    fn refill_player_tiles(&mut self, player_i: usize) {
        let player = &mut self.players[player_i];
        while player.tiles.len() < 7 && !self.tile_bag.is_empty() {
            player.tiles.push(self.tile_bag.pop().unwrap());
        }
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
