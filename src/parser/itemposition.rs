use super::position::Position;

#[derive(PartialEq, Debug)]
pub struct ItemPosition<I>(pub Position, pub I);

impl<I> ItemPosition<I> {
    pub fn new(start: usize, length: usize, item: I) -> ItemPosition<I> {
        ItemPosition(Position::new(start, length), item)
    }

    pub fn replace<U>(&self, x: U) -> ItemPosition<U> {
        ItemPosition(self.0, x)
    }
}
