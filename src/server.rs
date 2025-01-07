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
    sync::broadcast::{self, Receiver, Sender},
};
use tokio_serde::{formats::*, SymmetricallyFramed};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Client unexpected behavior: {0}")]
    CommunicationError(String),
    #[error("Error in message sending: {0}")]
    SendError(String),
    #[error("Error in message reading: {0}")]
    ReadError(String),
    #[error("Failed to acquire lock: {0}")]
    LockError(String),
}

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
    ) -> Result<(), ServerError> {
        let (reader, writer) = socket.into_split();

        let mut deserialized = Self::create_deserializer(reader);
        let mut serialized = Self::create_serializer(writer);

        let group_id = Self::read_group_id(&mut deserialized).await?;

        serialized
            .send(json!(ServerMessage::Correct))
            .await
            .map_err(|_e| ServerError::SendError("Initial message".into()))?;

        let group = Self::get_or_create_group(group_id, &state);

        let (tx, rx, history) = {
            let group_lock = group
                .lock()
                .map_err(|e| ServerError::LockError(e.to_string()))?;

            let tx = group_lock.broadcast_tx.clone();
            let history = group_lock.updates_history.clone();
            let rx = tx.subscribe();
            (tx, rx, history)
        };

        Self::send_group_history(history, &mut serialized).await?;
        Self::process_messages(&mut deserialized, &mut serialized, group, tx, rx).await
    }

    async fn read_group_id(deserialized: &mut Deserializer) -> Result<u32, ServerError> {
        match deserialized.try_next().await {
            Ok(Some(value)) => match serde_json::from_value::<ClientMessage>(value) {
                Ok(ClientMessage::JoinGroup(group_id)) => Ok(group_id),
                Ok(_) => Err(ServerError::CommunicationError(
                    "Unexpected message while reading Group ID".into(),
                )),
                Err(_e) => Err(ServerError::CommunicationError(
                    "Failed to parse group ID".into(),
                )),
            },
            Ok(None) => Err(ServerError::CommunicationError(
                "Client disconnected while reading group ID".into(),
            )),
            Err(_e) => Err(ServerError::ReadError("Group ID".into())),
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
        history: Vec<ServerMessage>,
        serialized: &mut Serializer,
    ) -> Result<(), ServerError> {
        for update in history {
            dbg!("Server sending | {}", &update);

            serialized
                .send(json!(update))
                .await
                .map_err(|_e| ServerError::SendError("History updates".into()))?;
        }
        Ok(())
    }

    async fn process_messages(
        deserialized: &mut Deserializer,
        serialized: &mut Serializer,
        group: Arc<Mutex<Group>>,
        tx: Sender<ServerMessage>,
        mut rx: Receiver<ServerMessage>,
    ) -> Result<(), ServerError> {
        loop {
            tokio::select! {
                msg = deserialized.try_next() => {
                    Self::handle_incoming_message(msg, &group, &tx, serialized).await?;
                }
                message = rx.recv() => {
                    if let Ok(update) = message {
                        serialized
                        .send(json!(update))
                        .await
                        .map_err(|_e| ServerError::SendError("Broadcast message".into()))?;
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
    ) -> Result<(), ServerError> {
        let msg = match msg {
            Ok(Some(value)) => value,
            Ok(None) => {
                eprintln!("Client disconnected while sending message.");
                return Ok(());
            }
            Err(_e) => {
                return Err(ServerError::ReadError("Incoming message".into()));
            }
        };

        let umessage = match serde_json::from_value(msg) {
            Ok(ClientMessage::Update(umessage)) => {
                dbg!("Server received UMessage | {}", &umessage);
                umessage
            }
            Ok(_) => {
                return Err(ServerError::CommunicationError(
                    "Unexpected message from client".into(),
                ));
            }
            Err(_e) => {
                return Err(ServerError::ReadError(
                    "Failed to deserialize message".into(),
                ));
            }
        };

        let server_response = {
            let mut group_lock = group
                .lock()
                .map_err(|e| ServerError::LockError(e.to_string()))?;

            if umessage.packet_id != group_lock.current_packet_number {
                ServerMessage::Error
            } else {
                group_lock.current_packet_number += 1;
                let umessage = ServerMessage::Update(umessage);
                group_lock.updates_history.push(umessage.clone());
                tx.send(umessage)
                    .map_err(|_e| ServerError::SendError("Failed to broadcast message".into()))?;

                ServerMessage::Correct
            }
        };

        dbg!("Server sending | {}", &server_response);
        serialized
            .send(json!(server_response))
            .await
            .map_err(|_e| ServerError::SendError("Failed to send server response".into()))?;

        Ok(())
    }
}
