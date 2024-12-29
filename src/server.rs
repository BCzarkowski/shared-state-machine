use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};
use serde_json::Result as SerdeResult;
use crate::messages;
use crate::umessage::UMessage;
use messages::{ClientMessage, ServerMessage};

#[derive(Debug)]
pub struct Group {
    id: u32,
    broadcast_tx: broadcast::Sender<ServerMessage>,
    current_packet_number: u32,
    updates_history: Vec<ServerMessage>,
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

pub async fn run_server() {
    let server_state: Arc<Mutex<ServerState>> = Arc::new(Mutex::new(ServerState::new()));
    let server_state_clone = server_state.clone();

    tokio::spawn(async move {
        run_server_with_state(server_state_clone).await;
    });
}

pub async fn run_server_with_state(state: Arc<Mutex<ServerState>>) {
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();

    loop {
        let (mut socket, _addr) = listener.accept().await.unwrap();
        let state = state.clone();

        tokio::spawn(async move {
            let (reader, writer) = socket.split();

            handle_connection(reader, writer, state)
                .await
                .expect("Failed to handle connection.");
        });
    }
}

pub async fn handle_connection<Reader, Writer>(
    reader: Reader,
    mut writer: Writer,
    state: Arc<Mutex<ServerState>>,
) -> std::io::Result<()>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin,
{
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    if reader.read_line(&mut line).await.unwrap() == 0 {
        return Ok(());
    }

    let group_id = match serde_json::from_str::<ClientMessage>(&line) {
        Ok(message) => {
            println!("Deserialized message: {:?}", message);
            match message {
                ClientMessage::JoinGroup(group_id)=> group_id,
                _ => {
                    eprintln!("Unexpected message from client: {:?}", message);
                    // return Ok(());
                    0
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to deserialize message from: {}", e);
            0
        }
    };

    // let group_id: u32 = line.trim().parse().unwrap();
    line.clear();

    // Retrieve or create the group
    let group = {
        let mut state_lock = state.lock().unwrap();
        state_lock
            .groups
            .entry(group_id)
            .or_insert_with(|| {
                let (tx, _rx) = broadcast::channel(16);
                Arc::new(Mutex::new(Group {
                    id: group_id,
                    broadcast_tx: tx,
                    current_packet_number: 0,
                    updates_history: vec![],
                }))
            })
            .clone()
    };

    // Send updates history to the client
    let history = {
        let group_lock = group.lock().unwrap();
        group_lock.updates_history.clone()
    };
    for update in history {
        let update = serde_json::to_string(&update).unwrap();
        let update = format!("{}\n", update);
        writer.write_all(update.as_bytes()).await?;
    }

    let tx = {
        let group_lock = group.lock().unwrap();
        group_lock.broadcast_tx.clone()
    };

    let mut rx = tx.subscribe();

    loop {
        tokio::select! {
            bytes_read = reader.read_line(&mut line) => {
                if bytes_read.unwrap() == 0 {
                    break Ok(());
                }
                // let update = line.trim().to_string();
                // line.clear();

                let umessage = match serde_json::from_str::<ClientMessage>(&line) {
                    Ok(message) => {
                        println!("Deserialized message: {:?}", message);
                        match message {
                            ClientMessage::Update(umessage)=> umessage,
                            _ => {
                                eprintln!("Unexpected message from client: {:?}", message);
                                return Ok(());
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to deserialize message from: {}", e);
                        return Ok(());
                    }
                };
                
                // Create a new update and broadcast it
                let mut group_lock = group.lock().unwrap();
                group_lock.current_packet_number += 1;
                let umessage = ServerMessage::Update(umessage);
                group_lock.updates_history.push(umessage.clone());
                tx.send(umessage).unwrap();
                line.clear();
            }
            message = rx.recv() => {
                let update = message.unwrap();
                let update = serde_json::to_string(&update).unwrap();
                let update = format!("{}\n", update);
                writer.write_all(update.as_bytes()).await?;
            }
        }
    }
}
