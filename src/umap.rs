use crate::recursive_structure_wrapper::StructureWrapper;
use crate::update;
use std::collections::HashMap;
use std::hash::Hash;
use update::Updatable;

pub struct UMap<K: Eq + Hash, T: Updatable> {
    map: HashMap<K, T>,
}

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
        umap.apply_update(insert_5);
        assert_eq!(umap.get(&foo), Some(&5));
        assert_eq!(umap.get(&bar), None);
        umap.apply_update(insert_7);
        assert_eq!(umap.get(&foo), Some(&5));
        assert_eq!(umap.get(&bar), Some(&7));
        umap.apply_update(remove_5);
        assert_eq!(umap.get(&foo), None);
        assert_eq!(umap.get(&bar), Some(&7));
        umap.apply_update(remove_7);
        assert_eq!(umap.get(&foo), None);
        assert_eq!(umap.get(&bar), None);
    }

    #[test]
    fn recursive_operations() {
        let mut umap: UMap<String, UMap<String, i32>> = UMap::new();
        let foo = String::from("foo");
        let bar = String::from("bar");
        umap.apply_update(umap.insert(foo.clone(), UMap::new()));

        umap.apply_update(umap.get_wrapped(foo.clone()).insert(bar.clone(), 5));
        assert_eq!(umap.get(&foo).unwrap().get(&bar).unwrap(), &5);
    }
}
