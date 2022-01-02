use crate::game::{run_game, Game};
use crate::key::{DISABLE_MOUSE, ENABLE_MOUSE};
use crate::rawmode::RawMode;
use std::io::{Read, Write};

#[derive(Clone, Copy)]
enum ParseState {
    None,
    WaitCommand,
    WaitOption,
    WaitSubnegtiation,
}

impl ParseState {
    fn is_plain_char(&mut self, byte: u8) -> bool {
        let (next_state, result) = match (*self, byte) {
            (_, 255) => (ParseState::WaitCommand, false),
            (ParseState::None, _) => (ParseState::None, true),
            (ParseState::WaitCommand, 250) => (ParseState::WaitSubnegtiation, false),
            (ParseState::WaitCommand, 251..=254) => (ParseState::WaitOption, false),
            (ParseState::WaitCommand, _) => (ParseState::None, false),
            (ParseState::WaitOption, _) => (ParseState::None, false),
            (ParseState::WaitSubnegtiation, _) => (ParseState::WaitSubnegtiation, false),
        };

        *self = next_state;
        result
    }
}

struct TelnetRead<R: Read> {
    state: ParseState,
    read: R,
}

impl<R: Read> TelnetRead<R> {
    fn new(read: R) -> Self {
        TelnetRead {
            state: ParseState::None,
            read,
        }
    }
}

impl<R: Read> Read for TelnetRead<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let size = self.read.read(buf)?;
        let mut ptr = 0;
        for i in 0..size {
            if self.state.is_plain_char(buf[i]) {
                buf.swap(i, ptr);
                ptr += 1;
            }
        }

        Ok(ptr)
    }
}

pub fn run_game_on_telnet<G, R, W>(game: G, read: R, mut write: W) -> std::io::Result<()>
where
    G: Game,
    R: Read,
    W: Write,
{
    write.write_all(&[255, 253, 34, 255, 250, 34, 1, 0, 255, 240, 255, 251, 1])?;
    write.write_all(ENABLE_MOUSE.as_bytes())?;
    write.flush()?;
    let result = run_game(game, TelnetRead::new(read), &mut write);
    write.write_all(DISABLE_MOUSE.as_bytes())?;
    result
}

pub fn run_game_on_tty<G, R, W>(game: G, read: R, mut write: W) -> std::io::Result<()>
where
    G: Game,
    R: Read,
    W: Write,
{
    let mut rawmode = RawMode::new();
    write.write_all(ENABLE_MOUSE.as_bytes())?;
    rawmode.enable()?;
    let result = run_game(game, read, &mut write);
    rawmode.disable()?;
    write.write_all(DISABLE_MOUSE.as_bytes())?;
    result
}
