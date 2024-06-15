use crate::*;

#[test]
fn solve_for_blanks_test() {
    use Tile::*;

    fn test(
        segments: &[&[Tile]],
        crossing_words: &[Option<(&[Tile], &[Tile])>],
        expecting: Option<&[Tile]>,
    ) {
        let m_fills = solve_for_blanks(segments, crossing_words);
        println!("{segments:?} + {crossing_words:?} -> {m_fills:?}");
        assert_eq!(m_fills.as_ref().map(|v| v.as_slice()), expecting);
    }

    test(&[&[A, P], &[L, E]], &[Some((&[A], &[P, L, E]))], Some(&[P]));
    test(&[&[], &[], &[]], &[None, None], Some(&[A, A]));
    test(&[&[], &[], &[], &[]], &[None, None, None], Some(&[A, A, A]));
    test(
        &[&[], &[], &[], &[], &[]],
        &[
            Some((&[], &[U, C, K])),
            Some((&[F], &[C, K])),
            Some((&[F, U], &[K])),
            Some((&[F, U, C], &[])),
        ],
        Some(&[B, A, C, K]),
    );
    test(
        &[&[W, O, R, D], &[]],
        &[Some((&[S, P, A, N, K], &[]))],
        Some(&[S]),
    );
    test(&[&[N, O, T, A, W, O, R, D], &[]], &[None], None);
    test(
        &[&[W, O, R, D], &[]],
        &[Some((&[N, O, T, A, W, O, R, D], &[]))],
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
    assert_eq!(Board::default().0.len(), 15);
    assert_eq!(Board::default()[0].len(), 15);
}

#[test]
fn load_wordlist() {
    once_cell::sync::Lazy::force(&WORDLIST);
}
