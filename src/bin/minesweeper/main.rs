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
    let difficulties = [(9, 9, 10), (16, 16, 40), (30, 16, 99)];
    let mut difficulty = 0;

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
            Key::Character('c') | Key::Character('C') => {
                difficulty = (difficulty + 1) % difficulties.len();
                let (width, height, mines) = difficulties[difficulty];
                game = Game::new(width, height, mines);
            }
            Key::Character('r') | Key::Character('R') => {
                let (width, height, mines) = difficulties[difficulty];
                game = Game::new(width, height, mines);
            }
            _ => (),
        }
    }

    rawmode.disable()
}
