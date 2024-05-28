#![allow(unused)]

//! Sources:
//! https://hasbro-new.custhelp.com/app/answers/detail/a_id/55/~/what-is-the-total-face-value-of-all-the-scrabble-tiles%3F
//! https://hasbro-new.custhelp.com/app/answers/detail/a_id/19/related/1
//! https://i.pinimg.com/originals/9a/91/ab/9a91abcf38624a17c3b158a56eaa7e84.jpg
//! https://github.com/dwyl/english-words

use std::{
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    ops::Index,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Tile {
    A = 0,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Blank = 26,
}

impl Tile {
    fn from_u8(n: u8) -> Option<Self> {
        if n <= 26 {
            unsafe { Some(std::mem::transmute(n)) }
        } else {
            None
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

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == Tile::Blank {
            write!(f, "*")
        } else {
            let ch = char::from(b'A' + *self as u8);
            write!(f, "{ch}")
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Modifier {
    DoubleLetter,
    TripleLetter,
    DoubleWord,
    TripleWord,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Board([[Option<Tile>; 15]; 15]);

impl std::ops::Deref for Board {
    type Target = [[Option<Tile>; 15]; 15];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Board {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#################")?;
        for i in 0..15 {
            write!(f, "#")?;
            for j in 0..15 {
                match self[i][j] {
                    Some(t) => write!(f, "{t}")?,
                    None => write!(f, " ")?,
                }
            }
            writeln!(f, "#")?;
        }
        write!(f, "#################")?;
        Ok(())
    }
}

#[derive(Default, Clone, Debug)]
struct Player {
    tiles: Vec<Tile>,
    score: u32,
}

#[derive(Clone, Debug)]
struct Game {
    board: Board,
    tile_bag: Vec<Tile>,
    players: Vec<Player>,
    whose_turn: u8,
}

impl Game {
    fn new(num_players: u8) -> Self {
        assert!(num_players > 1 && num_players <= 4);

        let board = Board::default();
        let tile_bag = Vec::from_iter(Tile::iter_game_count());
        let players = vec![Player::default(); num_players as usize];
        let whose_turn = 0;

        Self {
            players,
            board,
            tile_bag,
            whose_turn,
        }
    }
}

#[derive(Debug)]
pub struct WordList {
    words: Vec<String>,
}

impl WordList {
    pub fn load() -> io::Result<Self> {
        let f = File::open("words.txt")?;
        let rdr = BufReader::new(f);
        let mut words = rdr.lines().collect::<io::Result<Vec<String>>>()?;

        // This should be O(n) because the word list is already sorted, but just in case
        // there is an error somewhere.
        words.sort_unstable();

        Ok(WordList { words })
    }

    pub fn check(&self, word: &str) -> bool {
        self.words.binary_search(&word.to_string()).is_ok()
    }
}
