//! Sources:
//! - <https://hasbro-new.custhelp.com/app/answers/detail/a_id/55/~/what-is-the-total-face-value-of-all-the-scrabble-tiles%3F>
//! - <https://hasbro-new.custhelp.com/app/answers/detail/a_id/19/related/1>
//! - <https://i.pinimg.com/originals/9a/91/ab/9a91abcf38624a17c3b158a56eaa7e84.jpg>
//! - <https://github.com/dwyl/english-words>
//! - <https://www.hasbro.com/common/instruct/Scrabble_(2003).pdf>

pub mod solve;

use forr::forr;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    fmt::Display,
    iter,
    ops::{Index, IndexMut},
};

macro_rules! tile_enum {
    ( $(#[$attr:meta])* $name:ident { $(A = $a:expr,)? $(Blank $($blank:tt)+)? }) => {
        $(#[$attr])*
        pub enum $name {
            A $(= $a)?, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
            $(Blank $($blank)+)?
        }
    };
}

tile_enum! {
    /// Enum for letters in the alphabet. Does not contain a representation of a Blank tile, it is only alphabetical.
    /// It has the same repr as the corresponding lowercase ascii char.
    #[repr(u8)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
    Letter { A = b'a', }
}

impl Letter {
    fn as_ascii(self) -> u8 {
        self as u8
    }

    fn as_char(self) -> char {
        char::from(self.as_ascii())
    }
}

tile_enum! {
    /// Enum for Scrabble tiles that a player is holding. Contains all letters plus a Blank variant.
    /// Like `Letter`, it is repr'd by corresponding ascii chars, while Blank is repr'd as `b'*'`.
    #[repr(u8)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    Tile {
        A = b'a',
        Blank = b'*'
    }
}

impl Tile {
    fn as_ascii(self) -> u8 {
        self as u8
    }

    fn as_char(self) -> char {
        char::from(self.as_ascii())
    }

    #[cfg(test)]
    fn from_ascii(c: u8) -> Self {
        if !(c.is_ascii_lowercase() || c == b'*') {
            panic!("Invalid ascii byte for tile: {c}");
        }

        unsafe { std::mem::transmute(c) }
    }

    #[cfg(test)]
    fn from_char(c: char) -> Self {
        if !c.is_ascii_lowercase() {
            panic!("Invalid char for tile: {c}");
        }

        Self::from_ascii(c as u8)
    }

    #[cfg(test)]
    fn as_board_tile(self) -> BoardTile {
        forr! { $tile:tt in [A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z] $:
            match self {
                Self::Blank => panic!("Tile::Blank.as_board.tile()"),
                $(Self::$tile => BoardTile::$tile,)*
            }
        }
    }

    /// Point value according to official Scrabble rules
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

    /// Number of each tile that comes in a regular Scrabble tile bag
    fn number_in_game(self) -> usize {
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
        forr! { $tile:tt in [A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, Blank] $:
            iter::empty()
                $(.chain(iter::repeat(Self::$tile).take(Self::$tile.number_in_game())))*
        }
    }
}

tile_enum! {
    /// Enum for tiles on the board. Blank tiles hold the `Letter` that they represent on the board.
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
    BoardTile {
        Blank(Letter)
    }
}

impl BoardTile {
    fn as_tile(self) -> Tile {
        forr! { $letter:tt in [A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z] $:
            match self {
                BoardTile::Blank { .. } => Tile::Blank,
                $(Self::$letter => Tile::$letter,)*
            }
        }
    }

    fn as_letter(self) -> Letter {
        forr! { $letter:tt in [A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z] $:
            match self {
                BoardTile::Blank(fill) => fill,
                $(Self::$letter => Letter::$letter,)*
            }
        }
    }
}

#[cfg(test)]
impl From<Tile> for BoardTile {
    fn from(t: Tile) -> Self {
        forr! { $letter:tt in [A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z] $:
            match t {
                Tile::Blank => panic!("BoardTile::from(Tile::Blank)"),
                $(Tile::$letter => BoardTile::$letter,)*
            }
        }
    }
}

/// A set of tiles at some position on a Scrabble board.
/// `Move`s are not inherently valid - tiles in the move could overlap ones already on the board,
/// overlap each other, be at invalid coordinates, contain letters the player doesn't have, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Move {
    tiles: Vec<(Position, BoardTile)>,

    /// internal field for optimizing move validation.
    #[serde(skip)]
    sorted: bool,
}

impl Move {
    pub fn new(tiles: Vec<(Position, BoardTile)>) -> Self {
        let mut m = Self {
            tiles,
            sorted: false,
        };
        m.sort();
        m
    }

    fn crosses_center(&self) -> bool {
        self.tiles.iter().any(|(p, _)| *p == (7, 7))
    }

    fn positions(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.tiles.iter().map(|(p, _)| *p)
    }

    fn contains_position(&self, pos: Position) -> bool {
        self.positions().any(|p| p == pos)
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

    fn sorted(&self) -> Cow<Move> {
        if self.sorted {
            Cow::Borrowed(self)
        } else {
            let mut m = self.clone();
            m.sort();
            Cow::Owned(m)
        }
    }

    fn sort(&mut self) {
        self.tiles.sort_unstable_by_key(|t| t.0);
        self.sorted = true;
    }

    fn to_word(&self) -> String {
        self.sorted()
            .tiles
            .iter()
            .map(|(_, t)| t.as_letter().as_char())
            .collect()
    }
}

/// A player-facing message that explains why a move is invalid, along with a set of relevant board positions.
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

/// A move that a player previously played, along with the new words it introduced and their point values.
/// The value of the whole move is the sum of the words' values.
#[derive(Clone, Debug, Serialize)]
pub struct PlayedMove {
    original_move: Move,
    word_values: Vec<(String, u32)>,
}

/// A position on the board.
pub type Position = (usize, usize);

/// A game board, a 15x15 array of optional `BoardTile`s
#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq)]
pub struct Board([[Option<BoardTile>; 15]; 15]);

impl Board {
    fn new() -> Self {
        Self::default()
    }

    fn with_move_applied(&self, m: &Move) -> Board {
        let mut new_board = *self;
        for ((x, y), tile) in m.tiles.iter().copied() {
            new_board[x][y] = Some(tile);
        }
        new_board
    }

    fn is_empty(&self) -> bool {
        *self == Board::new()
    }

    fn play_move(&mut self, m: &Move) -> Result<PlayedMove, InvalidMove> {
        solve::validate_move(self, m)?;
        let (expanded_move, crossing_moves) = solve::expand_move(self, m);

        if expanded_move.tiles.len() == m.tiles.len()
            && crossing_moves.is_empty()
            && !self.is_empty()
        {
            return Err(InvalidMove::new("That move is disconnected", m.positions()));
        }

        if !m.positions().all(|p| expanded_move.contains_position(p)) {
            return Err(InvalidMove::new(
                "That move is not contiguous",
                m.positions(),
            ));
        }

        // We need to score the move before applying it to the board
        // because scoring takes into account which tiles are new and
        // which are from some prior move that we're playing off of
        let moves: Vec<_> = iter::once(&expanded_move)
            .chain(crossing_moves.iter())
            .map(|m| (m, m.to_word(), solve::score_move(self, m)))
            .collect();

        let new_board = self.with_move_applied(m);

        for (m, word, _score) in &moves {
            if !solve::is_word(word) {
                return Err(InvalidMove::new(
                    format!("'{}' is not a word", word.to_ascii_uppercase()),
                    m.positions(),
                ));
            }
        }

        let word_values = moves
            .into_iter()
            .map(|(_, word, value)| (word, value))
            .collect();

        *self = new_board;

        Ok(PlayedMove {
            original_move: m.clone(),
            word_values,
        })
    }
}

#[derive(Clone, Debug, Serialize)]
enum GameEnd {
    /// The player did not play the last move, and so has some remaining tiles that they lose points for.
    RemainingTiles(Vec<Tile>),
    /// The player used all of their tiles, and gained this many points from other players' remaining tiles.
    PlayedLastMove(u32),
}

#[derive(Clone, Debug, Serialize)]
enum Turn {
    PlayedMove(PlayedMove),
    TilesExchanged,
    GameEnd(GameEnd),
}

#[derive(Default, Clone, Debug, Serialize)]
pub struct Player {
    name: String,
    tiles: Vec<Tile>,
    turns: Vec<Turn>,
}

impl Player {
    fn has_tiles_to_play_move(&self, m: &Move) -> bool {
        let mut player_tiles_count: HashMap<Tile, u8> = HashMap::with_capacity(self.tiles.len());
        for &t in &self.tiles {
            *player_tiles_count.entry(t).or_insert(0) += 1;
        }
        for (_, t) in &m.tiles {
            match player_tiles_count.get_mut(&t.as_tile()) {
                Some(0) => return false,
                Some(n) => *n -= 1,
                None => return false,
            }
        }
        true
    }

    fn remove_played_tiles(&mut self, m: &Move) {
        for (_, bt) in &m.tiles {
            let i = self.tiles.iter().position(|t| bt.as_tile() == *t).unwrap();
            self.tiles.swap_remove(i);
        }
    }

    // fn score(&self) -> u32 {
    //     self.turns.iter().map(|pm| pm.value()).sum()
    // }

    fn refill_tiles_from(&mut self, tile_bag: &mut Vec<Tile>) {
        while self.tiles.len() < 7 && !tile_bag.is_empty() {
            self.tiles.push(tile_bag.pop().unwrap());
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Game {
    board: Board,
    tile_bag: Vec<Tile>,
    players: Vec<Player>,
    whose_turn: usize,
    finished: bool,
}

impl Game {
    pub fn new() -> Self {
        Game {
            players: vec![],
            board: Board::default(),
            tile_bag: Tile::iter_game_count().collect(),
            whose_turn: 0,
            finished: false,
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
        #[rustfmt::skip] macro_rules! player { () => { &mut self.players[self.whose_turn] }; }

        if !player!().has_tiles_to_play_move(m) {
            return Err(InvalidMove::new(
                "Tiles played that you don't have (impossible)",
                vec![],
            ));
        }

        let played_move = self.board.play_move(m)?;
        player!().turns.push(Turn::PlayedMove(played_move));
        player!().remove_played_tiles(m);
        player!().refill_tiles_from(&mut self.tile_bag);

        if !self.finished && self.game_finished_by_tiles() {
            let mut gained_points = 0;
            for (i, p) in self.players.iter_mut().enumerate() {
                if i != self.whose_turn {
                    let tiles = p.tiles.clone();
                    gained_points += tiles.iter().map(|t| t.point_value()).sum::<u32>();
                    p.turns.push(Turn::GameEnd(GameEnd::RemainingTiles(tiles)))
                }
            }

            player!()
                .turns
                .push(Turn::GameEnd(GameEnd::PlayedLastMove(gained_points)));
            self.finished = true;
        }

        self.advance_turn();
        Ok(())
    }

    pub fn exchange_tiles(&mut self) {
        let player = &mut self.players[self.whose_turn];
        self.tile_bag.extend_from_slice(&player.tiles);
        self.tile_bag.shuffle(&mut rand::thread_rng());
        player.turns.push(Turn::TilesExchanged);
        player.tiles.clear();
        player.refill_tiles_from(&mut self.tile_bag);
        self.advance_turn();
    }

    fn index_of_player(&self, name: &str) -> Option<usize> {
        self.players
            .iter()
            .enumerate()
            .find(|(_, p)| p.name == name)
            .map(|(i, _)| i)
    }

    fn advance_turn(&mut self) {
        self.whose_turn += 1;
        self.whose_turn %= self.players.len();
    }

    // fn player(&self, name: &str) -> Option<&Player> {
    //     self.index_of_player(name).map(|i| &self.players[i])
    // }

    pub fn is_players_turn(&self, name: &str) -> bool {
        self.index_of_player(name).unwrap() == self.whose_turn
    }

    pub fn ready_to_play(&self) -> bool {
        self.players.len() >= 2
    }

    pub fn has_player(&self, name: &str) -> bool {
        self.index_of_player(name).is_some()
    }

    pub fn add_player<T: Into<String>>(&mut self, name: T) {
        self.players.push(Player {
            name: name.into(),
            ..Default::default()
        })
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    // fn current_player(&self) -> &Player {
    //     &self.players[self.whose_turn]
    // }

    #[cfg(test)]
    fn current_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.whose_turn]
    }

    fn game_finished_by_tiles(&self) -> bool {
        self.tile_bag.is_empty() && self.players.iter().any(|p| p.tiles.is_empty())
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_char().to_ascii_uppercase())
    }
}

impl Index<usize> for Board {
    type Output = [Option<BoardTile>; 15];

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Board {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

    fn play_move<T: Into<BoardTile> + Copy>(
        g: &mut Game,
        m: &[(usize, usize, T)],
    ) -> Result<(), InvalidMove> {
        let p = g.current_player_mut();
        let mut tiles = Vec::new();
        for (x, y, t) in m.iter().copied() {
            let t = t.into();
            tiles.push(((x, y), t));
            p.tiles.push(t.as_tile());
        }
        g.play_move(&Move::new(tiles))
    }

    fn game(n_players: usize) -> Game {
        const PLAYERS: &[&str] = &["Alice", "Bob", "Charlie", "Daniel", "Elizabeth", "Frank"];
        let mut g = Game::new();
        for p in PLAYERS.iter().take(n_players) {
            g.add_player(*p);
        }
        g.start_game();
        g
    }

    #[test]
    fn extra_50_points_test() {
        fn value_of_letters(s: &str) -> u32 {
            s.chars().map(|c| Tile::from_char(c).point_value()).sum()
        }

        fn play_7_letters(s: &str) -> anyhow::Result<u32> {
            let mut g = game(2);

            for t in s.chars().map(Tile::from_char) {
                g.players[0].tiles.push(t);
            }

            let tiles = s
                .chars()
                .zip(4..)
                .map(|(c, x)| ((x, 7), Tile::from_char(c).as_board_tile()))
                .collect();
            let mov = Move::new(tiles);
            g.play_move(&mov).unwrap();
            let Turn::PlayedMove(pm) = g.players[0].turns[0].clone() else {
                unreachable!()
            };
            assert_eq!(mov, pm.original_move);
            Ok(pm.word_values[0].1)
        }

        const WORDS: &[&str] = &[
            "clonism", "lizards", "pimento", "elmwood", "torques", "souffle", "polling", "cerotic",
            "gestalt", "deskmen", "clewing", "retrace", "woodmen", "netsurf", "outback", "empathy",
            "erepsin", "maftirs", "hornito", "editors", "gerenuk", "opiated", "keglers", "jihadis",
            "burrers", "lixivia", "meeters", "broiler", "dogsled", "anymore", "capeesh", "sojourn",
        ];

        for s in WORDS {
            let letter_value = value_of_letters(s);
            let move_value = play_7_letters(s).unwrap();
            assert_eq!(move_value, letter_value * 2 + 50);
        }
    }

    #[test]
    fn game_end_test() {
        use Tile::*;

        let mut g = game(2);
        g.tile_bag.clear();
        g.players[0].tiles = vec![]; // test::play_move adds necessary tiles on demand
        g.players[1].tiles = vec![Z, Z, Z, Z, Z]; // 5 * 10 points

        play_move(&mut g, &[(7, 7, F), (7, 8, I), (7, 9, N)]).unwrap();
        assert!(g.finished);
        assert_eq!(g.players[0].turns.len(), 2);

        let Turn::GameEnd(GameEnd::PlayedLastMove(50)) = g.players[0].turns[1].clone() else {
            unreachable!()
        };

        let Turn::GameEnd(GameEnd::RemainingTiles(ts)) = g.players[1].turns[0].clone() else {
            unreachable!()
        };
        assert_eq!(ts, &[Z, Z, Z, Z, Z])
    }

    #[test]
    fn two_player_game_test() {
        use BoardTile::*;

        let mut g = game(2);

        // Doesn't cross the center
        play_move(&mut g, &[(0, 0, T), (0, 1, O), (0, 2, P)]).unwrap_err();

        play_move(&mut g, &[(7, 6, O), (7, 7, A), (7, 8, T)]).unwrap();
        play_move(&mut g, &[(5, 8, N), (6, 8, U)]).unwrap();

        // Is disconnected from existing tiles
        play_move(&mut g, &[(0, 0, T), (0, 1, O), (0, 2, P)]).unwrap_err();
    }
}
