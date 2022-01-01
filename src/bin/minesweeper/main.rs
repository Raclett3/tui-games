mod game;

use game::MineSweeper;
use nix::unistd::{fork, ForkResult};
use std::io::{stdin, stdout};
use std::io::{Read, Write};
use std::net::TcpListener;
use tui::game::run_game;
use tui::key::{ENABLE_MOUSE, DISABLE_MOUSE};
use tui::rawmode::RawMode;

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

impl<R: Read> TelnetRead<R> {
    fn new(read: R) -> Self {
        TelnetRead {
            state: ParseState::None,
            read,
        }
    }
}

fn main() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().take(3).collect();
    let args_str: Vec<_> = args.iter().map(|x| x.as_str()).collect();
    match args_str.as_slice() {
        [_, "--telnet", ipaddr] => {
            let listener = TcpListener::bind(ipaddr)?;
            for mut stream in listener.incoming().flatten() {
                if let Ok(ForkResult::Child) = unsafe { fork() } {
                    stream.write_all(&[255, 253, 34, 255, 250, 34, 1, 0, 255, 240, 255, 251, 1])?;
                    stream.flush()?;
                    let read_stream = stream.try_clone()?;
                    let mut write_stream = stream.try_clone()?;
                    read_stream.set_read_timeout(Some(std::time::Duration::from_secs(300)))?;
                    write_stream.write_all(ENABLE_MOUSE.as_bytes())?;
                    let _ = run_game(MineSweeper::new(0), TelnetRead::new(read_stream), stream);
                    write_stream.write_all(DISABLE_MOUSE.as_bytes())?;
                    return Ok(());
                }
            }

            Ok(())
        }
        _ => {
            let mut rawmode = RawMode::new();
            println!("{}", ENABLE_MOUSE);
            rawmode.enable()?;
            let result = run_game(MineSweeper::new(0), stdin(), stdout());
            println!("{}", DISABLE_MOUSE);
            rawmode.disable()?;
            result
        }
    }
}
