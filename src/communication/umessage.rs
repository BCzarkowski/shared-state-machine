use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UMessage {
    group_id: u32,
    pub packet_id: u32,
    pub update: String,
}

impl UMessage {
    pub fn new<T: Serialize>(
        group_id: u32,
        packet_id: u32,
        update: &T,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            group_id,
            packet_id,
            update: serde_json::to_string(update)?,
        })
    }
}

impl<'a> UMessage {
    pub fn get_update<T: Deserialize<'a>>(&'a self) -> Result<T, serde_json::Error> {
        serde_json::from_str(&self.update)
    }
}
