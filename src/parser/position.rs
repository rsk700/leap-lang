use std::cmp::max;

// todo: path only in struct/enum level
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Position {
    pub start: usize,  // index of start utf8 character
    pub length: usize, // length in utf8 characters
}

impl Position {
    pub fn new(start: usize, length: usize) -> Self {
        Position { start, length }
    }

    pub fn end(&self) -> usize {
        self.start + self.length
    }

    pub fn extend(&self, pos: &Self) -> Self {
        let length = max(self.length, pos.start + pos.length - self.start);
        Self {
            start: self.start,
            length,
        }
    }
}
