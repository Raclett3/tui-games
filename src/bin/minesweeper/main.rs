use tui::{
    key::{Key, KeyInput},
    rawmode::RawMode,
};

struct Game {
    cursor_x: usize,
    cursor_y: usize,
    board_w: usize,
    board_h: usize,
}

macro_rules! print_with_color {
    ($format:expr, $fg:expr, $bg:expr $(,$vars:expr)*) => {
        {
            print!("\x1b[{};{}m", $fg, $bg + 10);
            print!($format, $(,$vars)*);
            print!("\x1b[0m");
        }
    }
}

fn cls() {
    println!("\x1b[2J\x1b[H");
}

const GRAY: usize = 90;
const RED: usize = 31;
const WHITE: usize = 37;

impl Game {
    fn new() -> Self {
        Game {
            cursor_x: 0,
            cursor_y: 0,
            board_w: 9,
            board_h: 9,
        }
    }

    fn render(&self) {
        for y in 0..self.board_h {
            for x in 0..self.board_w {
                if self.cursor_x == x && self.cursor_y == y {
                    print_with_color!("[ ]", WHITE, RED);
                } else {
                    print_with_color!("[ ]", WHITE, GRAY);
                }
            }

            println!("\r");
        }
    }

    fn move_cursor(&mut self, x: isize, y: isize) {
        let x = x.rem_euclid(self.board_w as isize) as usize;
        let y = y.rem_euclid(self.board_h as isize) as usize;
        self.cursor_x += x;
        self.cursor_x %= self.board_w;
        self.cursor_y += y;
        self.cursor_y %= self.board_h;
    }
}

fn main() -> std::io::Result<()> {
    let mut rawmode = RawMode::new();
    rawmode.enable()?;
    let mut input = KeyInput::new();
    let mut game = Game::new();

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
            _ => (),
        }
    }

    rawmode.disable()
}
