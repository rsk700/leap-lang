use std::{
    hash::{Hash, Hasher},
    marker::PhantomData,
};

#[derive(Debug)]
pub struct Handle<T>(u32, PhantomData<T>);

impl<T> Handle<T> {
    #[inline]
    pub fn new(id: u32) -> Self {
        Self(id, PhantomData)
    }

    #[inline]
    pub fn as_index(&self) -> usize {
        self.0 as usize
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self::new(self.0)
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
