#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum WorkStatus {
    Vacant,
    Working,
    Complete,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct WorkUnit {
    pub id: u32,
    pub status: WorkStatus,
    pub current_worker: String,
}

#[derive(Serialize, Deserialize)]
pub enum ClientMessage {
    RequestOpenWorkUnit,
    Take(u32, String), // work_unit_id, client_name
    Complete(u32, String), // work_unit_id, client_name
    InvalidConnection, // only useful to the server
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    Unit(u32),
    Accepted,
    Denied,
    Unknown,
}
