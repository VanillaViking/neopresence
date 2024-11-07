use std::{collections::HashMap, sync::mpsc::Sender};

use lsp_types::{DidChangeTextDocumentParams, TextDocumentItem};
use serde::{Deserialize, Serialize};

use crate::{diff::get_diff, get_remote_url};

pub struct Context {
    pub changed_files: HashMap<String, FileData>,
    pub discord_tx: Sender<DiscordData>,
    pub current_file: Option<String>,
    pub remote_url: Option<String>,
}
impl Context {
    pub fn new(discord_tx: Sender<DiscordData>) -> Self {

        return Self {
            changed_files: HashMap::new(),
            discord_tx,
            current_file: None,
            remote_url: get_remote_url().ok(),
        }
    }

    
    pub fn update_file_contents(&mut self, filename: &str, new_contents: &str) -> Result<(), &str> {
        if filename == "" {
            return Err("no filename")
        }

        let file_data = self.changed_files.entry(filename.to_string()).or_insert(FileData { original_contents: new_contents.to_owned(), latest_contents: String::from("")});

        file_data.latest_contents = new_contents.to_string();

        self.send_discord();
        Ok(())
    }

    pub fn send_discord(&self) {
        // TODO: do this better, maybe set activity to "Idling"
        let mut additions = 0;
        let mut deletions = 0;

        for file_data in self.changed_files.values() {
            let (del, add) = get_diff(&file_data.original_contents, &file_data.latest_contents);
            additions += add;
            deletions += del;
        }

        let data = DiscordData {
            additions,
            deletions,
            num_files: self.changed_files.len() as u32,
            filename: self.current_file.to_owned(),
            remote_url: self.remote_url.to_owned(),
        };

        self.discord_tx.send(data);
    }
}

pub struct DiscordData {
    pub additions: u32,
    pub deletions: u32,
    pub num_files: u32,
    pub filename: Option<String>,
    pub remote_url: Option<String>,
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
