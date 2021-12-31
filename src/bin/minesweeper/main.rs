mod game;
mod screen;

use game::{Game, MineSweeper};
use std::io::{stdin, stdout};
use tui::{
    key::{Key, KeyInput},
    rawmode::RawMode,
};

fn cls() {
    println!("\x1b[2J\x1b[H");
}

fn main() -> std::io::Result<()> {
    let mut rawmode = RawMode::new();
    rawmode.enable()?;
    let mut input = KeyInput::new(stdin());
    let mut game = MineSweeper::new(0);

    cls();
    let mut screen = screen::Screen::new(stdout());

    loop {
        screen.render(game.render())?;

        let key = input.get_key();

        if key == Key::Control('C') {
            break;
        }

        game.process_key(key);
    }

    rawmode.disable()
}
