use shared_state_machine::uvec::UVec;
use shared_state_machine::update::Updatable;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_operations() {
        let mut uvec: UVec<i32> = UVec::new();
        let push_5 = uvec.push(5);
        let pop = uvec.pop();

        uvec.apply_update(push_5);
        assert_eq!(*uvec.top(), 5);
        let push_7 = uvec.push(7);
        assert_eq!(*uvec.top(), 5);
        uvec.apply_update(push_7);
        assert_eq!(*uvec.top(), 7);
        uvec.apply_update(pop);
        assert_eq!(*uvec.top(), 5); 
    }

    #[test]
    fn recursive_operations() {
        let mut uvec: UVec<UVec<i32>> = UVec::new();
        uvec.apply_update(uvec.push(UVec::new()));

        let inner_update = uvec.top_mut().push(5);
        let recursive_update = uvec.create_recursive(inner_update);

        uvec.apply_update(recursive_update);
        assert_eq!(*uvec.top().top(), 5);
    }   
}