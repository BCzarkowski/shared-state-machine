use shared_state_machine::ucore::umap::UMap;
use shared_state_machine::ucore::updateable::Updatable;

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
        assert_eq!(umap.get(&foo), Some(5));
        assert_eq!(umap.get(&bar), None);
        umap.apply_update(insert_7);
        assert_eq!(umap.get(&foo), Some(5));
        assert_eq!(umap.get(&bar), Some(7));
        umap.apply_update(remove_5);
        assert_eq!(umap.get(&foo), None);
        assert_eq!(umap.get(&bar), Some(7));
        umap.apply_update(remove_7);
        assert_eq!(umap.get(&foo), None);
        assert_eq!(umap.get(&bar), None);
    }

    #[test]
    fn recursive_operations() {
        let mut umap: UMap<String, UMap<i32, UMap<String, UMap<String, i32>>>> = UMap::new();
        let foo = String::from("foo");
        let bar = String::from("bar");
        let val = 7;
        umap.apply_update(umap.insert(foo.clone(), UMap::new()));
        umap.apply_update(umap.get_mut(foo.clone()).insert(val.clone(), UMap::new()));
        umap.apply_update(
            umap.get_mut(foo.clone())
                .get_mut(val.clone())
                .insert(bar.clone(), UMap::new()),
        );
        umap.apply_update(
            umap.get_mut(foo.clone())
                .get_mut(val.clone())
                .get_mut(bar.clone())
                .insert(foo.clone(), 5),
        );

        assert_eq!(
            umap.get(&foo)
                .unwrap()
                .get(&val)
                .unwrap()
                .get(&bar)
                .unwrap()
                .get(&foo)
                .unwrap(),
            5
        );
    }
}
