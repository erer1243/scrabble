use crate::*;

#[test]
fn solve_for_blanks_test() {
    use Tile::*;
    println!(
        "{:?}",
        solve_for_blanks(&[&[A, P], &[L, E]], &[Some((&[A], &[P, L, E]))])
    );

    println!("{:?}", solve_for_blanks(&[&[], &[], &[]], &[None, None]));
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
                "{} => {:?}",
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
