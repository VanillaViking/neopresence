use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct RequestMessage {
    pub jsonrpc: String,
    pub id: u32,
    pub method: String,
    #[serde(skip)]
    pub params: String,
}

impl RequestMessage {
    pub fn decode(input: &str) -> RequestMessage {
        //logger::log(message.trim(), logger::MessageType::Error);
        serde_json::from_str(input.trim()).unwrap()
    }
}
