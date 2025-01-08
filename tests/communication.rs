use shared_state_machine::smap::SMap;
use shared_state_machine::update::Updatable;
use shared_state_machine::uvec::UVec;
use shared_state_machine::{server::Server, smap, synchronizer, umap::UMap};
use std::{thread, time};

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn two_clients() {
        let port = 7870;
        let server_handle = tokio::spawn(async move {
            let server = Server::new(port.clone());
            server.run().await
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let client_handle = tokio::task::spawn_blocking(move || {
            let status = (|| -> synchronizer::Result<()> {
                let mut map1: SMap<String, i32> = SMap::new(port.clone(), 1)?;
                let mut map2: SMap<String, i32> = SMap::new(port.clone(), 1)?;

                let foo = String::from("foo");
                let bar = String::from("bar");
                let dog = String::from("dog");

                map1.insert(foo.clone(), 1);
                map1.insert(bar.clone(), 2);
                map1.insert(dog.clone(), 3);

                thread::sleep(time::Duration::from_millis(100));

                assert_eq!(map2.get(&foo), Some(1));
                assert_eq!(map2.get(&bar), Some(2));
                assert_eq!(map2.get(&dog), Some(3));

                map2.insert(foo.clone(), 9);
                map2.insert(bar.clone(), 8);
                map2.insert(dog.clone(), 7);

                thread::sleep(time::Duration::from_millis(100));

                assert_eq!(map1.get(&foo), Some(9));
                assert_eq!(map1.get(&bar), Some(8));
                assert_eq!(map1.get(&dog), Some(7));
                Ok(())
            })();
            if let Err(_) = status {
                panic!("Test failed!");
            }
        });

        match client_handle.await {
            Ok(_) => println!("Blocking client test completed successfully."),
            Err(e) => {
                panic!("Blocking client test failed: {:?}", e)
            }
        }

        server_handle.abort();
    }

    #[tokio::test]
    async fn nested_structure() {
        let port = 7871;
        let server_handle = tokio::spawn(async move {
            let server = Server::new(port.clone());
            server.run().await
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let client_handle = tokio::task::spawn_blocking(move || {
            let status = (|| -> synchronizer::Result<()> {
                let mut map1: SMap<String, UMap<i32, i32>> = SMap::new(port.clone(), 1)?;
                let mut map2: SMap<String, UMap<i32, i32>> = SMap::new(port.clone(), 1)?;

                let foo = String::from("foo");
                let bar = String::from("bar");
                let dog = String::from("dog");

                map1.insert(foo.clone(), UMap::new());
                map1.insert(bar.clone(), UMap::new());
                map1.insert(dog.clone(), UMap::new());

                map1.get_mut(foo.clone()).insert(1, 5);
                map1.get_mut(bar.clone()).insert(2, 6);
                map1.get_mut(dog.clone()).insert(3, 7);

                thread::sleep(time::Duration::from_millis(100));

                assert_eq!(map2.get(&foo).unwrap().get(&1).unwrap(), 5);
                assert_eq!(map2.get(&bar).unwrap().get(&2).unwrap(), 6);
                assert_eq!(map2.get(&dog).unwrap().get(&3).unwrap(), 7);

                map2.get_mut(foo.clone()).insert(1, 10);
                map2.get_mut(bar.clone()).insert(2, 11);
                map2.get_mut(dog.clone()).insert(3, 12);

                thread::sleep(time::Duration::from_millis(100));

                assert_eq!(map1.get(&foo).unwrap().get(&1).unwrap(), 10);
                assert_eq!(map1.get(&bar).unwrap().get(&2).unwrap(), 11);
                assert_eq!(map1.get(&dog).unwrap().get(&3).unwrap(), 12);
                Ok(())
            })();
            if let Err(_) = status {
                panic!("Test failed!");
            }
        });

        match client_handle.await {
            Ok(_) => println!("Blocking client test completed successfully."),
            Err(e) => {
                panic!("Blocking client test failed: {:?}", e)
            }
        }

        server_handle.abort();
    }

    #[tokio::test]
    async fn multiple_structures() {
        let port = 7872;
        let server_handle = tokio::spawn(async move {
            let server = Server::new(port.clone());
            server.run().await
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let client_handle = tokio::task::spawn_blocking(move || {
            let status = (|| -> synchronizer::Result<()> {
                let mut map1: SMap<String, i32> = SMap::new(port.clone(), 1)?;
                let map2: SMap<String, i32> = SMap::new(port.clone(), 1)?;
                let mut map3: SMap<String, i32> = SMap::new(port.clone(), 2)?;
                let map4: SMap<String, i32> = SMap::new(port.clone(), 2)?;

                let foo = String::from("foo");
                let bar = String::from("bar");
                let dog = String::from("dog");

                map1.insert(foo.clone(), 1);
                map1.insert(bar.clone(), 2);
                map1.insert(dog.clone(), 3);

                map3.insert(foo.clone(), 4);
                map3.insert(bar.clone(), 5);
                map3.insert(dog.clone(), 6);

                thread::sleep(time::Duration::from_millis(100));

                assert_eq!(map2.get(&foo).unwrap(), 1);
                assert_eq!(map2.get(&bar).unwrap(), 2);
                assert_eq!(map2.get(&dog).unwrap(), 3);

                assert_eq!(map4.get(&foo).unwrap(), 4);
                assert_eq!(map4.get(&bar).unwrap(), 5);
                assert_eq!(map4.get(&dog).unwrap(), 6);

                Ok(())
            })();
            if let Err(_) = status {
                panic!("Test failed!");
            }
        });

        match client_handle.await {
            Ok(_) => println!("Blocking client test completed successfully."),
            Err(e) => {
                panic!("Blocking client test failed: {:?}", e)
            }
        }

        server_handle.abort();
    }
}
