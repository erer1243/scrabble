use std::{
    collections::{HashMap, HashSet},
    io::{self, BufRead, BufReader},
    ops::Range,
    slice,
};

use arrayvec::ArrayVec;
use itertools::Itertools;
use once_cell::sync::Lazy;

use crate::{Board, InvalidMove, Move, Position, Tile};

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

fn tiles_to_bytes(tiles: &[Tile]) -> &[u8] {
    unsafe { std::mem::transmute(tiles) }
}

fn tiles_are_word(word: &[Tile]) -> bool {
    WORDLIST
        .binary_search_by(|w| w.as_bytes().cmp(tiles_to_bytes(word)))
        .is_ok()
}

pub fn validate_move(board: &Board, m: &Move) -> Result<(), InvalidMove> {
    if m.tiles.is_empty() {
        return Err(InvalidMove::new("Empty move", vec![]));
    }

    if m.tiles.len() > 7 {
        return Err(InvalidMove::new(
            "More than 7 tiles played (impossible)",
            m.positions(),
        ));
    }

    if !m.is_straight_line() {
        return Err(InvalidMove::new(
            "That move is not a straight line",
            m.positions(),
        ));
    }

    if !m.is_contiguous() {
        return Err(InvalidMove::new(
            "That move is not contiguous",
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
            .filter(|(x, y)| board.tiles[*x][*y].is_some())
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
        let modifier = if board.tiles[*x][*y].is_some() {
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

        score += t.point_value() * letter_multiplier;
    }

    score * word_multiplier
}

/// Preconditions: m is not empty, m is not a detached single letter, m doesn't overlap a previou move on the board
pub fn expand_move(board: &Board, m: &Move) -> (Move, Vec<Move>) {
    /// Extend the given move to include tiles before and after it
    /// eg PAIN[TER] -> [PAINTER]
    fn expand_move_in_direction(board: &Board, m: &Move, dx: isize, dy: isize) -> Move {
        let get = |x: isize, y: isize| -> Option<Tile> {
            if x < 0 || y < 0 || x >= 15 || y >= 15 {
                None
            } else {
                let pos = (x as usize, y as usize);
                board.get_non_blank(pos).or_else(|| {
                    m.tiles
                        .iter()
                        .find(|(m_pos, _)| pos == *m_pos)
                        .map(|(_, t)| *t)
                })
            }
        };

        // Use hashset to dedup
        let mut new_move_tiles: HashSet<(Position, Tile)> = HashSet::with_capacity(m.tiles.len());

        // Expand forwards
        let (x0, y0) = m.tiles[0].0;
        let (mut x, mut y) = (x0 as isize, y0 as isize);
        while let Some(tile) = get(x + dx, y + dy) {
            x += dx;
            y += dy;
            new_move_tiles.insert(((x as usize, y as usize), tile));
        }

        // Expand backwards
        let (mut x, mut y) = (x as isize, y as isize);
        while let Some(tile) = get(x - dx, y - dy) {
            x -= dx;
            y -= dy;
            new_move_tiles.insert(((x as usize, y as usize), tile));
        }

        Move {
            tiles: new_move_tiles.into_iter().collect(),
        }
    }

    assert!(!m.tiles.is_empty());

    // Special case for 1-length move
    if m.tiles.len() == 1 {
        let m_vertical = expand_move_in_direction(board, m, 1, 0);
        let m_horizontal = expand_move_in_direction(board, m, 0, 1);
        let mut moves = vec![m_vertical, m_horizontal];
        moves.retain(|m| m.tiles.len() > 1);
        assert!(!moves.is_empty());
        let main_word_move = moves.pop().unwrap();
        (main_word_move, moves)
    } else {
        let ((inline_dx, inline_dy), (adjacent_dx, adjacent_dy)) = if m.is_horizontal() {
            ((0, 1), (1, 0))
        } else {
            ((1, 0), (0, 1))
        };

        // Expand the main word
        let main_word_move = expand_move_in_direction(board, m, inline_dx, inline_dy);

        // Find all crossing words
        let mut crossing_words = Vec::new();
        let mut tmp_move = Move::default();
        for t in m.tiles.iter().copied() {
            tmp_move.tiles.clear();
            tmp_move.tiles.push(t);
            let crossing_move =
                expand_move_in_direction(board, &tmp_move, adjacent_dx, adjacent_dy);
            if crossing_move.tiles.len() > 1 {
                crossing_words.push(crossing_move);
            }
        }

        (main_word_move, crossing_words)
    }
}

pub fn solve_for_blanks(
    main_move: &Move,
    crossing_moves: &[Move],
) -> Result<Vec<Tile>, InvalidMove> {
    let num_blanks = main_move
        .tiles
        .iter()
        .filter(|(_pos, t)| *t == Tile::Blank)
        .count();

    if num_blanks == 0 {
        return Ok(vec![]);
    }

    let crossing_move_with_pos = |pos: Position| -> Option<&Move> {
        crossing_moves
            .iter()
            .find(|m| m.tiles.iter().any(|(m_pos, _t)| *m_pos == pos))
    };

    let mut blank_i = 0;
    let mut segments: Vec<Vec<Tile>> = vec![Vec::new(); num_blanks + 1];
    let mut crossing_words: Vec<(Vec<Tile>, Vec<Tile>)> = vec![Default::default(); num_blanks];

    for (pos, tile) in main_move.sorted().tiles {
        if tile == Tile::Blank {
            if let Some(crossing_move) = crossing_move_with_pos(pos) {
                use std::cmp::Ordering::*;
                let mut pre = Vec::new();
                let mut post = Vec::new();
                for (cm_pos, cm_tile) in crossing_move.sorted().tiles {
                    match cm_pos.cmp(&pos) {
                        Less => pre.push(cm_tile),
                        Greater => post.push(cm_tile),
                        Equal => {}
                    }
                }
                crossing_words[blank_i] = (pre, post);
            }

            blank_i += 1;
        } else {
            segments[blank_i].push(tile);
        }
    }

    let segments: Vec<_> = segments.iter().map(Vec::as_slice).collect();
    let crossing_words: Vec<_> = crossing_words
        .iter()
        .map(|(pre, post)| (pre.as_slice(), post.as_slice()))
        .collect();

    solve_for_blanks_segmented(&segments, &crossing_words).ok_or_else(|| {
        let blanks =
            main_move
                .tiles
                .iter()
                .filter_map(|(pos, t)| if *t == Tile::Blank { Some(*pos) } else { None });
        InvalidMove::new("No valid way to fill blank tile(s)", blanks)
    })
}

/// segments represents the tiles before and after each blank in the main word
/// crossing_words represents the tiles before and after a blank in a crossing word
/// eg:
///   D  E
///   D  E
/// AA*BB*CC  => segments: [[AA], [BB], [CC]]
///   F  G    => crossing_words: [([DD], [FF]), ([EE], [GG])]
///   F  G
/// Preconditions: number_of_blanks == segments.len() - 1, segments.len() > 1, crossing_words == segments.len() - 1
fn solve_for_blanks_segmented(
    segments: &[&[Tile]],
    crossing_words: &[(&[Tile], &[Tile])],
) -> Option<Vec<Tile>> {
    assert!(segments.len() > 1);
    assert!(crossing_words.len() == segments.len() - 1);

    let n_blanks = segments.len() - 1;
    let mut fills: Vec<Tile> = Vec::with_capacity(n_blanks);
    let mut buf: ArrayVec<Tile, 32> = ArrayVec::new();

    macro_rules! partial_word {
        () => { partial_word!(. None) };
        ($extra:expr) => { partial_word!(. Some(& $extra)) };
        (. $extra:expr) => {{
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
            let (a, b) = crossing_words[$i];
            buf.clear();
            buf.try_extend_from_slice(a).unwrap();
            buf.push($fill);
            buf.try_extend_from_slice(b).unwrap();
            buf.as_slice()
        }};
    }

    macro_rules! implies {
        ($a:expr, $b:expr) => {
            !$a || $b
        };
    }

    let mut tile = Tile::A;
    while fills.len() < n_blanks {
        let m_range = word_prefix_range(partial_word!(tile));
        let cwi = fills.len();
        let cw = crossing_words[cwi];
        let cw_non_empty = !(cw.0.is_empty() && cw.1.is_empty());

        if m_range.is_some()
            && implies!(cw_non_empty, tiles_are_word(crossing_word!(cwi, tile)))
            && implies!(
                fills.len() + 1 == n_blanks,
                tiles_are_word(partial_word!(tile))
            )
        {
            fills.push(tile);
            tile = Tile::A;
        } else {
            loop {
                match tile.successor() {
                    Some(next_tile) => {
                        tile = next_tile;
                        break;
                    }
                    None => tile = fills.pop()?,
                }
            }
        }
    }

    Some(fills)
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

    unreachable!("precondition unmet or bug in implementation");
}

#[test]
fn find_consequent_moves_test() {
    use Tile::*;

    fn test(
        premoves: impl IntoIterator<Item = Move>,
        m: Move,
        expected_moves: impl IntoIterator<Item = Move>,
    ) {
        use std::collections::HashSet;

        let mut b = Board::new();
        for pm in premoves {
            b = b.with_move_applied(&pm, &[]);
        }

        let (expanded_move, mut crossing_moves) = expand_move(&b, &m);
        crossing_moves.push(expanded_move);

        let expected_moves: HashSet<_> = expected_moves.into_iter().map(|m| m.sorted()).collect();
        let returned_moves: HashSet<_> = crossing_moves.into_iter().map(|m| m.sorted()).collect();

        assert_eq!(expected_moves, returned_moves);
    }

    macro_rules! m {
        ($($t:expr),*) => {
            Move {
                tiles: vec![$($t,)*]
            }
        }
    }

    test(
        [],
        m![((0, 0), A), ((0, 1), B)],
        [m![((0, 0), A), ((0, 1), B)]],
    );
    test(
        [m!(((0, 0), A))],
        m!(((1, 0), B)),
        [m!(((0, 0), A), ((1, 0), B))],
    );
}

#[test]
fn solve_for_blanks_test() {
    use Tile::*;

    fn test(
        segments: &[&[Tile]],
        crossing_words: &[(&[Tile], &[Tile])],
        expecting: Option<&[Tile]>,
    ) {
        let m_fills = solve_for_blanks_segmented(segments, crossing_words);
        println!("{segments:?} + {crossing_words:?} -> {m_fills:?}");
        assert_eq!(m_fills.as_ref().map(|v| v.as_slice()), expecting);
    }

    // Empty crossing word
    let mtcw: (&[Tile], &[Tile]) = (&[], &[]);

    test(&[&[A, P], &[L, E]], &[(&[A], &[P, L, E])], Some(&[P]));
    test(&[&[], &[], &[]], &[mtcw, mtcw], Some(&[A, A]));
    test(&[&[], &[], &[], &[]], &[mtcw, mtcw, mtcw], Some(&[A, A, A]));
    test(
        &[&[], &[], &[], &[], &[]],
        &[
            (&[], &[U, C, K]),
            (&[F], &[C, K]),
            (&[F, U], &[K]),
            (&[F, U, C], &[]),
        ],
        Some(&[B, A, C, K]),
    );
    test(
        &[&[W, O, R, D], &[]],
        &[(&[S, P, A, N, K], &[])],
        Some(&[S]),
    );
    test(&[&[N, O, T, A, W, O, R, D], &[]], &[mtcw], None);
    test(
        &[&[W, O, R, D], &[]],
        &[(&[N, O, T, A, W, O, R, D], &[])],
        None,
    );
}

#[test]
fn binary_search_for_prefix_range_test() {
    let arr = [
        "aa", "ab", "abb", "ac", "ad", "ba", "bb", "bc", "ca", "cb", "cc",
    ];
    let test = |prefix| binary_search_for_prefix_range(&arr, prefix);
    assert_eq!(test(b"a"), Some(0..5));
    assert_eq!(test(b"ab"), Some(1..3));
    assert_eq!(test(b"abb"), Some(2..3));
    assert_eq!(test(b""), Some(0..11));
    assert_eq!(test(b"aaa"), None);
    assert_eq!(test(b"x"), None);
    assert_eq!(test(b"A"), None);

    let test = |prefix| {
        if let Some(Range { start, end }) = binary_search_for_prefix_range(&*WORDLIST, prefix) {
            assert!(!WORDLIST[start - 1].as_bytes().starts_with(prefix));
            assert!(!WORDLIST[end + 1].as_bytes().starts_with(prefix));
            println!(
                "{} -> {:?}",
                std::str::from_utf8(prefix).unwrap(),
                &(&*WORDLIST)[start..end]
            );
        }
    };
    test(b"apple");
    test(b"fuck");
    test(b"poop");
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
    assert_eq!(Board::default().tiles.len(), 15);
    assert_eq!(Board::default().tiles[0].len(), 15);
}

#[test]
fn load_wordlist() {
    once_cell::sync::Lazy::force(&WORDLIST);
}
