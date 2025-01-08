use shared_state_machine::umessage::UMessage;
use shared_state_machine::update::Updatable;
use shared_state_machine::ustack::{UStack, UStackUpdate};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_operations() {
        let mut ustack: UStack<i32> = UStack::new();
        let push_5 = ustack.push(5);

        let umessage = UMessage::new(0, 0, &push_5).unwrap();
        let message_update = umessage.get_update::<UStackUpdate<i32>>().unwrap();

        ustack.apply_update(message_update);
        assert_eq!(ustack.top().unwrap(), 5);
    }

    #[test]
    fn recursive_operations() {
        let mut ustack: UStack<UStack<i32>> = UStack::new();
        ustack.apply_update(ustack.push(UStack::new()));

        let recursive_update = ustack.top_mut().push(5);
        let umessage = UMessage::new(0, 0, &recursive_update).unwrap();
        let message_update = umessage.get_update::<UStackUpdate<UStack<i32>>>().unwrap();

        ustack.apply_update(message_update);
        assert_eq!(ustack.top().unwrap().top().unwrap(), 5);
    }
}
