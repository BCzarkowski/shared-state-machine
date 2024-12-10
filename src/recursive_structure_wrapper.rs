use crate::update::Updatable;

pub struct StructureWrapper<'a, T, O, F>
where
    T: Updatable,
    F: FnOnce(T::Update) -> O,
{
    pub structure: &'a T,
    pub outside_wrapper: F,
}
