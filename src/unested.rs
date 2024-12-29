use crate::update::Updatable;

pub struct UNested<'a, T, O, F>
where
    T: Updatable,
    F: FnOnce(T::Update) -> O,
{
    pub nested: &'a T,
    pub apply_outer: F,
}
