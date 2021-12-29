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
}

pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
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
        .filter(move |&(x, y)| y < height && x < width)
}

impl Board {
    fn new(height: usize, width: usize, mines: usize) -> Self {
        let mut cells: Vec<_> = (0..width * height)
            .map(|x| Cell {
                is_mine: x < mines,
                adjacent_mines: 0,
                is_revealed: false,
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
        }
    }

    fn cell_at(&self, x: usize, y: usize) -> &Cell {
        &self.cells[y * self.width + x]
    }

    fn mut_cell_at(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[y * self.width + x]
    }

    fn open(&mut self, x: usize, y: usize) {
        if self.cell_at(x, y).is_revealed {
            return;
        }

        self.mut_cell_at(x, y).is_revealed = true;
        if self.mut_cell_at(x, y).adjacent_mines == 0 {
            for (adj_x, adj_y) in adjacent_cells_coord(x, y, self.width, self.height) {
                self.open(adj_x, adj_y);
            }
        }
    }
}

pub struct Game {
    cursor_x: usize,
    cursor_y: usize,
    board: Board,
}

impl Game {
    pub fn new(width: usize, height: usize, mines: usize) -> Self {
        Game {
            cursor_x: 0,
            cursor_y: 0,
            board: Board::new(width, height, mines),
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
                } else {
                    print_with_color!("[ ]", WHITE, bg);
                }
            }

            println!("\r");
        }
    }

    pub fn move_cursor(&mut self, x: isize, y: isize) {
        let x = x.rem_euclid(self.board.width as isize) as usize;
        let y = y.rem_euclid(self.board.height as isize) as usize;
        self.cursor_x += x;
        self.cursor_x %= self.board.width;
        self.cursor_y += y;
        self.cursor_y %= self.board.height;
    }

    pub fn open(&mut self) {
        self.board.open(self.cursor_x, self.cursor_y);
    }
}
