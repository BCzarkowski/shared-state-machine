pub trait Update<T> {
    fn apply_update(self, item: &mut T);
}
