use crate::messages::{ClientMessage, ServerMessage};
use crate::recursive_structure_wrapper::StructureWrapper;
use crate::umap::UMap;
use crate::umessage::UMessage;
use crate::update;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use update::Updatable;

pub struct SMap<K: Eq + Hash, T: Updatable> {
    map: Arc<Mutex<UMap<K, T>>>,
    connection: TcpStream,
}

impl<K: Eq + Hash, T: Updatable> SMap<K, T> {
    pub fn new(port: u16, group: u32) -> Self {
        let map = Arc::new(Mutex::new(UMap::new()));
        let connection = TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        let tcp_stream = connection.try_clone().unwrap();
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
            let expect_server_correct = || {
                let actual_message = receive_server_message();
                match actual_message {
                    ServerMessage::Correct => (),
                    _ => panic!("Expected ServerMessage::Correct"),
                }
            };
            send_client_message(ClientMessage::JoinGroup(group));
            expect_server_correct();
            loop {
                let message = receive_server_message();
                match message {
                    ServerMessage::Update(update) => {
                        expect_server_correct();
                    }
                    _ => {
                        panic!("Expected ServerMessage::Update");
                    }
                }
            }
        });
        SMap { map, connection }
    }

    pub fn insert(&self, key: K, value: T) -> UMapUpdate<K, T> {
        UMapUpdate::Insert(key, value)
    }

    pub fn remove(&self, key: K) -> UMapUpdate<K, T> {
        UMapUpdate::Remove(key)
    }

    pub fn get(&self, key: &K) -> Option<&T> {
        self.map.get(key)
    }

    pub fn get_wrapped(
        &self,
        key: K,
    ) -> StructureWrapper<
        T,
        UMapUpdate<K, T>,
        impl FnOnce(T::Update) -> UMapUpdate<K, T>, // + use<'_, K, T>,
    > {
        StructureWrapper {
            structure: self.get(&key).unwrap(),
            outside_wrapper: move |update| UMapUpdate::Nested(key, update),
        }
    }
}

impl<K: Eq + Hash, T: Updatable, O, F: FnOnce(UMapUpdate<K, T>) -> O>
    StructureWrapper<'_, UMap<K, T>, O, F>
{
    pub fn insert(self, key: K, value: T) -> O {
        (self.outside_wrapper)(self.structure.insert(key, value))
    }

    pub fn remove(self, key: K) -> O {
        (self.outside_wrapper)(self.structure.remove(key))
    }
}
