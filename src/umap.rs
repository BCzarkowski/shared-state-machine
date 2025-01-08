use crate::unested::UNested;
use crate::update;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use update::Updatable;

#[derive(Clone, Serialize, Deserialize)]
pub struct UMap<K, T>
where
    K: Eq + Hash + Clone + Serialize,
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    map: HashMap<K, T>,
}

#[derive(Serialize, Deserialize)]
pub enum UMapUpdate<K, T>
where
    K: Eq + Hash,
    T: Updatable,
{
    Insert(K, T),
    Remove(K),
    Nested(K, T::Update),
}

impl<K, T> Updatable for UMap<K, T>
where
    K: Eq + Hash + Clone + Serialize,
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    type Update = UMapUpdate<K, T>;

    fn apply_update(&mut self, update: Self::Update) {
        match update {
            UMapUpdate::Insert(key, value) => {
                self.map.insert(key, value);
            }
            UMapUpdate::Remove(key) => {
                self.map.remove(&key);
            }
            UMapUpdate::Nested(key, upd) => {
                self.map.get_mut(&key).unwrap().apply_update(upd);
            }
        }
    }
}

impl<K, T> Default for UMap<K, T>
where
    K: Eq + Hash + Clone + Serialize,
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, T> UMap<K, T>
where
    K: Eq + Hash + Clone + Serialize,
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    pub fn new() -> Self {
        UMap {
            map: HashMap::new(),
        }
    }

    pub fn insert(&self, key: K, value: T) -> UMapUpdate<K, T> {
        UMapUpdate::Insert(key, value)
    }

    pub fn remove(&self, key: K) -> UMapUpdate<K, T> {
        UMapUpdate::Remove(key)
    }

    pub fn get(&self, key: &K) -> Option<T> {
        self.map.get(key).map(|x: &T| x.clone())
    }

    pub fn get_ref(&self, key: &K) -> Option<&T> {
        self.map.get(key)
    }

    pub fn get_mut(
        &self,
        key: K,
    ) -> UNested<T, UMapUpdate<K, T>, impl FnOnce(T::Update) -> UMapUpdate<K, T>> {
        UNested {
            nested: self.get_ref(&key).unwrap(),
            apply_outer: move |update| UMapUpdate::Nested(key, update),
        }
    }
}

impl<'a, K, T, O, F> UNested<'a, UMap<K, T>, O, F>
where
    K: Eq + Hash + Clone + Serialize,
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
    F: FnOnce(UMapUpdate<K, T>) -> O,
{
    pub fn insert(self, key: K, value: T) -> O {
        (self.apply_outer)(self.nested.insert(key, value))
    }

    pub fn remove(self, key: K) -> O {
        (self.apply_outer)(self.nested.remove(key))
    }

    pub fn get_mut(self, key: K) -> UNested<'a, T, O, impl FnOnce(T::Update) -> O> {
        UNested {
            nested: self.nested.get_ref(&key).unwrap(),
            apply_outer: move |update| (self.apply_outer)(UMapUpdate::Nested(key, update)),
        }
    }
}
