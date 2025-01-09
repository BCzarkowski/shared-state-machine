use std::marker::PhantomData;

use crate::ucore::updateable;
use updateable::Updatable;

pub struct UNested<T, O, F>
where
    T: Updatable,
    F: FnOnce(T::Update) -> O,
{
    pub apply_outer: F,
    pub inner_type: PhantomData<T>,
}
