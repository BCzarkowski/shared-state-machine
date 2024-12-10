use crate::update::{self};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use update::Update;

pub struct UMap<K: Eq + Hash, T> {
    map: HashMap<K, T>,
}

pub enum UMapUpdate<K: Eq + Hash, T> {
    Insert(K, T),
    Remove(K),
}

pub struct RecursiveUMapUpdate<K: Eq + Hash, T, U: Update<T> + ?Sized> {
    key: K,
    nested_update: Box<U>,
    phantom: PhantomData<T>,
}

impl<K: Eq + Hash, T> Update<UMap<K, T>> for UMapUpdate<K, T> {
    fn apply_update(self, item: &mut UMap<K, T>) {
        match self {
            UMapUpdate::Insert(key, value) => item.map.insert(key, value),
            UMapUpdate::Remove(key) => item.map.remove(&key),
        };
        ()
    }
}

pub trait ByKeyExtractor<K> {
    type Extract;
    fn get_mut(&mut self, key: &K) -> Option<&mut Self::Extract>;
}

impl<K: Eq + Hash, T> ByKeyExtractor<K> for UMap<K, T> {
    type Extract = T;
    fn get_mut(&mut self, key: &K) -> Option<&mut Self::Extract> {
        self.map.get_mut(key)
    }
}

impl<A: Eq + Hash, B, K: Eq + Hash, U: Update<UMap<A, B>>> Update<UMap<A, B>>
    for RecursiveUMapUpdate<K, UMap<A, B>, U>
{
    fn apply_update(self, item: &mut UMap<A, B>) {
        self.nested_update
            .apply_update(item.map.get_mut(&self.key).unwrap());
        ()
    }
}

// impl Update<UMap<String, UMap<String, i32>>>
//     for RecursiveUMapUpdate<String, UMap<String, i32>, dyn Update<UMap<String, i32>>>
// {
//     fn apply_update(self, item: &mut UMap<String, UMap<String, i32>>) {}
// }

impl<K: Eq + Hash, T> UMap<K, T> {
    pub fn new() -> Self {
        UMap {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: T) -> UMapUpdate<K, T> {
        UMapUpdate::Insert(key, value)
    }

    pub fn remove(&mut self, key: K) -> UMapUpdate<K, T> {
        UMapUpdate::Remove(key)
    }

    pub fn get(&self, key: &K) -> Option<&T> {
        self.map.get(key)
    }

    pub fn create_recursive(
        &self,
        key: K,
        update: impl Update<T>,
    ) -> RecursiveUMapUpdate<K, T, impl Update<T>> {
        RecursiveUMapUpdate {
            key,
            nested_update: Box::new(update),
            phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_operations() {
        let mut umap: UMap<String, i32> = UMap::new();
        let foo = String::from("foo");
        let bar = String::from("bar");
        let insert_5 = umap.insert(foo.clone(), 5);
        let insert_7 = umap.insert(bar.clone(), 7);
        let remove_5 = umap.remove(foo.clone());
        let remove_7 = umap.remove(bar.clone());

        assert_eq!(umap.get(&foo), None);
        assert_eq!(umap.get(&bar), None);
        insert_5.apply_update(&mut umap);
        assert_eq!(umap.get(&foo), Some(&5));
        assert_eq!(umap.get(&bar), None);
        insert_7.apply_update(&mut umap);
        assert_eq!(umap.get(&foo), Some(&5));
        assert_eq!(umap.get(&bar), Some(&7));
        remove_5.apply_update(&mut umap);
        assert_eq!(umap.get(&foo), None);
        assert_eq!(umap.get(&bar), Some(&7));
        remove_7.apply_update(&mut umap);
        assert_eq!(umap.get(&foo), None);
        assert_eq!(umap.get(&bar), None);
    }

    #[test]
    fn recursive_operations() {
        let mut umap: UMap<String, UMap<String, i32>> = UMap::new();
        let foo = String::from("foo");
        let bar = String::from("bar");
        umap.insert(foo.clone(), UMap::new())
            .apply_update(&mut umap);
        let item = umap.get(&foo).unwrap();
        let item_update = item.insert(bar.clone(), 32);
        let whole_update = umap.create_recursive(foo.clone(), item_update);
        whole_update.apply_update(&mut umap);
        assert_eq!(umap.get(&foo).unwrap().get(&bar).unwrap(), &32);
    }
}
