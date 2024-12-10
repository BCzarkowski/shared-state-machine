use crate::update;
use std::collections::HashMap;
use std::hash::Hash;
use update::Updatable;

pub struct UMap<K: Eq + Hash, T: Updatable> {
    map: HashMap<K, T>,
}

pub enum UMapUpdate<K: Eq + Hash, T: Updatable, U> {
    Insert(K, T),
    Remove(K),
    Nested(K, U),
}

impl<K: Eq + Hash, T: Updatable> Updatable for UMap<K, T> {
    type Update = UMapUpdate<K, T, T::Update>;

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

    pub fn insert(&self, key: K, value: T) -> UMapUpdate<K, T, T::Update> {
        UMapUpdate::Insert(key, value)
    }

    pub fn remove(&self, key: K) -> UMapUpdate<K, T, T::Update> {
        UMapUpdate::Remove(key)
    }

    pub fn get(&self, key: &K) -> Option<&T> {
        self.map.get(key)
    }

    pub fn create_recursive(&self, key: K, update: T::Update) -> UMapUpdate<K, T, T::Update> {
        UMapUpdate::Nested(key, update)
    }
}

// impl Updatable for String {
//     type Update = ();
//     fn apply_update(&mut self, update: Self::Update) {}
// }

impl Updatable for i32 {
    type Update = ();
    fn apply_update(&mut self, update: Self::Update) {}
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

    #[test]
    fn recursive_operations() {
        let mut umap: UMap<String, UMap<String, i32>> = UMap::new();
        let foo = String::from("foo");
        let bar = String::from("bar");

        umap.apply_update(umap.insert(foo.clone(), UMap::new()));
        let inner_update = umap.get(&foo).unwrap().insert(bar.clone(), 5);
        let recursive_update = umap.create_recursive(foo.clone(), inner_update);

        umap.apply_update(recursive_update);
        assert_eq!(umap.get(&foo).unwrap().get(&bar).unwrap(), &5);
    }
}
