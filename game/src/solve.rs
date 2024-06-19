use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader},
};

use crate::{Board, BoardTile, InvalidMove, Move, Position};
use itertools::Itertools;
use once_cell::sync::Lazy;

static WORDLIST: Lazy<Vec<String>> = Lazy::new(|| {
    let reader;

    #[cfg(debug_assertions)]
    {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("words.txt");
        reader = std::fs::File::open(path).unwrap();
    }

    #[cfg(not(debug_assertions))]
    {
        reader = &include_bytes!("../words.txt")[..];
    }

    let buf_reader = BufReader::new(reader);
    let words = buf_reader
        .lines()
        .collect::<io::Result<Vec<String>>>()
        .unwrap();

    assert!(words.windows(2).all(|w| w[0] < w[1]));
    assert!(words
        .iter()
        .all(|w| w.chars().all(|c| c.is_ascii_lowercase())));

    words
});

pub fn is_word(s: &str) -> bool {
    WORDLIST.binary_search_by_key(&s, |w| w.as_str()).is_ok()
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

#[derive(Clone, Copy, Debug)]
pub enum Modifier {
    DoubleLetter,
    TripleLetter,
    DoubleWord,
    TripleWord,
}

pub fn validate_move(board: &Board, m: &Move) -> Result<(), InvalidMove> {
    if m.tiles.is_empty() {
        return Err(InvalidMove::new("Empty move (impossible)", vec![]));
    }

    if m.tiles.len() > 7 {
        return Err(InvalidMove::new(
            "More than 7 tiles played (impossible)",
            m.positions(),
        ));
    }

    if let Some(pair) = m.tiles.iter().combinations(2).find(|ts| ts[0].0 == ts[1].0) {
        return Err(InvalidMove::new(
            "Move is self-overlapping (impossible)",
            [pair[0].0],
        ));
    }

    if !m.is_straight_line() {
        return Err(InvalidMove::new(
            "That move is not a straight line",
            m.positions(),
        ));
    }

    if board.is_empty() {
        // First move of the game
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
    } else {
        // Not the first move of the game
        let overlaps: Vec<_> = m
            .positions()
            .filter(|(x, y)| board[*x][*y].is_some())
            .collect();
        if !overlaps.is_empty() {
            return Err(InvalidMove::new(
                "Some spaces in that move are already covered (impossible)",
                overlaps,
            ));
        }
    }

    Ok(())
}

pub fn score_move(board: &Board, m: &Move) -> u32 {
    use Modifier::*;

    let mut score = 0;
    let mut word_multiplier = 1;

    for ((x, y), t) in &m.tiles {
        let modifier = if board[*x][*y].is_some() {
            None
        } else {
            MODIFIERS.get(&(*x, *y))
        };

        let letter_multiplier = match modifier {
            None => 1,
            Some(DoubleLetter) => 2,
            Some(TripleLetter) => 3,
            Some(DoubleWord) => {
                word_multiplier *= 2;
                1
            }
            Some(TripleWord) => {
                word_multiplier *= 3;
                1
            }
        };

        score += t.as_tile().point_value() * letter_multiplier;
    }

    score * word_multiplier
}

/// Preconditions: m is not empty, m is not a detached single letter, m doesn't overlap a previous move on the board
pub fn expand_move(board: &Board, m: &Move) -> (Move, Vec<Move>) {
    /// Extend the given move to include tiles before and after it
    /// eg PAIN[TER] -> [PAINTER]
    fn expand_move_in_axis(board: &Board, m: &Move, (dx, dy): (isize, isize)) -> Move {
        let get = |x: isize, y: isize| -> Option<BoardTile> {
            if x < 0 || y < 0 || x >= 15 || y >= 15 {
                None
            } else {
                let pos @ (x, y) = (x as usize, y as usize);
                board[x][y].or_else(|| {
                    m.tiles
                        .iter()
                        .find(|(m_pos, _)| pos == *m_pos)
                        .map(|(_, t)| *t)
                })
            }
        };

        let mut tiles: Vec<(Position, BoardTile)> = Vec::with_capacity(m.tiles.len());

        // Start with initial tile
        let initial_tile @ ((x0, y0), _) = m.tiles[0];
        tiles.push(initial_tile);

        // Expand forwards
        let (mut x, mut y) = (x0 as isize, y0 as isize);
        while let Some(tile) = get(x + dx, y + dy) {
            x += dx;
            y += dy;
            tiles.push(((x as usize, y as usize), tile));
        }

        // Expand backwards
        let (mut x, mut y) = (x0 as isize, y0 as isize);
        while let Some(tile) = get(x - dx, y - dy) {
            x -= dx;
            y -= dy;
            tiles.push(((x as usize, y as usize), tile));
        }

        Move::new(tiles)
    }

    assert!(!m.tiles.is_empty());

    // Special case for 1-length move
    if m.tiles.len() == 1 {
        let m_vertical = expand_move_in_axis(board, m, (1, 0));
        let m_horizontal = expand_move_in_axis(board, m, (0, 1));

        if m_vertical.tiles.len() == 1 && m_horizontal.tiles.len() == 1 {
            (m_vertical, vec![])
        } else {
            let mut moves = vec![m_vertical, m_horizontal];
            moves.retain(|m| m.tiles.len() > 1);
            assert!(!moves.is_empty());
            let main_word_move = moves.pop().unwrap();
            (main_word_move, moves)
        }
    } else {
        let (parallel, perpendicular) = if m.is_horizontal() {
            ((0, 1), (1, 0))
        } else {
            ((1, 0), (0, 1))
        };

        // Expand the main word
        let main_word_move = expand_move_in_axis(board, m, parallel);

        // Find all crossing words
        let mut crossing_words = Vec::new();
        let mut tmp_move = Move::default();
        for t in m.tiles.iter().copied() {
            tmp_move.tiles.clear();
            tmp_move.tiles.push(t);
            let crossing_move = expand_move_in_axis(board, &tmp_move, perpendicular);
            if crossing_move.tiles.len() > 1 {
                crossing_words.push(crossing_move);
            }
        }

        (main_word_move, crossing_words)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn expand_move_test() {
        use BoardTile::*;

        fn test(
            premoves: impl IntoIterator<Item = Move>,
            m: Move,
            expected_moves: impl IntoIterator<Item = Move>,
        ) {
            use std::collections::HashSet;

            let mut b = Board::new();
            for pm in premoves {
                b = b.with_move_applied(&pm);
            }

            let (expanded_move, mut crossing_moves) = expand_move(&b, &m);
            crossing_moves.push(expanded_move);

            fn sort(ms: impl IntoIterator<Item = Move>) -> HashSet<Move> {
                use itertools::Itertools;
                ms.into_iter().update(|m| m.sort()).collect()
            }

            assert_eq!(sort(expected_moves), sort(crossing_moves));
        }

        macro_rules! m {
        ($(($x:expr, $y:expr, $t:expr)),*) => { Move::new(vec![$((($x, $y), $t)),*]) }
    }

        test([], m![(0, 0, A), (0, 1, B)], [m![(0, 0, A), (0, 1, B)]]);
        test([m![(0, 0, X)]], m![(1, 0, Y)], [m![(0, 0, X), (1, 0, Y)]]);
        test(
            [m![(10, 10, O), (10, 11, A), (10, 12, T)]],
            m![(8, 12, N), (9, 12, U)],
            [m![(8, 12, N), (9, 12, U), (10, 12, T)]],
        );
    }

    #[test]
    fn load_wordlist() {
        once_cell::sync::Lazy::force(&WORDLIST);
    }
}
