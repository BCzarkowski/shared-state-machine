use crate::update;
use serde::{Deserialize, Serialize};
use update::Updatable;

pub struct UVec<T: Updatable> {
    vec: Vec<T>,
}

#[derive(Serialize, Deserialize)]
pub enum UVecUpdate<T: Updatable> {
    Clear,
    Insert(usize, T),
    Pop,
    Push(T),
    Remove(usize),
    Nested(usize, T::Update),
}

impl<T: Updatable> Updatable for UVec<T> {
    type Update = UVecUpdate<T>;

    fn apply_update(&mut self, update: Self::Update) {
        match update {
            UVecUpdate::Clear => {
                self.vec.clear();
                ()
            }
            UVecUpdate::Insert(index, value) => {
                self.vec.insert(index, value);
                ()
            }
            UVecUpdate::Remove(index) => {
                self.vec.remove(index);
                ()
            }
            UVecUpdate::Push(value) => {
                self.vec.push(value);
                ()
            }
            UVecUpdate::Pop => {
                if self.vec.is_empty() {
                    panic!("Update tried to pop from an empty vector!");
                } else {
                    self.vec.pop();
                }
            }
            UVecUpdate::Nested(index, nested_update) => {
                if index >= self.vec.len() {
                    panic!("Index is greater then vector length!");
                } else {
                    self.vec[index].apply_update(nested_update);
                }
            }
        }
    }
}

impl<T: Updatable> UVec<T> {
    pub fn new() -> Self {
        UVec { vec: Vec::new() }
    }

    pub fn clear(&self) -> UVecUpdate<T> {
        UVecUpdate::Clear
    }

    pub fn insert(&self, index: usize, value: T) -> UVecUpdate<T> {
        UVecUpdate::Insert(index, value)
    }

    pub fn remove(&self, index: usize) -> UVecUpdate<T> {
        UVecUpdate::Remove(index)
    }

    pub fn push(&self, value: T) -> UVecUpdate<T> {
        UVecUpdate::Push(value)
    }

    pub fn pop(&self) -> UVecUpdate<T> {
        UVecUpdate::Pop
    }

    pub fn get(&self, index: usize) -> &T {
        self.vec.get(index).unwrap()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        self.vec.get_mut(index).unwrap()
    }

    pub fn last(&self) -> &T {
        self.vec.last().unwrap()
    }

    pub fn last_mut(&mut self) -> &mut T {
        self.vec.last_mut().unwrap()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn create_recursive(&self, index: usize, update: T::Update) -> UVecUpdate<T> {
        UVecUpdate::Nested(index, update)
    }
}
