use shared_state_machine::update::Updatable;
use shared_state_machine::uvec::UVec;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_clients() {
        // let server: Server = Server::new(7878);

        // // How is the map initialized? Does one of the clients have to be an initial
        // // client or do we have some notion of a default constructor and we don't have
        // // any other ways of construction. This way we could implement some logic where
        // // the client default-constructs the object, reads updates from the server thus
        // // building up the state and then proceeds.

        // // Port 7878, structure_id = 1
        // let mut map1: SMap<String, i32> = SMap::new(7878, 1);
        // let mut map2: SMap<String, i32> = SMap::new(7878, 1);
        // let foo = String::from("foo");
        // let bar = String::from("bar");
        // let dog = String::from("dog");
        // map1.insert(foo.clone(), 1);
        // map1.insert(bar.clone(), 2);
        // map1.insert(dog.clone(), 3);

        // assert_eq!(map2.get(&foo), Some(&1));
        // assert_eq!(map2.get(&bar), Some(&2));
        // assert_eq!(map2.get(&dog), Some(&3));

        // map2.insert(foo.clone(), 9);
        // map2.insert(bar.clone(), 8);
        // map2.insert(dog.clone(), 7);

        // assert_eq!(map1.get(&foo), Some(&9));
        // assert_eq!(map1.get(&bar), Some(&8));
        // assert_eq!(map1.get(&dog), Some(&7));
    }

    #[test]
    fn nested_structure() {
        // let server: Server = Server::new(7878);

        // let mut map1: SMap<String, UMap<i32, i32>> = SMap::new(7878, 1);
        // let mut map2: SMap<String, UMap<i32, i32>> = SMap::new(7878, 1);

        // let foo = String::from("foo");
        // let bar = String::from("bar");
        // let dog = String::from("dog");

        // map1.get_mut(foo.clone()).insert(1, 5);
        // map1.get_mut(bar.clone()).insert(2, 6);
        // map1.get_mut(dog.clone()).insert(3, 7);

        // assert_eq!(map2.get(&foo).get(&1), &5);
        // assert_eq!(map2.get(&bar).get(&2), &6);
        // assert_eq!(map2.get(&dog).get(&3), &7);

        // map2.get_mut(foo.clone()).insert(1, 10);
        // map2.get_mut(bar.clone()).insert(2, 11);
        // map2.get_mut(dog.clone()).insert(3, 12);

        // assert_eq!(map1.get(&foo).get(&1), &10);
        // assert_eq!(map1.get(&bar).get(&2), &11);
        // assert_eq!(map1.get(&dog).get(&3), &12);
    }

    #[test]
    fn multiple_structures() {
        // let server: Server = Server::new(7878);

        // let mut map1: SMap<String, i32> = SMap::new(7878, 1);
        // let mut map2: SMap<String, i32> = SMap::new(7878, 1);
        // let mut map3: SMap<String, i32> = SMap::new(7878, 2);
        // let mut map4: SMap<String, i32> = SMap::new(7878, 2);

        // let foo = String::from("foo");
        // let bar = String::from("bar");
        // let dog = String::from("dog");

        // map1.insert(foo.clone(), 1);
        // map1.insert(bar.clone(), 2);
        // map1.insert(dog.clone(), 3);

        // map3.insert(foo.clone(), 4);
        // map3.insert(bar.clone(), 5);
        // map3.insert(dog.clone(), 6);

        // assert_eq!(map2.get(&foo), 1);
        // assert_eq!(map2.get(&bar), 2);
        // assert_eq!(map2.get(&dog), 3);

        // assert_eq!(map4.get(&foo), 4);
        // assert_eq!(map4.get(&bar), 5);
        // assert_eq!(map4.get(&dog), 6);
    }
}
