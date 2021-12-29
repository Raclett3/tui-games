use std::io::{self, stdin};
use std::os::unix::io::AsRawFd;
use termios::{cfmakeraw, tcsetattr, Termios, TCSAFLUSH};

#[derive(Default)]
pub struct RawMode {
    origin: Option<Termios>,
}

impl RawMode {
    pub fn new() -> Self {
        RawMode::default()
    }

    pub fn enable(&mut self) -> io::Result<()> {
        let stdin_fd = stdin().as_raw_fd();

        if self.origin.is_some() {
            return Ok(());
        }

        let mut term = Termios::from_fd(stdin_fd)?;
        self.origin = Some(term);

        cfmakeraw(&mut term);
        tcsetattr(stdin_fd, TCSAFLUSH, &term)
    }

    pub fn disable(&mut self) -> io::Result<()> {
        let stdin_fd = stdin().as_raw_fd();

        if let Some(term) = &self.origin {
            tcsetattr(stdin_fd, TCSAFLUSH, term)?;
            self.origin = None;
        }

        Ok(())
    }
}
