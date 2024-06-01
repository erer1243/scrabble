use game::{Board, Game, Modifier, Tile, MODIFIERS};

fn main() {
    println!("Empty board:\n{}", Board::default());

    print!("All the tiles: ");
    Tile::iter_game_count().for_each(|t| print!("{t}"));
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
            let c = match MODIFIERS.get(&(x, y)) {
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
