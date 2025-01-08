use crate::synchronizer::{self, Synchronizer};
use crate::unested::UNested;
use crate::update;
use crate::ustack::{UStack, UStackUpdate};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use update::Updatable;

pub struct SStack<T>
where
    T: Updatable + Clone + Serialize,
    <T as Updatable>::Update: Serialize,
{
    syn: Synchronizer<UStack<T>>,
}

impl<T> SStack<T>
where
    T: Updatable + Clone + Serialize + for<'de> Deserialize<'de> + Send + 'static,
    <T as Updatable>::Update: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    pub fn new(port: u16, group: u32) -> synchronizer::Result<Self> {
        let syn = Synchronizer::new(port, group)?;
        Ok(SStack { syn })
    }

    pub fn push(&mut self, value: T) -> synchronizer::Result<()> {
        self.syn.publish_update(UStackUpdate::Push(value))
    }

    pub fn pop(&mut self) -> synchronizer::Result<()> {
        self.syn.publish_update(UStackUpdate::Pop)
    }

    pub fn top(&self) -> Option<T> {
        self.syn.get_lock().top()
    }

    pub fn get_lock(&self) -> std::sync::MutexGuard<'_, UStack<T>> {
        self.syn.get_lock()
    }

    pub fn get_mut(
        &mut self,
    ) -> UNested<
        T,
        synchronizer::Result<()>,
        impl FnOnce(T::Update) -> synchronizer::Result<()> + use<'_, T>,
    > {
        UNested {
            apply_outer: move |update| self.syn.publish_update(UStackUpdate::Nested(update)),
            inner_type: PhantomData,
        }
    }
}
