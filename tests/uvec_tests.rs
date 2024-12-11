use shared_state_machine::update::Updatable;
use shared_state_machine::uvec::UVec;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_operations() {
        let mut uvec: UVec<i32> = UVec::new();
        let push_5 = uvec.push(5);
        let insert_1_0 = uvec.insert(1, 0);
        let pop = uvec.pop();
        let remove_0 = uvec.remove(0);
        let clear = uvec.clear();

        uvec.apply_update(push_5);
        assert_eq!(*uvec.last(), 5);
        assert_eq!(*uvec.get(0), 5);
        let push_7 = uvec.push(7);
        assert_eq!(*uvec.last(), 5);
        uvec.apply_update(push_7);
        assert_eq!(*uvec.last(), 7);

        uvec.apply_update(insert_1_0);
        assert_eq!(*uvec.get(0), 5);
        assert_eq!(*uvec.get(1), 0);
        assert_eq!(*uvec.get(2), 7);

        uvec.apply_update(pop);
        assert_eq!(*uvec.last(), 0);

        uvec.apply_update(remove_0);
        assert_eq!(*uvec.get(0), 0);

        uvec.apply_update(clear);
        assert_eq!(uvec.is_empty(), true);
    }

    #[test]
    fn recursive_operations() {
        let mut uvec: UVec<UVec<i32>> = UVec::new();
        uvec.apply_update(uvec.push(UVec::new()));
        uvec.apply_update(uvec.insert(1, UVec::new()));

        let inner_update = uvec.get_mut(0).push(5);
        let recursive_update = uvec.create_recursive(0, inner_update);

        uvec.apply_update(recursive_update);
        assert_eq!(*uvec.get(0).last(), 5);
        assert_eq!(uvec.get(1).is_empty(), true);
    }
}
