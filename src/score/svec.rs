use crate::communication::synchronizer::{self, Synchronizer};
use crate::ucore::unested::UNested;
use crate::ucore::updateable;
use crate::ucore::uvec::{UVec, UVecUpdate};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use updateable::Updatable;

pub struct SVec<T>
where
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    syn: Synchronizer<UVec<T>>,
}

impl<T> SVec<T>
where
    T: Updatable + Clone + Serialize + for<'de> Deserialize<'de> + Send + 'static,
    <T as Updatable>::Update: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    pub fn new(port: u16, group: u32) -> synchronizer::Result<Self> {
        let syn = Synchronizer::new(port, group)?;
        Ok(SVec { syn })
    }

    pub fn clear(&mut self) -> synchronizer::Result<()> {
        self.syn.publish_update(UVecUpdate::Clear)
    }

    pub fn insert(&mut self, index: usize, value: T) -> synchronizer::Result<()> {
        self.syn.publish_update(UVecUpdate::Insert(index, value))
    }

    pub fn remove(&mut self, index: usize) -> synchronizer::Result<()> {
        self.syn.publish_update(UVecUpdate::Remove(index))
    }

    pub fn push(&mut self, value: T) -> synchronizer::Result<()> {
        self.syn.publish_update(UVecUpdate::Push(value))
    }

    pub fn pop(&mut self) -> synchronizer::Result<()> {
        self.syn.publish_update(UVecUpdate::Pop)
    }

    pub fn get(&self, index: usize) -> Option<T> {
        self.syn.get_lock().get(index)
    }

    pub fn get_lock(&self) -> std::sync::MutexGuard<'_, UVec<T>> {
        self.syn.get_lock()
    }

    pub fn get_mut(
        &mut self,
        index: usize,
    ) -> UNested<T, synchronizer::Result<()>, impl FnOnce(T::Update) -> synchronizer::Result<()> + '_>
    {
        UNested {
            apply_outer: move |update| self.syn.publish_update(UVecUpdate::Nested(index, update)),
            inner_type: PhantomData,
        }
    }
}
