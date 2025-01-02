use futures::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use shared_state_machine::messages::{ClientMessage, ServerMessage};
use shared_state_machine::server::Server;
use shared_state_machine::umessage::UMessage;
use shared_state_machine::ustack::UStack;
use tokio::{
    io::{
        AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt,
        BufReader,
    },
    net::{TcpListener, TcpStream},
    sync::broadcast,
};
use tokio_serde::formats::*;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

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
        tokio::spawn(async move {
            let server = Server::new(7878);
            server.run().await;
        });
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        //-- (2) --//
        let addr = "127.0.0.1:7878";

        // Setup clients' readers and writers.
        let client1 = TcpStream::connect(addr).await.unwrap();
        let (reader, writer) = client1.into_split();
        let mut reader1 = {
            let length_delimited = FramedRead::new(reader, LengthDelimitedCodec::new());
            tokio_serde::SymmetricallyFramed::new(
                length_delimited,
                SymmetricalJson::<Value>::default(),
            )
        };
        let mut writer1 = {
            let length_delimited = FramedWrite::new(writer, LengthDelimitedCodec::new());
            tokio_serde::SymmetricallyFramed::new(length_delimited, SymmetricalJson::default())
        };

        let client2 = TcpStream::connect(addr).await.unwrap();
        let (reader, writer) = client2.into_split();
        let mut reader2 = {
            let length_delimited = FramedRead::new(reader, LengthDelimitedCodec::new());
            tokio_serde::SymmetricallyFramed::new(
                length_delimited,
                SymmetricalJson::<Value>::default(),
            )
        };
        let mut writer2 = {
            let length_delimited = FramedWrite::new(writer, LengthDelimitedCodec::new());
            tokio_serde::SymmetricallyFramed::new(length_delimited, SymmetricalJson::default())
        };

        let client3 = TcpStream::connect(addr).await.unwrap();
        let (reader, writer) = client3.into_split();
        let mut reader3 = {
            let length_delimited = FramedRead::new(reader, LengthDelimitedCodec::new());
            tokio_serde::SymmetricallyFramed::new(
                length_delimited,
                SymmetricalJson::<Value>::default(),
            )
        };
        let mut writer3 = {
            let length_delimited = FramedWrite::new(writer, LengthDelimitedCodec::new());
            tokio_serde::SymmetricallyFramed::new(length_delimited, SymmetricalJson::default())
        };

        let client4 = TcpStream::connect(addr).await.unwrap();
        let (reader, writer) = client4.into_split();
        let mut reader4 = {
            let length_delimited = FramedRead::new(reader, LengthDelimitedCodec::new());
            tokio_serde::SymmetricallyFramed::new(
                length_delimited,
                SymmetricalJson::<Value>::default(),
            )
        };
        let mut writer4 = {
            let length_delimited = FramedWrite::new(writer, LengthDelimitedCodec::new());
            tokio_serde::SymmetricallyFramed::new(length_delimited, SymmetricalJson::default())
        };
        // END SETUP

        let join1 = ClientMessage::JoinGroup(1);
        let join2 = ClientMessage::JoinGroup(2);

        // Client 1 joins group 1.
        writer1.send(json!(join1)).await.unwrap();
        let msg = reader1.try_next().await.unwrap().unwrap();
        assert_eq!(msg, json!(ServerMessage::Correct));

        // Client 2 joins group 1.
        writer2.send(json!(join1)).await.unwrap();
        let msg = reader2.try_next().await.unwrap().unwrap();
        assert_eq!(msg, json!(ServerMessage::Correct));

        let ustack: UStack<i32> = UStack::new();
        let push_5 = ustack.push(5);

        let update1 = &ClientMessage::Update(UMessage::new(1, 1, &push_5).unwrap());
        let update2 = ClientMessage::Update(UMessage::new(1, 1, &push_5).unwrap());

        writer1.send(json!(update1)).await.unwrap();

        //-- (3) --//

        // Client 1 should receive update.
        let msg = reader1.try_next().await.unwrap().unwrap();
        assert_eq!(msg, json!(update1));

        // Client 2 should receive update.
        let msg = reader2.try_next().await.unwrap().unwrap();
        assert_eq!(msg, json!(update1));

        // Client 3 connects to a different group.
        writer3.send(json!(join2)).await.unwrap();
        let msg = reader3.try_next().await.unwrap().unwrap();
        assert_eq!(msg, json!(ServerMessage::Correct));

        // Client 2 sends and update that should be broadcasted to client 1.
        writer2.send(json!(update2)).await.unwrap();

        // Client 1 should receive update.
        let msg = reader1.try_next().await.unwrap().unwrap();
        assert_eq!(msg, json!(update2));

        // Client 2 should receive update.
        let msg = reader2.try_next().await.unwrap().unwrap();
        assert_eq!(msg, json!(update1));

        //-- (4) --//
        // Client 4 connects to group 1.
        writer4.send(json!(join1)).await.unwrap();
        let msg = reader4.try_next().await.unwrap().unwrap();
        assert_eq!(msg, json!(ServerMessage::Correct));

        // Client 4 should receive history.
        let msg = reader4.try_next().await.unwrap().unwrap();
        assert_eq!(msg, json!(update1));
        let msg = reader4.try_next().await.unwrap().unwrap();
        assert_eq!(msg, json!(update2));

        // Client 3 sends a message and only him should receive it.
        let update3 = ClientMessage::Update(UMessage::new(1, 1, &push_5).unwrap());
        writer3.send(json!(update3)).await.unwrap();
        let msg = reader3.try_next().await.unwrap().unwrap();
        assert_eq!(msg, json!(update3));
    }
}
