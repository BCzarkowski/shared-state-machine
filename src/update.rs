pub trait Updatable {
    type Update;

    fn apply_update(&mut self, update: Update);
}
