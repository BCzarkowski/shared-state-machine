pub trait Updatable {
    type Update;

    fn apply_update(&mut self, update: Self::Update);
}

impl Updatable for i32 {
    type Update = ();
    fn apply_update(&mut self, _update: Self::Update) {}
}
