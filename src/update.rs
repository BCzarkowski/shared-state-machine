use std::str::Bytes;

pub trait Updatable {
    type Update;

    fn apply_update(&mut self, update: Self::Update);
}

macro_rules !impl_updatable {
    ($($t:ty),*) => {
        $(
            impl Updatable for $t {
                type Update = ();
                fn apply_update(&mut self, _update: Self::Update) {}
            }
        )*
    };
}

impl_updatable!(
    bool,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
    char,
    str,
    String,
    Bytes<'_>
);
