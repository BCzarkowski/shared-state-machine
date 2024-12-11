use crate::update;
use update::Updatable;

pub struct UVec<T: Updatable> {
    vec: Vec<T>,
}

pub enum UVecUpdate<T: Updatable> {
    Push(T),
    Pop,
    Nested(T::Update),
}

impl<T: Updatable> Updatable for UVec<T> {
    type Update = UVecUpdate<T>;

    fn apply_update(&mut self, update: Self::Update) {
        match update {
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
            UVecUpdate::Nested(nested_update) => {
                if self.vec.is_empty() {
                    panic!("Nested update on empty vector!");
                } else {
                    self.vec.last_mut().unwrap().apply_update(nested_update);
                }
            }
        }
    }
}

impl<T: Updatable> UVec<T> {
    pub fn new() -> Self {
        UVec { vec: Vec::new() }
    }

    pub fn push(&self, value: T) -> UVecUpdate<T> {
        UVecUpdate::Push(value)
    }

    pub fn pop(&self) -> UVecUpdate<T> {
        UVecUpdate::Pop
    }

    pub fn top(&self) -> &T {
        self.vec.last().unwrap()
    }

    pub fn top_mut(&mut self) -> &mut T {
        self.vec.last_mut().unwrap()
    }

    pub fn create_recursive(&self, update: T::Update) -> UVecUpdate<T> {
        UVecUpdate::Nested(update)
    }
}