use shared_state_machine::communication::synchronizer;
use shared_state_machine::sync::svec::SVec;
use shared_state_machine::updateable::uvec::UVec;
use shared_state_machine::{communication::server::Server, updateable::umap::UMap};
use std::{thread, time};
use tokio_util::sync::CancellationToken;

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn nested_structure() {
        let shutdown_token = CancellationToken::new();
        let server_shutdown_token = shutdown_token.clone();

        let port = 7860;
        let server_handle = tokio::spawn(async move {
            let server = Server::new(port.clone());
            server.run(server_shutdown_token).await
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let client_handle = tokio::task::spawn_blocking(move || {
            let status = (|| -> synchronizer::Result<()> {
                let mut vec1: SVec<UMap<String, UVec<i32>>> = SVec::new(port.clone(), 1)?;
                let mut vec2: SVec<UMap<String, UVec<i32>>> = SVec::new(port.clone(), 1)?;

                let bar = String::from("bar");

                vec1.push(UMap::new())?;
                vec1.push(UMap::new())?;

                vec1.get_mut(1).insert(bar.clone(), UVec::new())?;

                vec1.get_mut(1).get_mut(bar.clone()).push(1)?;
                vec1.get_mut(1).get_mut(bar.clone()).push(2)?;
                vec1.get_mut(1).get_mut(bar.clone()).push(5)?;
                vec1.get_mut(1).get_mut(bar.clone()).push(8)?;

                thread::sleep(time::Duration::from_millis(100));

                assert_eq!(
                    vec2.get_lock()
                        .get_ref(1)
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .get(0)
                        .unwrap(),
                    1
                );
                assert_eq!(
                    vec2.get_lock()
                        .get_ref(1)
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .get(1)
                        .unwrap(),
                    2
                );
                assert_eq!(
                    vec2.get_lock()
                        .get_ref(1)
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .get(2)
                        .unwrap(),
                    5
                );
                assert_eq!(
                    vec2.get_lock()
                        .get_ref(1)
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .get(3)
                        .unwrap(),
                    8
                );

                vec2.get_mut(1).get_mut(bar.clone()).push(4)?;
                vec2.get_mut(1).get_mut(bar.clone()).push(3)?;
                vec2.get_mut(1).get_mut(bar.clone()).push(2)?;
                vec2.get_mut(1).get_mut(bar.clone()).push(1)?;

                thread::sleep(time::Duration::from_millis(100));

                assert_eq!(
                    vec1.get_lock()
                        .get_ref(1)
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .get(4)
                        .unwrap(),
                    4
                );
                assert_eq!(
                    vec1.get_lock()
                        .get_ref(1)
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .get(5)
                        .unwrap(),
                    3
                );
                assert_eq!(
                    vec1.get_lock()
                        .get_ref(1)
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .get(6)
                        .unwrap(),
                    2
                );
                assert_eq!(
                    vec1.get_lock()
                        .get_ref(1)
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .get(7)
                        .unwrap(),
                    1
                );

                // Graceful shutdown of server.
                shutdown_token.cancel();
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

        server_handle.await.unwrap();
    }
}
