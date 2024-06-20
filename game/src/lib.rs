//! Sources:
//! - <https://hasbro-new.custhelp.com/app/answers/detail/a_id/55/~/what-is-the-total-face-value-of-all-the-scrabble-tiles%3F>
//! - <https://hasbro-new.custhelp.com/app/answers/detail/a_id/19/related/1>
//! - <https://i.pinimg.com/originals/9a/91/ab/9a91abcf38624a17c3b158a56eaa7e84.jpg>
//! - <https://github.com/dwyl/english-words>
//! - <https://www.hasbro.com/common/instruct/Scrabble_(2003).pdf>

mod solve;
mod std_impls;

use forr::forr;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, iter};

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

    // fn iter_alphabet() -> impl Iterator<Item = Tile> {
    //     (b'a'..=b'z').map(|c| Tile::from_ascii(c).unwrap())
    // }

    fn iter_game_count() -> impl Iterator<Item = Tile> {
        forr! { $tile:tt in [A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, Blank] $:
            iter::empty()
                $(.chain(iter::repeat(Self::$tile).take(Self::$tile.number_in_game())))*
        }
    }
}

tile_enum! {
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

/// An attempt to play a word which may or may not be valid
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Move {
    tiles: Vec<(Position, BoardTile)>,

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
    board: Board,
    tile_bag: Vec<Tile>,
    players: Vec<Player>,
    whose_turn: usize,
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

    pub fn play_move(&mut self, m: &Move) -> Result<&PlayedMove, InvalidMove> {
        let n_players = self.players.len();
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
        self.whose_turn %= n_players;
        Ok(player.moves.last().unwrap())
    }

    fn index_of_player(&self, name: &str) -> Option<usize> {
        self.players
            .iter()
            .enumerate()
            .find(|(_, p)| p.name == name)
            .map(|(i, _)| i)
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

    pub fn add_player(&mut self, name: String) {
        self.players.push(Player {
            name,
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

    // fn game_finished_by_tiles(&self) -> bool {
    //     self.tile_bag.is_empty() && self.players.iter().any(|p| p.tiles.is_empty())
    // }
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

    #[test]
    fn two_player_game_test() {
        let mut g = Game::new();
        g.add_player("Alice".to_string());
        g.add_player("Bob".to_string());
        g.start_game();

        macro_rules! play_move {
            ($(($x:expr, $y:expr, $t:ident $($body:tt)?)),*) => {{
                let p = g.current_player_mut();
                $(
                    p.tiles.pop().unwrap();
                    p.tiles.insert(0, Tile::$t);
                )*
                g.play_move(&Move::new(vec![$((($x, $y), BoardTile::$t $($body)? )),*]))
            }}
        }

        play_move![(0, 0, T), (0, 1, O), (0, 2, P)].unwrap_err();

        play_move![(7, 6, O), (7, 7, A), (7, 8, T)].unwrap();
        play_move![(5, 8, N), (6, 8, U)].unwrap();

        play_move![(0, 0, T), (0, 1, O), (0, 2, P)].unwrap_err();
    }
}
