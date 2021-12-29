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
            Key::ArrowUp => game.move_cursor(0, -1),
            Key::ArrowDown => game.move_cursor(0, 1),
            Key::ArrowLeft => game.move_cursor(-1, 0),
            Key::ArrowRight => game.move_cursor(1, 0),
            Key::Letter(' ') => game.open(),
            _ => (),
        }
    }

    rawmode.disable()
}
