use serde::{Deserialize, Serialize};

// To Server
#[derive(Deserialize, Serialize)]
pub struct Send {
    pub destinations: Vec<String>,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct Identify {
    pub display_name: String,
}

// To Client
#[derive(Deserialize, Serialize)]
pub struct Message {
    pub payload: String,
}