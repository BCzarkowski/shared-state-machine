use crate::messages::{ClientMessage, ServerMessage};
use crate::umap::{UMap, UMapUpdate};
use crate::umessage::UMessage;
use crate::update;
use serde::{Deserialize, Serialize};
use serde_json::{to_vec, Value};
use std::hash::Hash;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::channel;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use tokio::net::tcp;
use tokio::sync::mpsc::error;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};
use update::Updatable;

pub struct SMap<K: Eq + Hash + Clone + Serialize, T: Updatable + Clone + Serialize> {
    map: Arc<Mutex<UMap<K, T>>>,
    last_packet_number: Arc<AtomicU32>,
    group_id: u32,
    connection: TcpStream,
    receiver: mpsc::Receiver<ResponseType>,
    name: String,
}

enum ResponseType {
    Accepted,
    Rejected,
}

fn send_client_message<W: Write>(message: ClientMessage, writer: &mut W) -> () {
    let serialized = to_vec(&message).unwrap();
    let mut framed = BytesMut::new();
    LengthDelimitedCodec::new()
        .encode(serialized.into(), &mut framed)
        .unwrap();
    writer.write_all(&framed).unwrap();
    writer.flush().unwrap();
}

fn receive_server_message<R: Read>(reader: &mut R) -> Result<ServerMessage, std::io::Error> {
    let mut codec = LengthDelimitedCodec::new();
    let mut buffer = BytesMut::new();

    // Read data from the socket until we get a full frame
    let mut temp_buffer = [0; 1024];
    loop {
        let bytes_read = reader.read(&mut temp_buffer)?;
        if bytes_read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Connection closed",
            ));
        }

        buffer.extend_from_slice(&temp_buffer[..bytes_read]);

        // Try to decode a frame
        if let Some(frame) = codec.decode(&mut buffer).unwrap() {
            // Deserialize the frame into the ServerMessage type
            let message: ServerMessage = serde_json::from_slice(&frame)?;
            return Ok(message);
        }
    }
}

impl<K, T> SMap<K, T>
where
    K: Eq + Hash + Clone + Serialize + for<'de> Deserialize<'de> + Send + 'static,
    T: Updatable + Clone + Serialize + for<'de> Deserialize<'de> + Send + 'static,
    <T as Updatable>::Update: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    pub fn new(port: u16, group: u32, name: String) -> Self {
        let map = Arc::new(Mutex::new(UMap::new()));
        let map_clone = map.clone();
        let connection = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        let tcp_stream = connection.try_clone().unwrap();
        let last_packet_number = Arc::new(AtomicU32::new(0));
        let packet_id = last_packet_number.clone();
        let (sender, receiver) = channel();
        let nname = name.clone();
        // let send_client_message = |message: ClientMessage| {
        //     let mut tcp_stream = &connection;
        //     serde_json::to_writer(tcp_stream, &message).unwrap();
        //     tcp_stream.flush().unwrap();
        // };
        {
            let mut tcp_stream = &tcp_stream;
            send_client_message(ClientMessage::JoinGroup(group), &mut tcp_stream);
        }
        {
            let mut tcp_stream = &tcp_stream;
            match receive_server_message(&mut tcp_stream).unwrap() {
                ServerMessage::Correct => (),
                _ => panic!("Expected ServerMessage::Correct"),
            }
        }
        dbg!("Server accepted connection | ".to_owned() + &name);
        thread::spawn(move || {
            let mut can_send_rejected = true;
            let mut should_send_reject_after_update = false;
            loop {
                dbg!("Enter another loop of receiving | ".to_owned() + &name);
                let mut tcp_stream = &tcp_stream;
                let message = receive_server_message(&mut tcp_stream).unwrap();
                match message {
                    ServerMessage::Update(umessage) => {
                        dbg!(
                            "Received update from server | ".to_owned()
                                + &name
                                + " | "
                                + &group.to_string()
                                + " | "
                                + &umessage.packet_id.to_string()
                        );
                        packet_id
                            .store(umessage.packet_id + 1, std::sync::atomic::Ordering::Relaxed);
                        dbg!(packet_id.load(std::sync::atomic::Ordering::Relaxed));
                        let update = umessage.get_update().unwrap();
                        map.lock().unwrap().apply_update(update);
                        if should_send_reject_after_update {
                            sender.send(ResponseType::Rejected).unwrap();
                        } else {
                            can_send_rejected = true;
                        }
                    }
                    ServerMessage::Correct => {
                        sender.send(ResponseType::Accepted).unwrap();
                    }
                    ServerMessage::Error => {
                        dbg!(
                            "Received Rejected | ".to_owned()
                                + &name
                                + " | "
                                + &can_send_rejected.to_string()
                        );
                        if can_send_rejected {
                            can_send_rejected = false;
                            sender.send(ResponseType::Rejected).unwrap();
                        } else {
                            should_send_reject_after_update = true;
                        }
                    }
                };
            }
        });
        SMap {
            map: map_clone,
            last_packet_number,
            connection,
            group_id: group,
            receiver,
            name: nname,
        }
    }

    fn publish_update(&mut self, update: UMapUpdate<K, T>) -> () {
        dbg!("Publishing an update | ".to_owned() + &self.name);
        loop {
            let packet_id = self
                .last_packet_number
                .load(std::sync::atomic::Ordering::Relaxed);
            dbg!("Sending update | ".to_owned() + &self.name + " | " + &packet_id.to_string());
            let group_id = self.group_id;
            let umessage = UMessage::new(group_id, packet_id, &update).unwrap();
            let message = ClientMessage::Update(umessage);
            let mut tcp_stream = &self.connection;
            send_client_message(message, &mut tcp_stream);
            match self.receiver.recv() {
                Ok(response) => {
                    if let ResponseType::Accepted = response {
                        dbg!("Published update successfully! | ".to_owned() + &self.name);
                        break;
                    }
                }
                Err(_) => {
                    panic!("TODO");
                }
            }
        }
    }

    pub fn insert(&mut self, key: K, value: T) -> () {
        dbg!("Inserting value to smap");
        Self::publish_update(self, UMapUpdate::Insert(key, value))
    }

    pub fn remove(&mut self, key: K) -> () {
        Self::publish_update(self, UMapUpdate::Remove(key))
    }

    pub fn get(&self, key: &K) -> Option<T> {
        self.map.lock().unwrap().get(key)
    }
}

// TODO - structure wrapper for S Structures
