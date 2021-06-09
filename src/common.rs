use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum FromClientMessage {
    Ping,
    InvalidConnection, // only useful to the server
}

#[derive(Serialize, Deserialize)]
pub enum FromServerMessage {
    Pong(String, usize), // Used for connection oriented protocols
    UnknownPong,         // Used for non-connection oriented protocols
}
