use serde::{Deserialize, Serialize};

pub use crate::ship::ShipDefinition as ShipSnapshot;

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientHello {
    pub protocol_version: u32,
    pub client_role: String,
}

impl ClientHello {
    pub fn new(client_role: impl Into<String>) -> Self {
        Self {
            protocol_version: 1,
            client_role: client_role.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    ShipSnapshot(ShipSnapshot),
    Error { message: String },
}
