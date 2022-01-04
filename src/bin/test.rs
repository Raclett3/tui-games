use std::io;
use tui::key::*;
use tui::rawmode::*;

fn main() -> io::Result<()> {
    let mut raw_mode = RawMode::new();
    raw_mode.enable()?;
    println!("{}", ENABLE_MOUSE);
    let mut input = KeyInput::new(io::stdin());
    loop {
        let next = input.get_key()?;
        print!("{:?}\r\n", next);
        if next == Key::Control('C') {
            break;
        }
    }
    raw_mode.disable()?;
    println!("{}", DISABLE_MOUSE);
    Ok(())
}
