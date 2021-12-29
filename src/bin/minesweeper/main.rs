mod game;

use game::Game;
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
    let mut input = KeyInput::new();
    let mut game = Game::new(9, 9, 10);

    loop {
        cls();
        game.render();

        let key = input.get_key();
        match key {
            Key::Control('C') => break,
            Key::Character('k') | Key::Character('K') | Key::ArrowUp => game.move_cursor(0, -1),
            Key::Character('j') | Key::Character('J') | Key::ArrowDown => game.move_cursor(0, 1),
            Key::Character('h') | Key::Character('H') | Key::ArrowLeft => game.move_cursor(-1, 0),
            Key::Character('l') | Key::Character('L') | Key::ArrowRight => game.move_cursor(1, 0),
            Key::Character('f') | Key::Character('F') => game.flag(),
            Key::Character(' ') => game.reveal(true),
            Key::Character('a') | Key::Character('A') => game.reveal(false),
            Key::Character('r') | Key::Character('R') => game = Game::new(9, 9, 10),
            _ => (),
        }
    }

    rawmode.disable()
}
