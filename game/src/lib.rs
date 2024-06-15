#![allow(dead_code)]

//! Sources:
//! - <https://hasbro-new.custhelp.com/app/answers/detail/a_id/55/~/what-is-the-total-face-value-of-all-the-scrabble-tiles%3F>
//! - <https://hasbro-new.custhelp.com/app/answers/detail/a_id/19/related/1>
//! - <https://i.pinimg.com/originals/9a/91/ab/9a91abcf38624a17c3b158a56eaa7e84.jpg>
//! - <https://github.com/dwyl/english-words>
//! - <https://www.hasbro.com/common/instruct/Scrabble_(2003).pdf>

mod std_impls;

#[cfg(test)]
mod tests;

use core::slice;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead, BufReader},
    ops::Range,
    path::Path,
};

use serde::{Deserialize, Serialize};

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
    let rdr = BufReader::new(File::open(path).unwrap());
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

// fn tiles_to_string(tiles: &[Tile]) -> String {
//     tiles.iter().copied().map(Tile::as_char).collect()
// }

// fn tiles_to_bytes(tiles: &[Tile]) -> Vec<u8> {
//     tiles_to_string(tiles).into_bytes()
// }

// fn bytes_to_tiles(bytes: &[u8]) -> Vec<Tile> {
//     bytes.iter().copied().map(Tile::from_ascii).collect()
// }

fn tiles_to_bytes(tiles: &[Tile]) -> &[u8] {
    unsafe { std::mem::transmute(tiles) }
}

fn is_word(word: &[Tile]) -> bool {
    WORDLIST
        .binary_search_by(|w| w.as_bytes().cmp(tiles_to_bytes(word)))
        .is_ok()
}

fn solve_for_blanks(
    segments: &[&[Tile]],
    crossing_words: &[Option<(&[Tile], &[Tile])>],
) -> Option<Vec<Tile>> {
    assert!(segments.len() > 1);
    assert!(crossing_words.len() == segments.len() - 1);

    let n_blanks = segments.len() - 1;
    let mut fills: Vec<Tile> = Vec::with_capacity(n_blanks);
    let mut buf: Vec<Tile> = Vec::with_capacity(16);

    macro_rules! partial_word {
        () => { partial_word!(. None) };
        ($extra:expr) => { partial_word!(. Some(& $extra)) };
        (. $extra:expr) => {{
            use itertools::Itertools;
            let iter = segments
                .iter()
                .copied()
                .interleave_shortest(fills.iter().chain($extra).map(std::slice::from_ref))
                .flatten()
                .copied();
            buf.clear();
            buf.extend(iter);
            buf.as_slice()
        }};
    }

    macro_rules! crossing_word {
        ($i:expr, $fill:expr) => {{
            let (a, b) = crossing_words[$i].unwrap();
            buf.clear();
            buf.extend_from_slice(a);
            buf.push($fill);
            buf.extend_from_slice(b);
            buf.as_slice()
        }};
    }

    macro_rules! implies {
        ($a:expr, $b:expr) => {
            !($a && !$b)
        };
    }

    let mut first = true;
    while fills.len() < n_blanks || !is_word(partial_word!()) {
        let mut tile = match fills.pop() {
            Some(prev_tile) => match prev_tile.successor() {
                Some(next_tile) => next_tile,
                None => continue,
            },
            None if first => {
                first = false;
                Tile::A
            }
            None => return None,
        };

        loop {
            let m_range = word_prefix_range(partial_word!(tile));
            let cwi = fills.len();
            if m_range.is_some()
                && implies!(
                    crossing_words[cwi].is_some(),
                    is_word(crossing_word!(cwi, tile))
                )
            {
                fills.push(tile);
                break;
            }

            if let Some(next_tile) = tile.successor() {
                tile = next_tile;
            } else {
                break;
            }
        }
    }

    // XXX the actual return value should be the fills only, this is just for debugging
    _ = partial_word!();
    Some(buf)

    // let mut prefix_stk: Vec<Range<usize>> = Vec::with_capacity(n_blanks);
    // while prefix_stk.len() < n_blanks {

    // }

    // let mut buf = [Tile::Blank; 128];

    // fn collect_into_slice<T, I: IntoIterator<Item = T>>(slice: &mut [T], iter: I) -> &[T] {
    //     let mut n = 0;
    //     for (a, b) in slice.iter_mut().zip(iter.into_iter()) {
    //         n += 1;
    //         *a = b;
    //     }
    //     &slice[..n]
    // }

    // let mut curr_word = &buf[..0];
    // macro_rules! curr_word {
    //     ($extra:expr) => {{
    //         let mut l = 0;
    //         for (i, fb) in filled_blanks.iter().enumerate() {
    //             // Copy in the segment before the filled blank
    //             let seg = non_blank_segments[i];
    //             (&mut buf[l..l + seg.len()]).copy_from_slice(seg);
    //             l += seg.len();

    //             // Copy in the filled blank
    //             buf[l] = fb.tile;
    //         }

    //         if let Some(b) = $extra {
    //             buf[l] = b;
    //             l += 1;
    //         }

    //         &buf[..l]
    //         // curr_word = &buf[..l];
    //         // curr_word
    //     }};

    //     () => {
    //         curr_word!(None)
    //     };
    // }

    // while filled_blanks.len() < n_blanks {
    //     let mut next_tile = Tile::A;
    //     let range
    // }

    // None
}

fn word_prefix_range(prefix: &[Tile]) -> Option<Range<usize>> {
    let prefix_bytes = unsafe { slice::from_raw_parts(prefix.as_ptr().cast(), prefix.len()) };
    binary_search_for_prefix_range(&WORDLIST, prefix_bytes)
}

fn binary_search_for_prefix_range<B>(arr: &[B], prefix: &[u8]) -> Option<Range<usize>>
where
    B: AsRef<[u8]>,
{
    let start = binary_search_for_prefix_range_start(arr, prefix)?;
    let end = start + 1 + binary_search_for_prefix_range_end(&arr[start..], prefix);
    Some(Range { start, end })
}

fn binary_search_for_prefix_range_start<B>(arr: &[B], prefix: &[u8]) -> Option<usize>
where
    B: AsRef<[u8]>,
{
    use std::cmp::Ordering::*;

    let mut l = 0;
    let mut r = arr.len() - 1;

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

/// Precondition: arr contains at least one element that is prefixed by `prefix`
/// i.e. `binary_search_for_prefix_range_start(arr, prefix)` returned `Some(..)`.
fn binary_search_for_prefix_range_end<B>(arr: &[B], prefix: &[u8]) -> usize
where
    B: AsRef<[u8]>,
{
    use std::cmp::Ordering::*;

    let mut l = 0;
    let mut r = arr.len() - 1;
    let last = r;

    while l <= r {
        let m = (l + r) / 2;
        let m_word = arr[m].as_ref();
        let found_end = || m == last || !arr[m + 1].as_ref().starts_with(prefix);

        match m_word.cmp(prefix) {
            Less => l = m + 1,
            Greater => {
                if m_word.starts_with(prefix) {
                    if found_end() {
                        return m;
                    } else {
                        l = m + 1;
                    }
                } else {
                    r = m - 1;
                }
            }
            Equal => {
                if found_end() {
                    return m;
                } else {
                    l = m + 1;
                }
            }
        }
    }

    unreachable!("there is no start to the prefix range, so there can be no end");
}
