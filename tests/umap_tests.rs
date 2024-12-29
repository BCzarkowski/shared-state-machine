use shared_state_machine::umap::UMap;
use shared_state_machine::update::Updatable;

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

        umap.apply_update(umap.get_mut(foo.clone()).insert(bar.clone(), 5));
        assert_eq!(umap.get(&foo).unwrap().get(&bar).unwrap(), &5);
    }
}
