use serde::{Deserialize, Serialize};
use crate::umessage;
use umessage::UMessage;

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    Update(UMessage),
    Correct,
    Error,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    JoinGroup(u32),
    Update(UMessage),
}