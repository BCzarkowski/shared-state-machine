use crate::synchronizer::{self, Synchronizer};
use crate::umap::{UMap, UMapUpdate};
use crate::unested::UNested;
use crate::update;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::MutexGuard;
use update::Updatable;

pub struct SMap<K, T>
where
    K: Eq + Hash + Clone + Serialize,
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    syn: Synchronizer<UMap<K, T>>,
}

impl<K, T> SMap<K, T>
where
    K: Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de> + Send + 'static,
    T: Updatable + Clone + Serialize + for<'de> Deserialize<'de> + Send + 'static,
    <T as Updatable>::Update: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    pub fn new(port: u16, group: u32) -> synchronizer::Result<Self> {
        let syn = Synchronizer::new(port, group)?;
        Ok(SMap { syn })
    }

    pub fn insert(&mut self, key: K, value: T) -> () {
        self.syn.publish_update(UMapUpdate::Insert(key, value))
    }

    pub fn remove(&mut self, key: K) -> () {
        self.syn.publish_update(UMapUpdate::Remove(key))
    }

    pub fn get(&self, key: &K) -> Option<T> {
        self.syn.get_lock().get(key).clone()
    }

    pub fn get_mut(
        &mut self,
        key: K,
    ) -> UNested<T, (), impl FnOnce(T::Update) -> () + use<'_, K, T>> {
        UNested {
            apply_outer: |update| self.syn.publish_update(UMapUpdate::Nested(key, update)),
            inner_type: PhantomData,
        }
    }
}
