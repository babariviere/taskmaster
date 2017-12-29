//! Implementation of a basic parsing structure

/// Basic parser
pub struct Parser<'a> {
    buf: &'a [u8],
    idx: usize,
    len: usize,
}

impl<'a> Parser<'a> {
    /// Create a new parser
    pub fn new(buf: &'a str) -> Parser<'a> {
        Parser {
            buf: buf.as_bytes(),
            idx: 0,
            len: buf.len(),
        }
    }

    /// Check if it's end of file
    #[inline]
    pub fn is_eof(&self) -> bool {
        self.idx >= self.len
    }

    /// Get next char
    pub fn next_char(&self) -> Option<char> {
        if self.is_eof() {
            return None;
        }
        Some(self.buf[self.idx] as char)
    }

    /// Get next line
    pub fn next_line(&self) -> String {
        let mut idx = self.idx;
        let mut buf = String::new();
        while idx < self.len {
            if self.buf[idx] as char == '\n' {
                break;
            }
            buf.push(self.buf[idx] as char);
            idx += 1;
        }
        buf
    }

    /// Get while f is true
    pub fn get_while<F>(&self, f: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut idx = self.idx;
        let mut buf = String::new();
        while idx < self.len && f(self.buf[idx] as char) {
            buf.push(self.buf[idx] as char);
            idx += 1;
        }
        buf
    }

    /// Eat next char
    pub fn eat_char(&mut self) -> Option<char> {
        let c = self.next_char();
        if c.is_some() {
            self.idx += 1;
        }
        c
    }

    /// Eat next line
    pub fn eat_line(&mut self) -> String {
        let line = self.next_line();
        self.idx += line.len() + 1;
        line
    }

    /// Eat while
    pub fn eat_while<F>(&mut self, f: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let buf = self.get_while(f);
        self.idx += buf.len();
        buf
    }
}
