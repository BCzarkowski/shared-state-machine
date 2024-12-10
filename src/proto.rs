pub trait UpperTrait {
    fn recursive_call(&self);
}

pub enum A<T: UpperTrait> {
    Case1,
    Case2(T),
}

pub enum B<T: UpperTrait> {
    Case1,
    Case2(T),
}

struct C {}

impl UpperTrait for C {
    fn recursive_call(&self) {}
}

impl<T: UpperTrait> UpperTrait for A<T> {
    fn recursive_call(&self) {
        match self {
            A::Case1 => {
                println!("Arrived at Case1 of A");
            }
            A::Case2(nested_call) => {
                println!("Calling in Case2 for a nested type in A");
                nested_call.recursive_call();
            }
        }
    }
}

impl<T: UpperTrait> UpperTrait for B<T> {
    fn recursive_call(&self) {
        match self {
            B::Case1 => {
                println!("Arrived at Case1 of B");
            }
            B::Case2(nested_call) => {
                println!("Calling in Case2 for a nested type in B");
                nested_call.recursive_call();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_this_shit_even_work() {
        let boy = A::Case2(B::Case2(B::Case2(A::Case2(A::Case2(B::<C>::Case1)))));
        boy.recursive_call();
    }
}
