use rand::seq::SliceRandom;
use std::num::TryFromIntError;

macro_rules! print_with_color {
    ($format:expr, $fg:expr, $bg:expr $(,$vars:expr)*) => {
        {
            print!("\x1b[{};{}m", $fg, $bg + 10);
            print!($format $(,$vars)*);
            print!("\x1b[0m");
        }
    }
}

const BLACK: usize = 30;
const RED: usize = 31;
const GREEN: usize = 32;
const YELLOW: usize = 33;
const BLUE: usize = 34;
const CYAN: usize = 36;
const WHITE: usize = 37;
const GRAY: usize = 90;
const NUMBER_COLORS: [usize; 9] = [BLACK, CYAN, GREEN, RED, BLUE, RED, GREEN, CYAN, BLACK];

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
    fn new(height: usize, width: usize, mines: usize) -> Self {
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

pub struct Game {
    cursor_x: usize,
    cursor_y: usize,
    board: Board,
    result: Option<GameResult>,
}

impl Game {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        Game {
            cursor_x: 0,
            cursor_y: 0,
            board: Board::new(width, height, mines),
            result: None,
        }
    }

    pub fn render(&self) {
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
                        print_with_color!(" X ", RED, bg);
                    } else if cell.adjacent_mines > 0 {
                        let fg = NUMBER_COLORS[cell.adjacent_mines];
                        print_with_color!(" {} ", fg, bg, cell.adjacent_mines);
                    } else {
                        print_with_color!("   ", WHITE, bg);
                    }
                } else if cell.is_flagged {
                    print_with_color!("[", WHITE, bg);
                    print_with_color!("F", RED, bg);
                    print_with_color!("]", WHITE, bg);
                } else {
                    print_with_color!("[ ]", WHITE, bg);
                }
            }

            println!("\r");
        }
        let status = match self.result {
            Some(GameResult::Success) => "All safe cells revealed! You win! Press R to retry",
            Some(GameResult::Failure) => "You lose... Press R to retry",
            None => "Arrow (or HJKL) - Move cursor, A - Reveal, Space - Reveal (Can perform \"Chord\"), F - Flag, R - Retry",
        };

        println!("{}\r", status);
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
