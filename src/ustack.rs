use crate::update;
use update::Updatable;

pub struct UStack<T: Updatable> {
    stack: Vec<T>,
}

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
            },
            UStackUpdate::Pop => {
                if self.stack.is_empty() {
                    panic!("Update tried to pop from an empty stack!");
                } else {
                    self.stack.pop();
                }
            },
            UStackUpdate::Nested(nested_update) => {
                if self.stack.is_empty() {
                    panic!("Nested update on empty stack!");
                }
                else {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_operations() {
        let mut ustack: UStack<i32> = UStack::new();
        let push_5 = ustack.push(5);
        let pop = ustack.pop();

        ustack.apply_update(push_5);
        assert_eq!(*ustack.top(), 5);
        let push_7 = ustack.push(7);
        assert_eq!(*ustack.top(), 5);
        ustack.apply_update(push_7);
        assert_eq!(*ustack.top(), 7);
        ustack.apply_update(pop);
        assert_eq!(*ustack.top(), 5);
    }

    #[test]
    fn recursive_operations() {
        let mut ustack: UStack<UStack<i32>> = UStack::new();
        ustack.apply_update(ustack.push(UStack::new()));

        let inner_update = ustack.top_mut().push(5);
        let recursive_update = ustack.create_recursive(inner_update);
        
        ustack.apply_update(recursive_update);
        assert_eq!(*ustack.top().top(), 5);
    }
}
