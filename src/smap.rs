use crate::messages::{ClientMessage, ServerMessage};
use crate::umap::{UMap, UMapUpdate};
use crate::umessage::UMessage;
use crate::update;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::hash::Hash;
use std::io::Write;
use std::net::TcpStream;
use std::sync::mpsc::channel;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use update::Updatable;

pub struct SMap<K: Eq + Hash, T: Updatable + Clone> {
    map: Arc<Mutex<UMap<K, T>>>,
    last_packet_number: Arc<u32>,
    group_id: u32,
    connection: TcpStream,
    receiver: mpsc::Receiver<ResponseType>,
}

enum ResponseType {
    Accepted,
    Rejected,
}

impl<K: Eq + Hash + Serialize + for<'a> Deserialize<'a>, T: Updatable + Clone + Serialize>
    SMap<K, T>
where
    <T as Updatable>::Update: Serialize,
{
    pub fn new(port: u16, group: u32) -> Self {
        let map = Arc::new(Mutex::new(UMap::new()));
        let connection = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        let tcp_stream = connection.try_clone().unwrap();
        let last_packet_number = Arc::new(0);
        let packet_id = last_packet_number.clone();
        let (sender, receiver) = channel();
        thread::spawn(move || {
            let send_client_message = |message: ClientMessage| {
                let mut tcp_stream = &tcp_stream;
                serde_json::to_writer(tcp_stream, &message).unwrap();
                tcp_stream.flush().unwrap();
            };
            let receive_server_message = || {
                let mut deserializer = serde_json::Deserializer::from_reader(&tcp_stream);
                let value = Value::deserialize(&mut deserializer).unwrap();
                let message: ServerMessage = serde_json::from_value(value).unwrap();
                message
            };
            send_client_message(ClientMessage::JoinGroup(group));
            match receive_server_message() {
                ServerMessage::Correct => (),
                _ => panic!("Expected ServerMessage::Correct"),
            }
            loop {
                let message = receive_server_message();
                match message {
                    ServerMessage::Update(umessage) => {
                        *packet_id = umessage.packet_id;
                        let update = umessage.get_update().unwrap();
                        map.lock().unwrap().apply_update(update);
                    }
                    ServerMessage::Correct => {
                        sender.send(ResponseType::Accepted);
                    }
                    ServerMessage::Error => {
                        sender.send(ResponseType::Rejected);
                    }
                };
            }
        });
        SMap { map, connection }
    }

    fn publish_update(&mut self, update: UMapUpdate<K, T>) -> () {
        loop {
            let packet_id = *self.last_packet_number;
            let group_id = self.group_id;
            let umessage = UMessage::new(group_id, packet_id, &update).unwrap();
            let message = ClientMessage::Update(umessage);
            let mut tcp_stream = &self.connection;
            serde_json::to_writer(tcp_stream, &message).unwrap();
            tcp_stream.flush().unwrap();
            match self.receiver.recv() {
                Ok(response) => {
                    if let ResponseType::Accepted = response {
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
