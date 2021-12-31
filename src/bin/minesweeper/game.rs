use crate::screen::ScreenBuffer;
use rand::seq::SliceRandom;
use std::num::TryFromIntError;
use tui::key::Key;

const BLACK: usize = 30;
const RED: usize = 31;
const GREEN: usize = 32;
const YELLOW: usize = 33;
const BLUE: usize = 34;
const CYAN: usize = 36;
const WHITE: usize = 37;
const GRAY: usize = 90;
const NUMBER_COLORS: [usize; 9] = [BLACK, CYAN, GREEN, RED, BLUE, RED, GREEN, CYAN, BLACK];

static DIFFICULTIES: [(usize, usize, usize); 3] = [(9, 9, 10), (16, 16, 40), (30, 16, 99)];

struct Cell {
    pub is_mine: bool,
    pub is_revealed: bool,
    pub adjacent_mines: usize,
    pub is_flagged: bool,
}

pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    safe_cells: usize,
    revealed_cells: usize,
    revealed_mine: bool,
}

fn cells_coord(width: usize, height: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..height).flat_map(move |y| (0..width).map(move |x| (y, x)))
}

fn adjacent_cells_coord(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> impl Iterator<Item = (usize, usize)> {
    (-1..=1)
        .flat_map(move |dy| {
            (-1..=1).flat_map::<Result<_, TryFromIntError>, _>(move |dx| {
                Ok((
                    usize::try_from(x as isize + dx)?,
                    usize::try_from(y as isize + dy)?,
                ))
            })
        })
        .filter(move |&(adj_x, adj_y)| !(adj_x == x && adj_y == y))
        .filter(move |&(x, y)| y < height && x < width)
}

impl Board {
    fn new(width: usize, height: usize, mines: usize) -> Self {
        let mut cells: Vec<_> = (0..width * height)
            .map(|x| Cell {
                is_mine: x < mines,
                adjacent_mines: 0,
                is_revealed: false,
                is_flagged: false,
            })
            .collect();
        cells.shuffle(&mut rand::thread_rng());

        for (x, y) in cells_coord(height, width) {
            for (adj_x, adj_y) in adjacent_cells_coord(x, y, width, height) {
                if cells[adj_y * width + adj_x].is_mine {
                    cells[y * width + x].adjacent_mines += 1;
                }
            }
        }

        Board {
            width,
            height,
            cells,
            safe_cells: height * width - mines,
            revealed_cells: 0,
            revealed_mine: false,
        }
    }

    fn cell_at(&self, x: usize, y: usize) -> &Cell {
        &self.cells[y * self.width + x]
    }

    fn mut_cell_at(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[y * self.width + x]
    }

    fn chord_reveal(&mut self, x: usize, y: usize) {
        if !self.cell_at(x, y).is_revealed {
            self.reveal(x, y);
            return;
        }

        let adjacent_flags = adjacent_cells_coord(x, y, self.width, self.height)
            .filter(|&(x, y)| self.cell_at(x, y).is_flagged)
            .count();

        if adjacent_flags != self.cell_at(x, y).adjacent_mines {
            return;
        }

        for (adj_x, adj_y) in adjacent_cells_coord(x, y, self.width, self.height) {
            if self.cell_at(adj_x, adj_y).is_revealed {
                continue;
            }

            self.reveal(adj_x, adj_y);
        }
    }

    fn reveal(&mut self, x: usize, y: usize) {
        if self.cell_at(x, y).is_revealed || self.cell_at(x, y).is_flagged {
            return;
        }

        self.mut_cell_at(x, y).is_revealed = true;
        self.revealed_cells += 1;

        if self.cell_at(x, y).is_mine {
            self.revealed_mine = true;
            return;
        }

        if self.cell_at(x, y).adjacent_mines == 0 {
            for (adj_x, adj_y) in adjacent_cells_coord(x, y, self.width, self.height) {
                self.reveal(adj_x, adj_y);
            }
        }
    }

    fn flag(&mut self, x: usize, y: usize) {
        if self.cell_at(x, y).is_revealed {
            return;
        }

        self.mut_cell_at(x, y).is_flagged ^= true;
    }

    fn is_cleared(&self) -> bool {
        self.revealed_cells >= self.safe_cells
    }
}

enum GameResult {
    Success,
    Failure,
}

pub struct MineSweeper {
    cursor_x: usize,
    cursor_y: usize,
    difficulty: usize,
    board: Board,
    result: Option<GameResult>,
}

impl MineSweeper {
    pub fn new(difficulty: usize) -> Self {
        let (width, height, mines) = DIFFICULTIES[difficulty];

        MineSweeper {
            cursor_x: 0,
            cursor_y: 0,
            difficulty,
            board: Board::new(width, height, mines),
            result: None,
        }
    }

    pub fn move_cursor(&mut self, x: isize, y: isize) {
        if self.result.is_some() {
            return;
        }

        let x = x.rem_euclid(self.board.width as isize) as usize;
        let y = y.rem_euclid(self.board.height as isize) as usize;
        self.cursor_x += x;
        self.cursor_x %= self.board.width;
        self.cursor_y += y;
        self.cursor_y %= self.board.height;
    }

    pub fn reveal(&mut self, chord: bool) {
        if self.result.is_some() || self.board.cell_at(self.cursor_x, self.cursor_y).is_flagged {
            return;
        }

        if chord {
            self.board.chord_reveal(self.cursor_x, self.cursor_y);
        } else {
            self.board.reveal(self.cursor_x, self.cursor_y);
        }

        if self.board.revealed_mine {
            self.result = Some(GameResult::Failure);
        } else if self.board.is_cleared() {
            self.result = Some(GameResult::Success);
        }
    }

    pub fn flag(&mut self) {
        if self.result.is_none() {
            self.board.flag(self.cursor_x, self.cursor_y);
        }
    }
}

pub trait Game {
    fn render(&self) -> ScreenBuffer;

    fn process_key(&mut self, key: Key);
}

impl Game for MineSweeper {
    fn render(&self) -> ScreenBuffer {
        let mut screen = ScreenBuffer::new();

        for y in 0..self.board.height {
            for x in 0..self.board.width {
                let cell = self.board.cell_at(x, y);

                let bg = if self.cursor_x == x && self.cursor_y == y {
                    YELLOW
                } else if cell.is_revealed {
                    WHITE
                } else {
                    GRAY
                };

                if cell.is_revealed {
                    if cell.is_mine {
                        screen.write_color(" X ", RED, bg);
                    } else if cell.adjacent_mines > 0 {
                        let fg = NUMBER_COLORS[cell.adjacent_mines];
                        screen.write_color(&format!(" {} ", cell.adjacent_mines), fg, bg);
                    } else {
                        screen.write_color("   ", WHITE, bg);
                    }
                } else if cell.is_flagged {
                    screen.write_color("[", WHITE, bg);
                    screen.write_color("F", RED, bg);
                    screen.write_color("]", WHITE, bg);
                } else {
                    screen.write_color("[ ]", WHITE, bg);
                }
            }

            screen.new_line();
        }
        match self.result {
            Some(GameResult::Success) => {
                screen.write("All safe cells revealed! You win! Press R to retry")
            }
            Some(GameResult::Failure) => screen.write("You lose... Press R to retry"),
            None => {
                screen.write("Arrow (or HJKL) - Move cursor, A - Reveal, Space - Reveal (Can perform \"Chord\"), F - Flag");
                screen.new_line();
                screen.write("R - Retry, C - Change difficulty");
            }
        }

        screen
    }

    fn process_key(&mut self, key: Key) {
        match key {
            Key::Character('k') | Key::Character('K') | Key::ArrowUp => self.move_cursor(0, -1),
            Key::Character('j') | Key::Character('J') | Key::ArrowDown => self.move_cursor(0, 1),
            Key::Character('h') | Key::Character('H') | Key::ArrowLeft => self.move_cursor(-1, 0),
            Key::Character('l') | Key::Character('L') | Key::ArrowRight => self.move_cursor(1, 0),
            Key::Character('f') | Key::Character('F') => self.flag(),
            Key::Character(' ') => self.reveal(true),
            Key::Character('a') | Key::Character('A') => self.reveal(false),
            Key::Character('c') | Key::Character('C') => {
                let difficulty = (self.difficulty + 1) % DIFFICULTIES.len();
                *self = MineSweeper::new(difficulty);
            }
            Key::Character('r') | Key::Character('R') => {
                *self = MineSweeper::new(self.difficulty);
            }
            _ => (),
        }
    }
}
