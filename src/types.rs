use std::{collections::HashMap, process::exit, time::{SystemTime, UNIX_EPOCH}};

use discord_presence::{models::EventData, Client, Event};
use lsp_types::{DidChangeTextDocumentParams, TextDocumentItem};
use serde::{Deserialize, Serialize};

pub struct Context {
    pub drpc: Client,
    pub start_time: u64,
    pub changed_files: HashMap<String, FileData>,
}
impl Context {
    pub fn new(discord_client_id: u64) -> Self {
        let mut drpc = Client::new(discord_client_id);
        drpc.on_ready(|_ctx| {
            // println!("ready?");
        })
        .persist();
        drpc.on_error(move |err| {
            if let EventData::Error(err) = err.event {
                let msg = err.message.unwrap_or_default();
                if msg == "Io Error" {
                    // TODO: change this to instead retry connection every ~5 seconds
                    exit(1);
                }
            }
        })
        .persist();
        drpc.start();
        drpc.block_until_event(Event::Ready).unwrap();

        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get system time")
            .as_secs();

        return Self {
            drpc,
            start_time,
            changed_files: HashMap::new()
        }

    }

    pub fn update_file_contents(&mut self, filename: &str, new_contents: &str) -> Result<(), &str> {
        if filename == "" {
            return Err("no filename")
        }

        let file_data = self.changed_files.entry(filename.to_string()).or_insert(FileData { original_contents: new_contents.to_owned(), latest_contents: String::from("")});

        file_data.latest_contents = new_contents.to_string();

        Ok(())
    }
}

pub struct FileData {
    pub original_contents: String,
    pub latest_contents: String,
}

pub fn decode<'a, T: Deserialize<'a>>(input: &'a str) -> T {
    serde_json::from_str(input.trim()).unwrap()
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct InitializeRequest {
    pub jsonrpc: String,
    pub id: u32,
    pub method: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct InitialMessage {
    pub method: String,
}
pub fn get_method(input: &str) -> String {
    let message: InitialMessage = serde_json::from_str(input.trim()).unwrap();
    message.method
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DidOpenNotification {
    pub method: String,
    pub params: DidOpenParams,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DidChangeNotification {
    pub method: String,
    pub params: DidChangeTextDocumentParams,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DidOpenParams {
    pub textDocument: TextDocumentItem
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn decode_works() {
        let test_message = InitializeRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: String::from("initialize"),
        };

        assert_eq!(test_message, decode("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"asdf\":34}}"));
    }

    #[test]  
    fn get_method_works() {
        assert_eq!("initialize", get_method("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"asdf\":34}}"));
    }
}
