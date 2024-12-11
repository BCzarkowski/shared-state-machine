use crate::recursive_structure_wrapper::StructureWrapper;
use crate::update;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use update::Updatable;

pub struct UMap<K: Eq + Hash, T: Updatable> {
    map: HashMap<K, T>,
}

#[derive(Serialize, Deserialize)]
pub enum UMapUpdate<K: Eq + Hash, T: Updatable> {
    Insert(K, T),
    Remove(K),
    Nested(K, T::Update),
}

impl<K: Eq + Hash, T: Updatable> Updatable for UMap<K, T> {
    type Update = UMapUpdate<K, T>;

    fn apply_update(&mut self, update: Self::Update) {
        match update {
            UMapUpdate::Insert(key, value) => {
                self.map.insert(key, value);
                ()
            }
            UMapUpdate::Remove(key) => {
                self.map.remove(&key);
                ()
            }
            UMapUpdate::Nested(key, upd) => {
                self.map.get_mut(&key).unwrap().apply_update(upd);
                ()
            }
        }
    }
}

impl<K: Eq + Hash, T: Updatable> UMap<K, T> {
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

    pub fn get(&self, key: &K) -> Option<&T> {
        self.map.get(key)
    }

    pub fn get_wrapped(
        &self,
        key: K,
    ) -> StructureWrapper<
        T,
        UMapUpdate<K, T>,
        impl FnOnce(T::Update) -> UMapUpdate<K, T> + use<'_, K, T>,
    > {
        StructureWrapper {
            structure: self.get(&key).unwrap(),
            outside_wrapper: move |update| UMapUpdate::Nested(key, update),
        }
    }
}

impl<K: Eq + Hash, T: Updatable, O, F: FnOnce(UMapUpdate<K, T>) -> O>
    StructureWrapper<'_, UMap<K, T>, O, F>
{
    pub fn insert(self, key: K, value: T) -> O {
        (self.outside_wrapper)(self.structure.insert(key, value))
    }

    pub fn remove(self, key: K) -> O {
        (self.outside_wrapper)(self.structure.remove(key))
    }
}
