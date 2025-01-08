use crate::messages::{ClientMessage, ServerMessage};
use crate::umessage::UMessage;
use crate::update;
use serde::{Deserialize, Serialize};
use serde_json::to_vec;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::AtomicU32;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};
use std::{result, thread};
use tokio_util::bytes::{self, BytesMut};
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};
use update::Updatable;

pub struct Synchronizer<T>
where
    T: Updatable + Serialize,
    <T as Updatable>::Update: Serialize,
{
    inner: Arc<Mutex<T>>,
    last_packet_number: Arc<AtomicU32>,
    group_id: u32,
    connection: TcpStream,
    receiver: mpsc::Receiver<ResponseType>,
}

enum ResponseType {
    Accepted,
    Rejected,
}

pub enum SError {
    ConnectionError(String),
    ServerError(String),
    InternalError(String),
}
pub type Result<T> = result::Result<T, SError>;

fn to_connection_error<T: ToString>(error: T) -> SError {
    SError::ConnectionError(error.to_string())
}

fn to_server_error<T: ToString>(error: T) -> SError {
    SError::ServerError(error.to_string())
}

fn to_internal_error<T: ToString>(error: T) -> SError {
    SError::InternalError(error.to_string())
}

fn send_client_message<W: Write>(message: ClientMessage, writer: &mut W) -> Result<()> {
    let serialized = to_vec(&message).map_err(to_internal_error)?;
    let mut framed = BytesMut::new();
    LengthDelimitedCodec::new()
        .encode(serialized.into(), &mut framed)
        .unwrap();
    writer.write_all(&framed).map_err(to_internal_error)?;
    writer.flush().map_err(to_internal_error)?;
    Ok(())
}

fn stream_server_messages<R: Read>(reader: R, sender: Sender<ServerMessage>) -> Result<()> {
    let mut codec = LengthDelimitedCodec::new();
    let mut buffer = BytesMut::new();
    let mut reader = reader;

    let mut temp_buffer = [0; 1024];
    loop {
        let status = (|| -> Result<()> {
            let bytes_read = reader.read(&mut temp_buffer).map_err(to_internal_error)?;
            if bytes_read == 0 {
                return Err(SError::ServerError("Connection closed".to_owned()));
            }

            buffer.extend_from_slice(&temp_buffer[..bytes_read]);

            loop {
                let frame = codec.decode(&mut buffer).map_err(to_internal_error)?;
                if let Some(frame) = frame {
                    let message: ServerMessage =
                        serde_json::from_slice(&frame).map_err(to_internal_error)?;
                    sender.send(message).map_err(to_internal_error)?;
                } else {
                    break;
                }
            }
            Ok(())
        })();
        if let Err(error) = status {
            drop(sender);
            return Err(error);
        }
    }
}

impl<T> Synchronizer<T>
where
    T: Updatable + Default + Serialize + for<'de> Deserialize<'de> + Send + 'static,
    <T as Updatable>::Update: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    pub fn new(port: u16, group: u32) -> Result<Self> {
        let inner = Arc::new(Mutex::new(T::default()));
        let tcp_stream =
            TcpStream::connect(format!("127.0.0.1:{}", port)).map_err(to_connection_error)?;
        let last_packet_number = Arc::new(AtomicU32::new(0));
        let (server_message_sender, server_message_receiver) = channel();
        let _ = {
            let tcp_stream = tcp_stream.try_clone().map_err(to_internal_error)?;
            thread::spawn(|| stream_server_messages(tcp_stream, server_message_sender));
        };
        let _ = {
            let mut tcp_stream = &tcp_stream;
            send_client_message(ClientMessage::JoinGroup(group), &mut tcp_stream)
        }?;
        let _ = {
            let message = server_message_receiver.recv().map_err(to_internal_error)?;
            match message {
                ServerMessage::Correct => {
                    dbg!("Connected to the server");
                    Ok(())
                }
                _ => Err(SError::ConnectionError(
                    "Server didn't accept join request".to_owned(),
                )),
            }
        }?;
        let (response_sender, response_receiver) = channel();
        let result = {
            let connection = tcp_stream.try_clone().map_err(to_internal_error)?;
            Synchronizer {
                inner: inner.clone(),
                last_packet_number: last_packet_number.clone(),
                connection,
                group_id: group,
                receiver: response_receiver,
            }
        };
        thread::spawn(move || {
            let mut can_send_rejected = true;
            let mut should_send_reject_after_update = false;
            loop {
                let status = (|| -> Result<()> {
                    let message = server_message_receiver.recv().map_err(to_internal_error)?;
                    match message {
                        ServerMessage::Update(umessage) => {
                            dbg!("Received update");
                            last_packet_number.store(
                                umessage.packet_id + 1,
                                std::sync::atomic::Ordering::Relaxed,
                            );
                            let update = umessage.get_update().map_err(to_internal_error)?;
                            inner.lock().unwrap().apply_update(update);
                            if should_send_reject_after_update {
                                response_sender
                                    .send(ResponseType::Rejected)
                                    .map_err(to_internal_error)
                            } else {
                                can_send_rejected = true;
                                Ok(())
                            }
                        }
                        ServerMessage::Correct => {
                            dbg!("Received Correct");
                            response_sender
                                .send(ResponseType::Accepted)
                                .map_err(to_internal_error)
                        }
                        ServerMessage::Error => {
                            dbg!("Received Error");
                            if can_send_rejected {
                                can_send_rejected = false;
                                response_sender
                                    .send(ResponseType::Rejected)
                                    .map_err(to_internal_error)
                            } else {
                                should_send_reject_after_update = true;
                                Ok(())
                            }
                        }
                    }
                })();
                if let Err(_) = status {
                    let _ = tcp_stream.shutdown(std::net::Shutdown::Both);
                    break;
                }
            }
        });
        Ok(result)
    }

    pub fn publish_update(&mut self, update: T::Update) -> Result<()> {
        loop {
            let packet_id = self
                .last_packet_number
                .load(std::sync::atomic::Ordering::Relaxed);
            let group_id = self.group_id;
            let umessage = UMessage::new(group_id, packet_id, &update).unwrap();
            let message = ClientMessage::Update(umessage);
            let mut tcp_stream = &self.connection;
            send_client_message(message, &mut tcp_stream)?;
            match self.receiver.recv() {
                Ok(response) => {
                    if let ResponseType::Accepted = response {
                        return Ok(());
                    }
                }
                Err(error) => {
                    return Err(to_internal_error(error));
                }
            }
        }
    }
}

impl<T> Synchronizer<T>
where
    T: Updatable + Default + Serialize + for<'de> Deserialize<'de> + Send + 'static,
    <T as Updatable>::Update: Serialize + for<'de> Deserialize<'de> + Send + 'static,
{
    pub fn get_lock(&self) -> std::sync::MutexGuard<'_, T> {
        self.inner.lock().unwrap()
    }
}

impl<T> Drop for Synchronizer<T>
where
    T: Updatable + Serialize,
    <T as Updatable>::Update: Serialize,
{
    fn drop(&mut self) {
        let _ = self.connection.shutdown(std::net::Shutdown::Both);
    }
}
