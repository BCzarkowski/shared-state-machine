<<<<<<< HEAD
use shared_state_machine::communication::synchronizer;
use shared_state_machine::score::sstack::SStack;
use shared_state_machine::ucore::ustack::UStack;
use shared_state_machine::{communication::server::Server, ucore::umap::UMap};
=======
use shared_state_machine::sstack::SStack;
use shared_state_machine::synchronizer;
use shared_state_machine::ustack::UStack;
use shared_state_machine::{server::Server, umap::UMap};
>>>>>>> main
use std::{thread, time};
use tokio_util::sync::CancellationToken;

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn nested_structure() {
        let shutdown_token = CancellationToken::new();
        let server_shutdown_token = shutdown_token.clone();

        let port = 7850;
        let server_handle = tokio::spawn(async move {
            let server = Server::new(port.clone());
            server.run(server_shutdown_token).await
        });

        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let client_handle = tokio::task::spawn_blocking(move || {
            let status = (|| -> synchronizer::Result<()> {
                let mut stc1: SStack<UMap<String, UStack<i32>>> = SStack::new(port.clone(), 1)?;
                let mut stc2: SStack<UMap<String, UStack<i32>>> = SStack::new(port.clone(), 1)?;

                let bar = String::from("bar");

                stc1.push(UMap::new())?;
                stc1.push(UMap::new())?;

                stc1.top_mut().insert(bar.clone(), UStack::new())?;

                stc1.top_mut().get_mut(bar.clone()).push(1)?;
                stc1.top_mut().get_mut(bar.clone()).push(2)?;
                stc1.top_mut().get_mut(bar.clone()).push(5)?;
                stc1.top_mut().get_mut(bar.clone()).push(8)?;

                thread::sleep(time::Duration::from_millis(100));

                assert_eq!(
                    stc2.get_lock()
                        .top_ref()
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .top()
                        .unwrap(),
                    8
                );
                stc2.top_mut().get_mut(bar.clone()).pop()?;
                thread::sleep(time::Duration::from_millis(100));
                assert_eq!(
                    stc2.get_lock()
                        .top_ref()
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .top()
                        .unwrap(),
                    5
                );
                stc2.top_mut().get_mut(bar.clone()).pop()?;
                thread::sleep(time::Duration::from_millis(100));
                assert_eq!(
                    stc2.get_lock()
                        .top_ref()
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .top()
                        .unwrap(),
                    2
                );
                stc2.top_mut().get_mut(bar.clone()).pop()?;
                thread::sleep(time::Duration::from_millis(100));
                assert_eq!(
                    stc2.get_lock()
                        .top_ref()
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .top()
                        .unwrap(),
                    1
                );
                stc2.top_mut().get_mut(bar.clone()).pop()?;
                thread::sleep(time::Duration::from_millis(100));

                stc2.top_mut().get_mut(bar.clone()).push(4)?;
                stc2.top_mut().get_mut(bar.clone()).push(3)?;
                stc2.top_mut().get_mut(bar.clone()).push(2)?;
                stc2.top_mut().get_mut(bar.clone()).push(1)?;

                thread::sleep(time::Duration::from_millis(100));

                assert_eq!(
                    stc1.get_lock()
                        .top_ref()
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .top()
                        .unwrap(),
                    1
                );
                stc1.top_mut().get_mut(bar.clone()).pop()?;
                thread::sleep(time::Duration::from_millis(100));
                assert_eq!(
                    stc1.get_lock()
                        .top_ref()
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .top()
                        .unwrap(),
                    2
                );
                stc1.top_mut().get_mut(bar.clone()).pop()?;
                thread::sleep(time::Duration::from_millis(100));
                assert_eq!(
                    stc1.get_lock()
                        .top_ref()
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .top()
                        .unwrap(),
                    3
                );
                stc1.top_mut().get_mut(bar.clone()).pop()?;
                thread::sleep(time::Duration::from_millis(100));
                assert_eq!(
                    stc1.get_lock()
                        .top_ref()
                        .unwrap()
                        .get_ref(&bar)
                        .unwrap()
                        .top()
                        .unwrap(),
                    4
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
