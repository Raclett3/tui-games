use std::io::{BufWriter, Write};

#[derive(PartialEq, Clone, Copy)]
struct Character {
    character: char,
    color: Option<(usize, usize)>,
}

impl Character {
    fn new(character: char, color: Option<(usize, usize)>) -> Self {
        Character { character, color }
    }
}

fn write_char(dest: &mut impl Write, x: usize, y: usize, c: Character) -> std::io::Result<()> {
    if let Some((fg_color, bg_color)) = c.color {
        write!(
            dest,
            "\x1b[{};{}H\x1b[{};{}m{}\x1b[0m",
            y + 1,
            x + 1,
            fg_color,
            bg_color,
            c.character,
        )
    } else {
        write!(dest, "\x1b[{};{}H{}", y + 1, x + 1, c.character,)
    }
}

pub struct Screen {
    characters: Vec<Vec<Character>>,
    dest: Box<dyn Write>,
}

impl Screen {
    pub fn new(dest: impl Write + 'static) -> Self {
        Screen {
            characters: vec![vec![]],
            dest: Box::new(dest),
        }
    }

    pub fn render(&mut self, new: ScreenBuffer) -> std::io::Result<()> {
        let mut stream = BufWriter::new(&mut self.dest);

        if self.characters.len() < new.characters.len() {
            self.characters.resize_with(new.characters.len(), Vec::new);
        }

        for y in 0.. {
            if self.characters.len().max(new.characters.len()) <= y {
                break;
            }

            for x in 0.. {
                let old_char = self.characters.get(y).and_then(|l| l.get(x));
                let new_char = new.characters.get(y).and_then(|l| l.get(x));

                match (old_char, new_char) {
                    (Some(old), Some(new)) => {
                        if old != new {
                            self.characters[y][x] = *new;
                            write_char(&mut stream, x, y, *new)?;
                        }
                    }
                    (None, Some(new)) => {
                        self.characters[y].push(*new);
                        write_char(&mut stream, x, y, *new)?;
                    }
                    (Some(_), None) => {
                        let left_chars = self.characters[y].len() - x;
                        self.characters[y].truncate(x);
                        write!(stream, "\x1b[{};{}H", y + 1, x + 1)?;
                        for _ in 0..left_chars {
                            write!(stream, " ")?;
                        }
                        break;
                    }
                    (None, None) => break,
                }
            }
        }
        write!(stream, "\x1b[{};1H", self.characters.len() + 1)?;
        stream.flush()
    }
}

pub struct ScreenBuffer {
    characters: Vec<Vec<Character>>,
}

impl ScreenBuffer {
    pub fn new() -> Self {
        ScreenBuffer {
            characters: vec![vec![]],
        }
    }

    pub fn write_color(&mut self, chars: &str, fg_color: usize, bg_color: usize) {
        let line = self.characters.last_mut().unwrap();

        for c in chars.chars() {
            line.push(Character::new(c, Some((fg_color, bg_color + 10))));
        }
    }

    pub fn write(&mut self, chars: &str) {
        let line = self.characters.last_mut().unwrap();

        for c in chars.chars() {
            line.push(Character::new(c, None));
        }
    }

    pub fn new_line(&mut self) {
        self.characters.push(Vec::new());
    }
}
