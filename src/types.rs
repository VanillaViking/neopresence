use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct RequestMessage {
    jsonrpc: String,
    pub id: u32,
    pub method: String,
    pub params: String,
}
