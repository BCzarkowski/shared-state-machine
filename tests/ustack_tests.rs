use shared_state_machine::updateable::updateable::Updatable;
use shared_state_machine::updateable::ustack::UStack;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_operations() {
        let mut ustack: UStack<i32> = UStack::new();
        let push_5 = ustack.push(5);
        let pop = ustack.pop();

        ustack.apply_update(push_5);
        assert_eq!(ustack.top().unwrap(), 5);
        let push_7 = ustack.push(7);
        assert_eq!(ustack.top().unwrap(), 5);
        ustack.apply_update(push_7);
        assert_eq!(ustack.top().unwrap(), 7);
        ustack.apply_update(pop);
        assert_eq!(ustack.top().unwrap(), 5);
    }

    #[test]
    fn recursive_operations() {
        let mut ustack: UStack<UStack<UStack<UStack<i32>>>> = UStack::new();

        ustack.apply_update(ustack.push(UStack::new()));
        ustack.apply_update(ustack.top_mut().push(UStack::new()));
        ustack.apply_update(ustack.top_mut().top_mut().push(UStack::new()));
        ustack.apply_update(ustack.top_mut().top_mut().top_mut().push(5));

        assert_eq!(
            ustack
                .top()
                .unwrap()
                .top()
                .unwrap()
                .top()
                .unwrap()
                .top()
                .unwrap(),
            5
        )
    }
}
