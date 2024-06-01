use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::{Board, Game, Tile};

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\u{0332}", self.as_char())
    }
}

impl Index<usize> for Board {
    type Output = [Option<Tile>; 15];

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Board {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "┌──────────────────────────────┐")?;
        for i in 0..15 {
            write!(f, "│")?;
            for j in 0..15 {
                match self[i][j] {
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
            writeln!(f, "Player {n} | {} points | ⎣{}⎦", p.score, tiles)?;
        }
        writeln!(f, "It is player {}'s turn", self.whose_turn)?;
        write!(f, "Remaining tiles: {}", tiles_str(&self.tile_bag))?;
        Ok(())
    }
}
