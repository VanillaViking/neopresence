use std::fs::OpenOptions;

use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::io::prelude::*;

use crate::nvim;

#[derive(Serialize, Deserialize)]
struct LogMessage {
    method: String,
    params: LogMessageParams,
}

impl LogMessage {
    fn new(method: String, params: LogMessageParams) -> Self {
        Self { method, params }
    }
}

#[derive(Serialize, Deserialize)]
struct LogMessageParams {
    #[serde(rename = "type")]
    message_type: MessageType,
    message: String,
}

impl LogMessageParams {
    fn new(message_type: MessageType, message: String) -> Self {
        Self {
            message_type,
            message,
        }
    }
}

#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum MessageType {
    Error = 1,
    Warning = 2,
    Info = 3,
    Log = 4,
    Debug = 5,
}

pub fn log(message: &str, message_type: MessageType) {
    let log_message = LogMessage::new(
        String::from("window/logMessage"),
        LogMessageParams::new(message_type, message.to_owned()),
    );

    nvim::send(&serde_json::to_string(&log_message).unwrap());
}

//TODO: panics if log file is missing
pub fn ghetto_log(message: &str) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("/home/vanilla/log")
        .unwrap();

    writeln!(file, "{message}");
}
