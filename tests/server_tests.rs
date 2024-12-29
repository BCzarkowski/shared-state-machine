use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast,
};
use shared_state_machine::umessage::UMessage;
use shared_state_machine::messages::ClientMessage;
use shared_state_machine::server::{run_server_with_state, ServerState};
use shared_state_machine::ustack::UStack;

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

        let join1 = serde_json::to_string(&ClientMessage::JoinGroup(1)).unwrap();
        let join2 = serde_json::to_string(&ClientMessage::JoinGroup(2)).unwrap();
        let join1 = format!("{}\n", join1);
        let join2 = format!("{}\n", join2);

        client1.write_all(join1.as_bytes()).await.unwrap();
        client2.write_all(join1.as_bytes()).await.unwrap();


        let ustack: UStack<i32> = UStack::new();
        let push_5 = ustack.push(5);

        let update1 = serde_json::to_string(&ClientMessage::Update(UMessage::new(1, 1,&push_5).unwrap())).unwrap();
        let update1 = format!("{}\n", update1);
        let update2 = serde_json::to_string(&ClientMessage::Update(UMessage::new(1, 1,&push_5).unwrap())).unwrap();
        let update2 = format!("{}\n", update2);

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
        client3.write_all(join2.as_bytes()).await.unwrap();

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
        client4.write_all(join1.as_bytes()).await.unwrap();

        // Client 4 should receive history.
        client4.read_line(&mut line).await.unwrap();
        assert_eq!(line, update1);
        line.clear();
        client4.read_line(&mut line).await.unwrap();
        assert_eq!(line, update2);
        line.clear();

        // Client 3 sends a message and only him should receive it.
        let update3 = serde_json::to_string(&ClientMessage::Update(UMessage::new(1, 1,&push_5).unwrap())).unwrap();
        let update3 = format!("{}\n", update3);

        client3.write_all(update3.as_bytes()).await.unwrap();
        client3.read_line(&mut line).await.unwrap();
        assert_eq!(line, update3);
        line.clear();
    }
}
