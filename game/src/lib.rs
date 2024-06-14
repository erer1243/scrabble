#![allow(dead_code)]

//! Sources:
//! - <https://hasbro-new.custhelp.com/app/answers/detail/a_id/55/~/what-is-the-total-face-value-of-all-the-scrabble-tiles%3F>
//! - <https://hasbro-new.custhelp.com/app/answers/detail/a_id/19/related/1>
//! - <https://i.pinimg.com/originals/9a/91/ab/9a91abcf38624a17c3b158a56eaa7e84.jpg>
//! - <https://github.com/dwyl/english-words>
//! - <https://www.hasbro.com/common/instruct/Scrabble_(2003).pdf>

mod std_impls;

use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    ops::Index,
    path::Path,
};

use serde::{Deserialize, Serialize};

#[rustfmt::skip]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Tile {
    A = 0, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, Blank,
}

impl Tile {
    fn from_u8(n: u8) -> Option<Self> {
        // 26 letters + blank = 27
        if n < 27 {
            unsafe { Some(std::mem::transmute(n)) }
        } else {
            None
        }
    }

    fn as_ascii(self) -> u8 {
        if self == Tile::Blank {
            b'*'
        } else {
            b'a' + self as u8
        }
    }

    fn as_char(self) -> char {
        char::from(self.as_ascii())
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

    fn iter_game_count() -> impl Iterator<Item = Tile> {
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

fn tiles_to_string(tiles: &[Tile]) -> String {
    tiles.iter().copied().map(Tile::as_char).collect()
}

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

#[derive(Default, Clone, Debug, Serialize)]
pub struct Player {
    pub name: String,
    pub tiles: Vec<Tile>,
    pub moves: Vec<PlayedMove>,
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

static WORDLIST: Lazy<Vec<String>> = Lazy::new(|| {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("words.txt");
    let f = File::open(path).unwrap();
    let rdr = BufReader::new(f);
    let mut words = rdr.lines().collect::<io::Result<Vec<String>>>().unwrap();

    // This should be O(n) because the word list is already sorted, but sort just in case
    // there is an error somewhere.
    words.sort_unstable();

    // I want to be able to assume chars are single lowercase bytes in these strings
    assert!(words
        .iter()
        .all(|w| w.chars().all(|c| c.is_ascii_lowercase())));

    words
});

type Bytes<'a> = &'a [u8];

fn is_word(word: Bytes) -> bool {
    WORDLIST
        .binary_search_by(|w| w.as_bytes().cmp(word))
        .is_ok()
}

fn solve_multiword(
    segments: &[Bytes],
    crossers: &[Option<(Bytes, Bytes)>],
) -> Option<&'static str> {
    struct State {}

    let mut buf = Vec::<u8>::with_capacity(16);
    let mut stk = Vec::<State>::with_capacity(segments.len() - 1);

    // stk.push(b'a');

    None
}

fn binary_search_for_prefix_range<T, B>(
    arr: &T,
    prefix: Bytes,
    mut l: usize,
    mut r: usize,
) -> Option<(usize, usize)>
where
    T: Index<usize, Output = B>,
    B: AsRef<[u8]>,
{
    l = binary_search_for_prefix_range_start(arr, prefix, l, r)?;
    r = binary_search_for_prefix_range_end(arr, prefix, l, r)?;
    Some((l, r))
}

fn binary_search_for_prefix_range_start<T, B>(
    arr: &T,
    prefix: Bytes,
    mut l: usize,
    mut r: usize,
) -> Option<usize>
where
    T: Index<usize, Output = B>,
    B: AsRef<[u8]>,
{
    use std::cmp::Ordering::*;

    while l <= r {
        let m = (l + r) / 2;
        let m_word = arr[m].as_ref();

        match m_word.cmp(prefix) {
            Less => l = m + 1,
            Greater => {
                // This check allows the least word that still has the given prefix to satisfy.
                // Normal binary search is only searching for an exact match to the prefix,
                // but given ["aa", "bb", "cc"] and "b", we still want to find "bb".
                if m_word.starts_with(prefix) && (m == 0 || arr[m - 1].as_ref() < prefix) {
                    return Some(m);
                }

                // If prefix is less than the entire list, then eventually m = 0 but m_word > prefix,
                // so this would subtract 1 from 0usize and underflow.
                r = m.checked_sub(1)?;
            }
            Equal => return Some(m),
        }
    }

    None
}

fn binary_search_for_prefix_range_end<T, B>(
    arr: &T,
    prefix: Bytes,
    mut l: usize,
    mut r: usize,
) -> Option<usize>
where
    T: Index<usize, Output = B>,
    B: AsRef<[u8]>,
{
    use std::cmp::Ordering::*;

    let right_edge = r;

    while l <= r {
        let m = (l + r) / 2;
        let m_word = arr[m].as_ref();

        match m_word.cmp(prefix) {
            Less => l = m + 1,
            Greater => {
                if m_word.starts_with(prefix) {
                    if m == right_edge || !arr[m + 1].as_ref().starts_with(prefix) {
                        return Some(m);
                    } else {
                        l = m + 1;
                    }
                } else {
                    r = m - 1;
                }
            }
            Equal => {
                if m == right_edge || !arr[m + 1].as_ref().starts_with(prefix) {
                    return Some(m);
                } else {
                    l = m + 1;
                }
            }
        }
    }

    None
}

#[test]
fn binary_search_for_prefix_range_test() {
    let arr = [
        "aa", "ab", "abb", "ac", "ad", "ba", "bb", "bc", "ca", "cb", "cc",
    ];
    let test = |prefix| binary_search_for_prefix_range(&arr, prefix, 0, arr.len() - 1);
    assert_eq!(test(b"a"), Some((0, 4)));
    assert_eq!(test(b"ab"), Some((1, 2)));
    assert_eq!(test(b"abb"), Some((2, 2)));
    assert_eq!(test(b"aaa"), None);
    assert_eq!(test(b"x"), None);
    assert_eq!(test(b"A"), None);

    let test = |prefix| {
        if let Some((l, r)) =
            binary_search_for_prefix_range(&*WORDLIST, prefix, 0, WORDLIST.len() - 1)
        {
            assert!(!WORDLIST[l - 1].as_bytes().starts_with(prefix));
            assert!(!WORDLIST[r + 1].as_bytes().starts_with(prefix));
            println!(
                "{} => {:?}\n",
                std::str::from_utf8(prefix).unwrap(),
                &(&*WORDLIST)[l..=r]
            );
        }
    };
    test(b"apple");
    test(b"fuck");
    test(b"zoo");
    test(b"onomatopoeia");
    test(b"this is not a word");
    test(b"XXXXXXXXXXXXXX");
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

#[test]
fn wordlist_tests() {
    once_cell::sync::Lazy::force(&WORDLIST);
}
