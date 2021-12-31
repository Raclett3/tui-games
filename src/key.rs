use std::io::Read;

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
}

fn process_input(input: &mut KeyInput) -> std::io::Result<Option<Key>> {
    let key = match input.next_char()? {
        9 => Key::Tab,
        13 => Key::Return,
        27 => return Ok(process_escape(input)),
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
        Some(b'[') => {
            let mut params_str = String::new();
            while let Some(next) = input.next_char_in_buf() {
                params_str.push(next as char);
            }
            let next = params_str.pop()?;
            let _params = if params_str.is_empty() {
                Vec::new()
            } else {
                params_str
                    .split(';')
                    .flat_map(|x| x.parse::<u64>())
                    .collect()
            };

            match next {
                'A' => Key::ArrowUp,
                'B' => Key::ArrowDown,
                'C' => Key::ArrowRight,
                'D' => Key::ArrowLeft,
                _ => return None,
            }
        }
        _ => {
            input.flush_buf();
            return None;
        }
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
