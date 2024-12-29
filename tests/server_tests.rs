use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};

use shared_state_machine::server::{run_server_with_state, ServerState};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_group_creation_and_update_broadcast() {
        // This tests covers following server functionalities:
        // (1) Server creation.
        // (2) Creating and joining groups.
        // (3) Broadcasting messages in a group.
        // (4) Sending history updates.
        // (5) TODO: Refusing to update when not correct.

        //-- (1) --//
        let server_state: Arc<Mutex<ServerState>> = Arc::new(Mutex::new(ServerState::new()));
        let server_state_clone = server_state.clone();

        tokio::spawn(async move {
            run_server_with_state(server_state_clone).await;
        });
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        //-- (2) --//
        let addr = "127.0.0.1:7878";
        let client1 = TcpStream::connect(addr).await.unwrap();
        let client2 = TcpStream::connect(addr).await.unwrap();
        let mut client1 = BufReader::new(client1);
        let mut client2 = BufReader::new(client2);

        client1.write_all(b"1\n").await.unwrap();
        client2.write_all(b"1\n").await.unwrap();

        let update1 = "Update from client 1\n";
        let update2 = "Update from client 2\n";

        client1.write_all(update1.as_bytes()).await.unwrap();

        //-- (3) --//
        let mut line = String::new();

        // Client 1 should receive update.
        client1.read_line(&mut line).await.unwrap();
        assert_eq!(line, update1);
        line.clear();

        // Client 2 should receive update.
        client2.read_line(&mut line).await.unwrap();
        assert_eq!(line, update1);
        line.clear();

        // Client 3 connects do different group.
        let client3 = TcpStream::connect(addr).await.unwrap();
        let mut client3 = BufReader::new(client3);
        client3.write_all(b"2\n").await.unwrap();

        // Client 2 sends and update that should be broadcasted to client 1.
        client2.write_all(update2.as_bytes()).await.unwrap();

        // Client 1 should receive update.
        client1.read_line(&mut line).await.unwrap();
        assert_eq!(line, update2);
        line.clear();

        // Client 2 should receive update.
        client2.read_line(&mut line).await.unwrap();
        assert_eq!(line, update2);
        line.clear();

        //-- (4) --//
        // Client 4 connects to group 1.
        let client4 = TcpStream::connect(addr).await.unwrap();
        let mut client4 = BufReader::new(client4);
        client4.write_all(b"1\n").await.unwrap();

        // Client 4 should receive history.
        client4.read_line(&mut line).await.unwrap();
        assert_eq!(line, update1);
        line.clear();
        client4.read_line(&mut line).await.unwrap();
        assert_eq!(line, update2);
        line.clear();

        // Client 3 sends a message and only him should receive it.
        let update3 = "Update from client 3\n";
        client3.write_all(update3.as_bytes()).await.unwrap();
        client3.read_line(&mut line).await.unwrap();
        assert_eq!(line, update3);
        line.clear();
    }
}
