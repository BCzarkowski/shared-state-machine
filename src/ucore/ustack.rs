use crate::ucore::unested::UNested;
use crate::ucore::updateable;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use updateable::Updatable;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UStack<T>
where
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    stack: Vec<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum UStackUpdate<T>
where
    T: Updatable,
{
    Push(T),
    Pop,
    Nested(T::Update),
}

impl<T> Updatable for UStack<T>
where
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    type Update = UStackUpdate<T>;

    fn apply_update(&mut self, update: Self::Update) {
        match update {
            UStackUpdate::Push(value) => {
                self.stack.push(value);
            }
            UStackUpdate::Pop => {
                self.stack.pop();
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

impl<T> Default for UStack<T>
where
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> UStack<T>
where
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    pub fn new() -> Self {
        UStack { stack: Vec::new() }
    }

    pub fn push(&self, value: T) -> UStackUpdate<T> {
        UStackUpdate::Push(value)
    }

    pub fn pop(&self) -> UStackUpdate<T> {
        UStackUpdate::Pop
    }

    pub fn top(&self) -> Option<T> {
        self.stack.last().cloned()
    }

    pub fn top_ref(&self) -> Option<&T> {
        self.stack.last()
    }

    pub fn top_mut(
        &self,
    ) -> UNested<T, UStackUpdate<T>, impl FnOnce(T::Update) -> UStackUpdate<T>> {
        UNested {
            apply_outer: move |update| UStackUpdate::Nested(update),
            inner_type: PhantomData,
        }
    }
}

impl<T, O, F> UNested<UStack<T>, O, F>
where
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
    F: FnOnce(UStackUpdate<T>) -> O,
{
    pub fn push(self, value: T) -> O {
        (self.apply_outer)(UStackUpdate::Push(value))
    }

    pub fn pop(self) -> O {
        (self.apply_outer)(UStackUpdate::Pop)
    }

    pub fn top_mut(self) -> UNested<T, O, impl FnOnce(T::Update) -> O> {
        UNested {
            apply_outer: move |update| (self.apply_outer)(UStackUpdate::Nested(update)),
            inner_type: PhantomData,
        }
    }
}
