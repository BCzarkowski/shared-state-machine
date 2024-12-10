use crate::update;
use std::collections::HashMap;
use std::hash::Hash;
use update::Updatable;

pub struct UMap<K: Eq + Hash, T> {
    map: HashMap<K, T>,
}

pub enum UMapUpdate<K: Eq + Hash, T, U> {
    Insert(K, T),
    Remove(K),
    NestedUpdate(K, U),
}

impl<K: Eq + Hash, T, U> Updatable for UMap<K, T> {
    type Update = UMapUpdate<K, T, U>;

    fn apply_update(&mut self, update: Self::Update) {
        match update {
            UMapUpdate::Insert(key, value) => self.map.insert(key, value),
            UMapUpdate::Remove(key) => self.map.remove(&key),
            UMapUpdate::NestedUpdate(key, nested_update) => {
                self.map.get_mut(&key).unwrap().apply_update(nested_update)
            }
        };
        ()
    }
}

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_operations() {
        let mut umap: UMap<String, i32> = UMap::new();
        let insert_5 = umap.insert(String::from("foo"), 5);
        let insert_7 = umap.insert(String::from("bar"), 7);
        let remove_5 = umap.remove(String::from("foo"));
        let remove_7 = umap.remove(String::from("bar"));

        assert_eq!(umap.get(&String::from("foo")), None);
        assert_eq!(umap.get(&String::from("bar")), None);
        umap.apply_update(insert_5);
        assert_eq!(umap.get(&String::from("foo")), Some(&5));
        assert_eq!(umap.get(&String::from("bar")), None);
        umap.apply_update(insert_7);
        assert_eq!(umap.get(&String::from("foo")), Some(&5));
        assert_eq!(umap.get(&String::from("bar")), Some(&7));
        umap.apply_update(remove_5);
        assert_eq!(umap.get(&String::from("foo")), None);
        assert_eq!(umap.get(&String::from("bar")), Some(&7));
        umap.apply_update(remove_7);
        assert_eq!(umap.get(&String::from("foo")), None);
        assert_eq!(umap.get(&String::from("bar")), None);
    }
}
