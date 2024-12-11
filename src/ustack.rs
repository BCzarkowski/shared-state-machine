use crate::update;
use serde::{Deserialize, Serialize};
use update::Updatable;

pub struct UStack<T: Updatable> {
    stack: Vec<T>,
}

#[derive(Serialize, Deserialize)]
pub enum UStackUpdate<T: Updatable> {
    Push(T),
    Pop,
    Nested(T::Update),
}

impl<T: Updatable> Updatable for UStack<T> {
    type Update = UStackUpdate<T>;

    fn apply_update(&mut self, update: Self::Update) {
        match update {
            UStackUpdate::Push(value) => {
                self.stack.push(value);
                ()
            }
            UStackUpdate::Pop => {
                if self.stack.is_empty() {
                    panic!("Update tried to pop from an empty stack!");
                } else {
                    self.stack.pop();
                }
            }
            UStackUpdate::Nested(nested_update) => {
                if self.stack.is_empty() {
                    panic!("Nested update on empty stack!");
                } else {
                    self.stack.last_mut().unwrap().apply_update(nested_update);
                }
            }
        }
    }
}

impl<T: Updatable> UStack<T> {
    pub fn new() -> Self {
        UStack { stack: Vec::new() }
    }

    pub fn push(&self, value: T) -> UStackUpdate<T> {
        UStackUpdate::Push(value)
    }

    pub fn pop(&self) -> UStackUpdate<T> {
        UStackUpdate::Pop
    }

    pub fn top(&self) -> &T {
        self.stack.last().unwrap()
    }

    pub fn top_mut(&mut self) -> &mut T {
        self.stack.last_mut().unwrap()
    }

    pub fn create_recursive(&self, update: T::Update) -> UStackUpdate<T> {
        UStackUpdate::Nested(update)
    }
}
