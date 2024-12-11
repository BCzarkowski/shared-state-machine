use shared_state_machine::ustack::UStack;
use shared_state_machine::update::Updatable;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_operations() {
        let mut ustack: UStack<i32> = UStack::new();
        let push_5 = ustack.push(5);
        let pop = ustack.pop();

        ustack.apply_update(push_5);
        assert_eq!(*ustack.top(), 5);
        let push_7 = ustack.push(7);
        assert_eq!(*ustack.top(), 5);
        ustack.apply_update(push_7);
        assert_eq!(*ustack.top(), 7);
        ustack.apply_update(pop);
        assert_eq!(*ustack.top(), 5);
    }

    #[test]
    fn recursive_operations() {
        let mut ustack: UStack<UStack<i32>> = UStack::new();
        ustack.apply_update(ustack.push(UStack::new()));

        let inner_update = ustack.top_mut().push(5);
        let recursive_update = ustack.create_recursive(inner_update);

        ustack.apply_update(recursive_update);
        assert_eq!(*ustack.top().top(), 5);
    }
}
