use game_logic::{Board, Tile, WordList};

fn main() {
    println!("Empty board:\n{}", game_logic::Board::default());

    print!("All the tiles: ");
    game_logic::Tile::iter_game_count().for_each(|t| print!("{t}"));
    println!();

    let mut board = Board::default();
    board[0][0] = Some(Tile::A);
    board[0][1] = Some(Tile::A);
    board[0][2] = Some(Tile::A);
    println!("{board}");

    let wordlist = WordList::load().unwrap();
    println!("{wordlist:?}");
}
