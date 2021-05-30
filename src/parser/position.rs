// todo: add length of token
#[derive(PartialEq, Debug)]
pub struct Position<T>(pub usize, pub T);

impl<T> Position<T> {
    pub fn replaced<U>(&self, x: U) -> Position<U> {
        Position(self.0, x)
    }
}