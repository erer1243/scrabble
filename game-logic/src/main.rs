use game_logic::{Board, Game, Modifier, Tile};

fn main() {
    println!("Empty board:\n{}", game_logic::Board::default());

    print!("All the tiles: ");
    game_logic::Tile::iter_game_count().for_each(|t| print!("{t}"));
    println!();

    let mut board = Board::default();
    board[0][0] = Some(Tile::A);
    board[0][1] = Some(Tile::A);
    board[0][2] = Some(Tile::A);
    println!("Board with some letters:\n{board}");

    let game = Game::new(4);
    println!("Empty randomized game:\n{game}");

    println!("Modifiers on a board:");
    for x in 0..15usize {
        for y in 0..15usize {
            let c = match game_logic::MODIFIERS.get(&(x, y)) {
                Some(Modifier::DoubleLetter) => 'd',
                Some(Modifier::TripleLetter) => 't',
                Some(Modifier::DoubleWord) => 'D',
                Some(Modifier::TripleWord) => 'T',
                None => '‧',
            };
            print!("{c}");
        }
        println!()
    }
}
