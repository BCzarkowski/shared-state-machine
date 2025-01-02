use crate::messages;
use futures::prelude::*;
use messages::{ClientMessage, ServerMessage};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    io::{self},
    sync::{Arc, Mutex},
};
use tokio::{
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpListener, TcpStream,
    },
    sync::broadcast,
};
use tokio_serde::{formats::*, SymmetricallyFramed};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

#[derive(Debug)]
pub struct Group {
    broadcast_tx: broadcast::Sender<ServerMessage>,
    current_packet_number: u32,
    updates_history: Vec<ServerMessage>,
}

impl Group {
    pub fn new(broadcast_tx: broadcast::Sender<ServerMessage>) -> Self {
        Self {
            broadcast_tx,
            current_packet_number: 0,
            updates_history: vec![],
        }
    }
}

#[derive(Debug)]
pub struct ServerState {
    groups: HashMap<u32, Arc<Mutex<Group>>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}

type Deserializer = SymmetricallyFramed<
    FramedRead<OwnedReadHalf, LengthDelimitedCodec>,
    Value,
    SymmetricalJson<Value>,
>;
type Serializer = SymmetricallyFramed<
    FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>,
    Value,
    SymmetricalJson<Value>,
>;

#[derive(Debug)]
pub struct Server {
    state: Arc<Mutex<ServerState>>,
    port: u16,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Self {
            state: Arc::new(Mutex::new(ServerState::new())),
            port,
        }
    }

    pub async fn run(&self) {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port))
            .await
            .unwrap();

        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let state = self.state.clone();
            tokio::spawn(async move {
                if let Err(e) = Server::handle_connection(socket, state).await {
                    eprintln!("Connection handling failed: {}", e);
                }
            });
        }
    }

    fn create_deserializer(reader: OwnedReadHalf) -> Deserializer {
        let length_delimited = FramedRead::new(reader, LengthDelimitedCodec::new());
        SymmetricallyFramed::new(length_delimited, SymmetricalJson::<Value>::default())
    }

    fn create_serializer(writer: OwnedWriteHalf) -> Serializer {
        let length_delimited = FramedWrite::new(writer, LengthDelimitedCodec::new());
        SymmetricallyFramed::new(length_delimited, SymmetricalJson::default())
    }

    async fn handle_connection(
        socket: TcpStream,
        state: Arc<Mutex<ServerState>>,
    ) -> std::io::Result<()> {
        let (reader, writer) = socket.into_split();

        let mut deserialized = Self::create_deserializer(reader);
        let mut serialized = Self::create_serializer(writer);

        let group_id = Self::read_group_id(&mut deserialized).await.unwrap_or(0);
        serialized
            .send(json!(ServerMessage::Correct))
            .await
            .unwrap();

        let group = Self::get_or_create_group(group_id, &state);
        Self::send_group_history(&group, &mut serialized).await?;
        Self::process_messages(&mut deserialized, &mut serialized, group).await
    }

    async fn read_group_id(deserialized: &mut Deserializer) -> Option<u32> {
        let message = deserialized.try_next().await.unwrap_or(None);
        match message {
            Some(value) => match serde_json::from_value(value).ok()? {
                ClientMessage::JoinGroup(group_id) => Some(group_id),
                _ => None,
            },
            _ => None,
        }
    }

    fn get_or_create_group(group_id: u32, state: &Arc<Mutex<ServerState>>) -> Arc<Mutex<Group>> {
        let mut state_lock = state.lock().unwrap();
        state_lock
            .groups
            .entry(group_id)
            .or_insert_with(|| {
                let (tx, _rx) = broadcast::channel(16);
                Arc::new(Mutex::new(Group::new(tx)))
            })
            .clone()
    }

    async fn send_group_history(
        group: &Arc<Mutex<Group>>,
        serialized: &mut Serializer,
    ) -> std::io::Result<()> {
        let history = {
            let group_lock = group.lock().unwrap();
            group_lock.updates_history.clone()
        };

        for update in history {
            serialized.send(json!(update)).await?;
        }

        Ok(())
    }

    async fn process_messages(
        deserialized: &mut Deserializer,
        serialized: &mut Serializer,
        group: Arc<Mutex<Group>>,
    ) -> std::io::Result<()> {
        let tx: broadcast::Sender<ServerMessage> = {
            let group_lock = group.lock().unwrap();
            group_lock.broadcast_tx.clone()
        };
        let mut rx = tx.subscribe();

        loop {
            tokio::select! {
                msg = deserialized.try_next() => {
                    Self::handle_incoming_message(msg, &group, &tx, serialized).await?;
                }
                message = rx.recv() => {
                    if let Ok(update) = message {
                        serialized.send(json!(update)).await.unwrap();
                    }
                }
            }
        }
    }

    async fn handle_incoming_message(
        msg: Result<Option<Value>, io::Error>,
        group: &Arc<Mutex<Group>>,
        tx: &broadcast::Sender<ServerMessage>,
        serialized: &mut Serializer,
    ) -> std::io::Result<()> {
        let msg = msg
            .unwrap_or(None)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid message"))?;

        let umessage = match serde_json::from_value(msg) {
            Ok(ClientMessage::Update(umessage)) => umessage,
            Ok(_) => {
                eprintln!("Unexpected message from client");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Failed to deserialize message: {}", e);
                return Ok(());
            }
        };

        let server_response = {
            let mut group_lock = group.lock().unwrap();
            if umessage.packet_id != group_lock.current_packet_number {
                ServerMessage::Error
            } else {
                group_lock.current_packet_number += 1;
                let umessage = ServerMessage::Update(umessage);
                group_lock.updates_history.push(umessage.clone());
                tx.send(umessage).unwrap();
                ServerMessage::Correct
            }
        };

        serialized.send(json!(server_response)).await.unwrap();

        Ok(())
    }
}
