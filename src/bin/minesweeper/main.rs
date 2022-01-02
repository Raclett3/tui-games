mod game;

use game::MineSweeper;
use nix::unistd::{fork, ForkResult};
use std::io::{stdin, stdout};
use std::net::TcpListener;
use tui::runner::{run_game_on_telnet, run_game_on_tty};

fn main() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().take(3).collect();
    let args_str: Vec<_> = args.iter().map(|x| x.as_str()).collect();
    match args_str.as_slice() {
        [_, "--telnet", ipaddr] => {
            let listener = TcpListener::bind(ipaddr)?;
            for stream in listener.incoming().flatten() {
                if let Ok(ForkResult::Child) = unsafe { fork() } {
                    let read_stream = stream.try_clone()?;
                    let write_stream = stream;
                    read_stream.set_read_timeout(Some(std::time::Duration::from_secs(300)))?;

                    let _ = run_game_on_telnet(MineSweeper::new(0), read_stream, write_stream);
                    return Ok(());
                }
            }

            Ok(())
        }
        _ => {
            run_game_on_tty(MineSweeper::new(0), stdin(), stdout())
        }
    }
}
