use std::fmt::Display;

use crate::{Board, Game, Tile};

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\u{0332}", self.as_char())
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌──────────────────────────────┐")?;
        for i in 0..15 {
            write!(f, "│")?;
            for j in 0..15 {
                match self.tiles[i][j] {
                    Some(t) => write!(f, "{t}")?,
                    None => write!(f, " ")?,
                }
                write!(f, "·")?;
            }
            writeln!(f, "│")?;
        }
        write!(f, "└──────────────────────────────┘")?;
        Ok(())
    }
}

impl Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn tiles_str(ts: &[Tile]) -> String {
            ts.iter().map(Tile::to_string).collect::<String>()
        }

        writeln!(f, "{}", self.board)?;
        for (n, p) in self.players.iter().enumerate() {
            let tiles = tiles_str(&p.tiles);
            writeln!(f, "Player {n} | {} points | ⎣{}⎦", p.score(), tiles)?;
        }
        writeln!(f, "It is player {}'s turn", self.whose_turn)?;
        write!(f, "Remaining tiles: {}", tiles_str(&self.tile_bag))?;
        Ok(())
    }
}

impl Default for Game {
    fn default() -> Self {
        Game::new()
    }
}
