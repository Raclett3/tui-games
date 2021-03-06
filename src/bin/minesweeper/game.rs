use rand::seq::SliceRandom;
use std::num::TryFromIntError;
use tui::game::Game;
use tui::key::{Key, MouseButton};
use tui::screen::ScreenBuffer;

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
    mines: usize,
    is_initialized: bool,
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
        let cells: Vec<_> = (0..width * height)
            .map(|_| Cell {
                is_mine: false,
                adjacent_mines: 0,
                is_revealed: false,
                is_flagged: false,
            })
            .collect();

        Board {
            width,
            height,
            cells,
            mines,
            is_initialized: false,
            safe_cells: height * width - mines,
            revealed_cells: 0,
            revealed_mine: false,
        }
    }

    fn init(&mut self, excluded_x: usize, excluded_y: usize) {
        let excluded_cell = excluded_y * self.width + excluded_x;

        let mut mines: Vec<_> = (0..(self.width * self.height - 1))
            .map(|x| x < self.mines)
            .collect();
        mines.shuffle(&mut rand::thread_rng());

        self.cells
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| *i != excluded_cell)
            .zip(mines.into_iter())
            .for_each(|((_, cell), is_mine)| cell.is_mine = is_mine);

        for (x, y) in cells_coord(self.height, self.width) {
            for (adj_x, adj_y) in adjacent_cells_coord(x, y, self.width, self.height) {
                if self.cells[adj_y * self.width + adj_x].is_mine {
                    self.cells[y * self.width + x].adjacent_mines += 1;
                }
            }
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

        if !self.is_initialized {
            self.init(x, y);
            self.is_initialized = true;
        }

        self.mut_cell_at(x, y).is_revealed = true;
        self.revealed_cells += 1;

        if self.cell_at(x, y).is_mine {
            self.revealed_mine = true;
            return;
        }

        if self.is_cleared() {
            self.cells
                .iter_mut()
                .filter(|x| x.is_mine)
                .for_each(|x| x.is_flagged = true);
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

    fn contains_coord(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
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
    hold_mouse_buttons: (bool, bool),
    result: Option<GameResult>,
    ticks_elapsed: usize,
    is_started: bool,
}

impl MineSweeper {
    pub fn new(difficulty: usize) -> Self {
        let (width, height, mines) = DIFFICULTIES[difficulty];

        MineSweeper {
            cursor_x: 0,
            cursor_y: 0,
            difficulty,
            board: Board::new(width, height, mines),
            hold_mouse_buttons: (false, false),
            result: None,
            ticks_elapsed: 0,
            is_started: false,
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

    pub fn move_cursor_smart(&mut self, x: isize, y: isize) {
        if self.result.is_some() {
            return;
        }

        let mut is_first_move = true;

        loop {
            let x_mod = x.rem_euclid(self.board.width as isize) as usize;
            let y_mod = y.rem_euclid(self.board.height as isize) as usize;

            let next_x = (self.cursor_x + x_mod) % self.board.width;
            let next_y = (self.cursor_y + y_mod) % self.board.height;

            let current_cell = self.board.cell_at(self.cursor_x, self.cursor_y);
            let next_cell = self.board.cell_at(next_x, next_y);

            if (next_x as isize - self.cursor_x as isize).signum() != x.signum()
                || (next_y as isize - self.cursor_y as isize).signum() != y.signum()
                || current_cell.is_revealed != next_cell.is_revealed
            {
                if is_first_move {
                    self.cursor_x = next_x;
                    self.cursor_y = next_y;
                }

                break;
            }

            self.cursor_x = next_x;
            self.cursor_y = next_y;

            is_first_move = false;
        }
    }

    pub fn set_cursor(&mut self, x: usize, y: usize) {
        if self.result.is_some() {
            return;
        }

        self.cursor_x = x;
        self.cursor_y = y;
    }

    pub fn reveal(&mut self, chord: bool) {
        if self.result.is_some() || self.board.cell_at(self.cursor_x, self.cursor_y).is_flagged {
            return;
        }

        self.is_started = true;

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
        self.is_started = true;

        if self.result.is_none() {
            self.board.flag(self.cursor_x, self.cursor_y);
        }
    }
}

impl Game for MineSweeper {
    fn render(&self) -> ScreenBuffer {
        let mut screen = ScreenBuffer::new();

        let time = self.ticks_elapsed / 60;
        let flags = self.board.cells.iter().filter(|x| x.is_flagged).count();
        let mines = self.board.mines.saturating_sub(flags);
        screen.write_color(&format!(" {:0>3}   {:0>3} ", mines, time), RED, WHITE);
        screen.new_line();

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
                screen.write("Arrow (or HJKL) - Move cursor, Shift + HJKL - Smart cursor");
                screen.new_line();
                screen.write("A - Reveal, Space - Reveal (Can perform \"Chord\"), F - Flag");
                screen.new_line();
                screen.write("R - Retry, C - Change difficulty, Ctrl-C - Quit");
            }
        }

        screen
    }

    fn process_key(&mut self, key: Key) {
        match key {
            Key::Character('k') | Key::ArrowUp => self.move_cursor(0, -1),
            Key::Character('j') | Key::ArrowDown => self.move_cursor(0, 1),
            Key::Character('h') | Key::ArrowLeft => self.move_cursor(-1, 0),
            Key::Character('l') | Key::ArrowRight => self.move_cursor(1, 0),
            Key::Character('K') => self.move_cursor_smart(0, -1),
            Key::Character('J') => self.move_cursor_smart(0, 1),
            Key::Character('H') => self.move_cursor_smart(-1, 0),
            Key::Character('L') => self.move_cursor_smart(1, 0),
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
            Key::Mousedown(MouseButton::Left, x, y) if y >= 2 => {
                let x = (x - 1) / 3;
                let y = y - 2;
                if self.board.contains_coord(x, y) {
                    self.set_cursor(x, y);
                }
                self.hold_mouse_buttons.0 = true;
            }
            Key::Mouseup(MouseButton::Left, x, y) if y >= 2 => {
                let x = (x - 1) / 3;
                let y = y - 2;
                if self.board.contains_coord(x, y)
                    && (self.board.cell_at(x, y).is_revealed || !self.hold_mouse_buttons.1)
                {
                    self.set_cursor(x, y);
                    self.reveal(self.hold_mouse_buttons.0 && self.hold_mouse_buttons.1);
                }
                self.hold_mouse_buttons.0 = false;
            }
            Key::Mousedown(MouseButton::Right, x, y) if y >= 2 => {
                let x = (x - 1) / 3;
                let y = y - 2;
                if self.board.contains_coord(x, y) {
                    self.set_cursor(x, y);
                    self.flag();
                }
                self.hold_mouse_buttons.1 = true;
            }
            Key::Mouseup(MouseButton::Right, x, y) if y >= 2 => {
                let x = (x - 1) / 3;
                let y = y - 2;
                if self.hold_mouse_buttons.0
                    && self.hold_mouse_buttons.1
                    && self.board.contains_coord(x, y)
                    && self.board.cell_at(x, y).is_revealed
                {
                    self.set_cursor(x, y);
                    self.reveal(true);
                }
                self.hold_mouse_buttons.1 = false;
            }
            _ => (),
        }
    }

    fn tick(&mut self) {
        if self.is_started && self.result.is_none() {
            self.ticks_elapsed += 1;
        }
    }
}
