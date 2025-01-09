use crate::communication::synchronizer::{self, Synchronizer};
use crate::ucore::umap::{UMap, UMapUpdate};
use crate::ucore::unested::UNested;
use crate::ucore::updateable;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use std::marker::PhantomData;
use updateable::Updatable;

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

    pub fn insert(&mut self, key: K, value: T) -> synchronizer::Result<()> {
        self.syn.publish_update(UMapUpdate::Insert(key, value))
    }

    pub fn remove(&mut self, key: K) -> synchronizer::Result<()> {
        self.syn.publish_update(UMapUpdate::Remove(key))
    }

    pub fn get(&self, key: &K) -> Option<T> {
        self.syn.get_lock().get(key)
    }

    pub fn get_lock(&self) -> std::sync::MutexGuard<'_, UMap<K, T>> {
        self.syn.get_lock()
    }

    pub fn get_mut(
        &mut self,
        key: K,
    ) -> UNested<T, synchronizer::Result<()>, impl FnOnce(T::Update) -> synchronizer::Result<()> + '_>
    {
        UNested {
            apply_outer: move |update| self.syn.publish_update(UMapUpdate::Nested(key, update)),
            inner_type: PhantomData,
        }
    }
}
