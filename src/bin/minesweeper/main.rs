mod game;
mod screen;

use game::{run_game, MineSweeper};
use std::io::{stdin, stdout};
use tui::rawmode::RawMode;

fn main() -> std::io::Result<()> {
    let mut rawmode = RawMode::new();
    rawmode.enable()?;
    let result = run_game(MineSweeper::new(0), stdin(), stdout());
    rawmode.disable()?;
    result
}
