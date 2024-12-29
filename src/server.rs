use crate::messages;
use crate::umessage::UMessage;
use messages::{ClientMessage, ServerMessage};
use serde::{Deserialize, Serialize};
use serde_json::Result as SerdeResult;
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

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
            let (mut socket, _addr) = listener.accept().await.unwrap();
            let state = self.state.clone();

            tokio::spawn(async move {
                let (reader, writer) = socket.split();

                if let Err(e) = Server::handle_connection(reader, writer, state).await {
                    eprintln!("Connection handling failed: {}", e);
                }
            });
        }
    }

    async fn handle_connection<Reader, Writer>(
        reader: Reader,
        mut writer: Writer,
        state: Arc<Mutex<ServerState>>,
    ) -> std::io::Result<()>
    where
        Reader: AsyncRead + Unpin,
        Writer: AsyncWrite + Unpin,
    {
        let mut reader = BufReader::new(reader);
        let group_id = Self::read_group_id(&mut reader).await?;
        let group = Self::get_or_create_group(group_id, &state);
        Self::send_history(&group, &mut writer).await?;
        Self::process_messages(reader, writer, group).await
    }

    async fn read_group_id<Reader>(reader: &mut BufReader<Reader>) -> std::io::Result<u32>
    where
        Reader: AsyncRead + Unpin,
    {
        let mut line = String::new();
        if reader.read_line(&mut line).await.unwrap() == 0 {
            return Ok(0);
        }

        match serde_json::from_str::<ClientMessage>(&line) {
            Ok(ClientMessage::JoinGroup(group_id)) => Ok(group_id),
            Ok(_) => Ok(0),
            Err(e) => {
                eprintln!("Failed to deserialize message: {}", e);
                Ok(0)
            }
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

    async fn send_history<Writer>(
        group: &Arc<Mutex<Group>>,
        writer: &mut Writer,
    ) -> std::io::Result<()>
    where
        Writer: AsyncWrite + Unpin,
    {
        let history = {
            let group_lock = group.lock().unwrap();
            group_lock.updates_history.clone()
        };

        for update in history {
            let update = serde_json::to_string(&update).unwrap();
            writer.write_all(format!("{}\n", update).as_bytes()).await?;
        }

        Ok(())
    }

    async fn process_messages<Reader, Writer>(
        mut reader: BufReader<Reader>,
        mut writer: Writer,
        group: Arc<Mutex<Group>>,
    ) -> std::io::Result<()>
    where
        Reader: AsyncRead + Unpin,
        Writer: AsyncWrite + Unpin,
    {
        let tx = {
            let group_lock = group.lock().unwrap();
            group_lock.broadcast_tx.clone()
        };

        let mut rx = tx.subscribe();
        let mut line = String::new();

        loop {
            tokio::select! {
                bytes_read = reader.read_line(&mut line) => {
                    if bytes_read.unwrap() == 0 {
                        break Ok(());
                    }

                    let umessage = match serde_json::from_str::<ClientMessage>(&line) {
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

                    {
                        let mut group_lock = group.lock().unwrap();
                        group_lock.current_packet_number += 1;
                        let umessage = ServerMessage::Update(umessage);
                        group_lock.updates_history.push(umessage.clone());
                        tx.send(umessage).unwrap();
                    }

                    line.clear();
                }
                message = rx.recv() => {
                    let update = message.unwrap();
                    writer.write_all(format!("{}\n", serde_json::to_string(&update).unwrap()).as_bytes()).await?;
                }
            }
        }
    }
}
