use crate::update;
use update::Updatable;

pub struct UStack<T> {
    stack: Vec<T>,
}

pub enum UStackUpdate<T> {
    Push(T),
    Pop,
}

impl<T> Updatable for UStack<T> {
    type Update = UStackUpdate<T>;

    fn apply_update(&mut self, update: Self::Update) {
        match update {
            UStackUpdate::Push(value) => self.stack.push(value),
            UStackUpdate::Pop => {
                if self.stack.is_empty() {
                    panic!("Update tried to pop from an empty stack!");
                } else {
                    self.stack.pop();
                }
            }
        }
    }
}

impl<T> UStack<T> {
    pub fn new() -> Self {
        UStack { stack: Vec::new() }
    }

    pub fn push(&mut self, value: T) -> UStackUpdate<T> {
        UStackUpdate::Push(value)
    }

    pub fn pop(&mut self) -> UStackUpdate<T> {
        UStackUpdate::Pop
    }

    pub fn top(&self) -> &T {
        self.stack.last().unwrap()
    }
}
