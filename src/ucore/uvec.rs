use crate::ucore::unested::UNested;
use crate::ucore::updateable;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use updateable::Updatable;

#[derive(Clone, Serialize, Deserialize)]
pub struct UVec<T: Updatable>
where
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    vec: Vec<T>,
}

#[derive(Serialize, Deserialize)]
pub enum UVecUpdate<T>
where
    T: Updatable,
{
    Clear,
    Insert(usize, T),
    Pop,
    Push(T),
    Remove(usize),
    Nested(usize, T::Update),
}

impl<T> Updatable for UVec<T>
where
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    type Update = UVecUpdate<T>;

    fn apply_update(&mut self, update: Self::Update) {
        match update {
            UVecUpdate::Clear => {
                self.vec.clear();
            }
            UVecUpdate::Insert(index, value) => {
                self.vec.insert(index, value);
            }
            UVecUpdate::Remove(index) => {
                self.vec.remove(index);
            }
            UVecUpdate::Push(value) => {
                self.vec.push(value);
            }
            UVecUpdate::Pop => {
                self.vec.pop();
            }
            UVecUpdate::Nested(index, nested_update) => {
                self.vec.get_mut(index).unwrap().apply_update(nested_update);
            }
        }
    }
}

impl<T> Default for UVec<T>
where
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> UVec<T>
where
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
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

    pub fn get(&self, index: usize) -> Option<T> {
        self.vec.get(index).cloned()
    }

    pub fn get_ref(&self, index: usize) -> Option<&T> {
        self.vec.get(index)
    }

    pub fn get_mut(
        &self,
        index: usize,
    ) -> UNested<T, UVecUpdate<T>, impl FnOnce(T::Update) -> UVecUpdate<T>> {
        UNested {
            apply_outer: move |update| UVecUpdate::Nested(index, update),
            inner_type: PhantomData,
        }
    }

    pub fn last(&self) -> Option<T> {
        self.vec.last().cloned()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}

impl<T, O, F> UNested<UVec<T>, O, F>
where
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
    F: FnOnce(UVecUpdate<T>) -> O,
{
    pub fn clear(self) -> O {
        (self.apply_outer)(UVecUpdate::Clear)
    }

    pub fn insert(self, index: usize, value: T) -> O {
        (self.apply_outer)(UVecUpdate::Insert(index, value))
    }

    pub fn remove(self, index: usize) -> O {
        (self.apply_outer)(UVecUpdate::Remove(index))
    }

    pub fn push(self, value: T) -> O {
        (self.apply_outer)(UVecUpdate::Push(value))
    }

    pub fn pop(self) -> O {
        (self.apply_outer)(UVecUpdate::Pop)
    }

    pub fn get_mut(self, index: usize) -> UNested<T, O, impl FnOnce(T::Update) -> O> {
        UNested {
            apply_outer: move |update| (self.apply_outer)(UVecUpdate::Nested(index, update)),
            inner_type: PhantomData,
        }
    }
}
