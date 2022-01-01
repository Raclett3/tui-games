use std::io::Read;

#[derive(PartialEq, Debug)]
pub enum MouseButton {
    Left,
    Right,
}

#[derive(PartialEq, Debug)]
pub enum Key {
    ArrowUp,
    ArrowDown,
    ArrowRight,
    ArrowLeft,
    Delete,
    Escape,
    Return,
    Tab,
    Control(char),
    Character(char),
    Mousedown(MouseButton, usize, usize),
    Mouseup(MouseButton, usize, usize),
}

pub const ENABLE_MOUSE: &str = "\x1b[?1000h\x1b[?1002h\x1b[?1015h\x1b[?1006h";
pub const DISABLE_MOUSE: &str = "\x1b[?1006l\x1b[?1015l\x1b[?1002l\x1b[?1000l";

fn process_input(input: &mut KeyInput) -> std::io::Result<Option<Key>> {
    let key = match input.next_char()? {
        9 => Key::Tab,
        13 => Key::Return,
        27 => {
            let key = process_escape(input);
            input.flush_buf();
            return Ok(key);
        }
        127 => Key::Delete,

        x @ 1..=31 => Key::Control((x + b'A' - 1) as char),
        x @ 32..=126 => Key::Character(x as char),

        // UTF-8 multibytes characters
        0b11000000..=0b11011111 => {
            input.skip(1);
            return Ok(None);
        }
        0b11100000..=0b11101111 => {
            input.skip(2);
            return Ok(None);
        }
        0b11110000..=0b11110111 => {
            input.skip(3);
            return Ok(None);
        }

        _ => return Ok(None),
    };

    Ok(Some(key))
}

fn process_escape(input: &mut KeyInput) -> Option<Key> {
    let key = match input.next_char_in_buf() {
        None => Key::Escape,
        Some(b'[') => match input.next_char_in_buf()? {
            b'<' => {
                let mut params_str = String::new();
                while let Some(next) =
                    input.next_char_in_buf_if(|c| (b'0'..=b'9').contains(&c) || c == b';')
                {
                    params_str.push(next as char);
                }

                let params = if params_str.is_empty() {
                    Vec::new()
                } else {
                    params_str
                        .split(';')
                        .flat_map(|x| x.parse::<usize>())
                        .collect()
                };

                let cb = *params.get(0)?;
                let cx = *params.get(1)?;
                let cy = *params.get(2)?;

                let button = match cb {
                    0 => MouseButton::Left,
                    2 => MouseButton::Right,
                    _ => return None,
                };

                match input.next_char_in_buf()? {
                    b'm' => Key::Mouseup(button, cx, cy),
                    b'M' => Key::Mousedown(button, cx, cy),
                    _ => return None,
                }
            }
            b'A' => Key::ArrowUp,
            b'B' => Key::ArrowDown,
            b'C' => Key::ArrowRight,
            b'D' => Key::ArrowLeft,
            _ => return None,
        },
        _ => return None,
    };

    Some(key)
}

pub struct KeyInput {
    buf: [u8; 16],
    buf_position: usize,
    buf_size: usize,
    source: Box<dyn Read>,
}

impl KeyInput {
    pub fn new(source: impl Read + 'static) -> Self {
        KeyInput {
            buf: [0; 16],
            buf_position: 0,
            buf_size: 0,
            source: Box::new(source),
        }
    }

    fn next_char_in_buf(&mut self) -> Option<u8> {
        if self.buf_size > self.buf_position {
            self.buf_position += 1;
            Some(self.buf[self.buf_position - 1])
        } else {
            None
        }
    }

    fn next_char_in_buf_if<F: Fn(u8) -> bool>(&mut self, f: F) -> Option<u8> {
        let head = *self.buf.get(self.buf_position)?;
        if f(head) {
            self.buf_position += 1;
            Some(head)
        } else {
            None
        }
    }

    fn next_char(&mut self) -> std::io::Result<u8> {
        loop {
            if let Some(next) = self.next_char_in_buf() {
                return Ok(next);
            }

            let size = self.source.read(&mut self.buf)?;
            self.buf_size = size;
            self.buf_position = 0;
        }
    }

    fn flush_buf(&mut self) {
        self.buf_size = 0;
    }

    fn skip(&mut self, steps: usize) {
        self.buf_position += steps;
    }

    pub fn get_key(&mut self) -> std::io::Result<Key> {
        loop {
            if let Some(key) = process_input(self)? {
                return Ok(key);
            }
        }
    }
}
