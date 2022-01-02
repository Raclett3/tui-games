use std::io::{Read, Write};
use crate::key::{Key, KeyInput};
use crate::screen::{Screen, ScreenBuffer};

pub trait Game {
    fn render(&self) -> ScreenBuffer;

    fn process_key(&mut self, key: Key);
}

pub fn run_game<T, R, W>(mut game: T, read: R, mut write: W) -> std::io::Result<()>
where
    T: Game,
    R: Read,
    W: Write,
{
    let mut input = KeyInput::new(read);

    write!(write, "\x1b[2J\x1b[H")?;
    let mut screen = Screen::new(write);

    loop {
        screen.render(game.render())?;
        let key = input.get_key()?;
        if key == Key::Control('C') {
            break;
        }
        game.process_key(key);
    }

    Ok(())
}