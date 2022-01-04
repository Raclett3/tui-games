use crate::key::{Key, KeyInput};
use crate::screen::{Screen, ScreenBuffer};
use std::io::{Read, Write};
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

enum Event {
    Tick,
    Key(Key),
    Terminate,
}

pub trait Game {
    fn render(&self) -> ScreenBuffer;

    fn process_key(&mut self, key: Key);
    fn tick(&mut self);
}

pub fn run_game<T, R, W>(mut game: T, read: R, mut write: W) -> std::io::Result<()>
where
    T: Game,
    R: Read + Send + 'static,
    W: Write,
{
    let (sender, receiver) = channel();

    let key_sender = sender.clone();

    thread::spawn(move || {
        let mut input = KeyInput::new(read);

        loop {
            let result = match input.get_key() {
                Ok(Key::Control('C')) => {
                    let _ = key_sender.send(Ok(Event::Terminate));
                    break;
                }
                Ok(key) => key_sender.send(Ok(Event::Key(key))),
                Err(err) => {
                    let _ = key_sender.send(Err(err));
                    break;
                }
            };

            if result.is_err() {
                break;
            }
        }
    });

    let tick_sender = sender;

    thread::spawn(move || {
        let mut sleep_duration = Duration::from_nanos(1_000_000_000 / 60);
        loop {
            let now = Instant::now();
            thread::sleep(sleep_duration);
            if tick_sender.send(Ok(Event::Tick)).is_err() {
                break;
            }
            sleep_duration = Duration::from_nanos(2_000_000_000 / 60).saturating_sub(now.elapsed());
        }
    });

    write!(write, "\x1b[2J\x1b[H")?;
    let mut screen = Screen::new(write);

    loop {
        screen.render(game.render())?;
        match receiver.recv().unwrap()? {
            Event::Tick => game.tick(),
            Event::Key(key) => game.process_key(key),
            Event::Terminate => break,
        }
    }

    std::mem::drop(receiver);

    Ok(())
}
