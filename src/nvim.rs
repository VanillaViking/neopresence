use std::io::Stdin;
use std::{
    error,
    io::{self, BufRead, Read, Write},
};

use log::logger;
use lsp_types::{DidChangeTextDocumentParams, InitializeResult, PositionEncodingKind, SaveOptions, ServerCapabilities, ServerInfo, TextDocumentItem, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;

use crate::logger::{self, ghetto_log};

pub enum NvimMessage {
    NvimError(String),
    FileOpened { filename: String },
    FileChanged { filename: String, contents: String },
    Shutdown
}

pub fn get_method(input: &str) -> String {
    let message: InitialMessage = serde_json::from_str(input.trim()).unwrap();
    message.method
}

pub async fn message_handler(message: &str, nvim_tx: Sender<NvimMessage>) {
    let response = match get_method(message).as_str() {
        "initialize" => serde_json::to_string(&initialize(message)).ok(),
        "textDocument/didOpen" => {
            did_open(message, nvim_tx).await;
            None
        }
        "textDocument/didChange" => {
            did_change(message, nvim_tx).await;
            None
        }
        "shutdown" => {
            nvim_tx.send(NvimMessage::Shutdown).await.expect("channel to be open");
            None
        }
        _ => None,
    };

    if let Some(res) = response {
        // logger::log(&res, logger::MessageType::Error);
        send(&res);
    }
}

fn initialize(message: &str) -> Response {
    let request: InitializeRequest = decode(message.trim());
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const NAME: &str = env!("CARGO_PKG_NAME");

    Response {
        id: request.id,
        result: Some(
            serde_json::to_value(InitializeResult {
                server_info: Some(ServerInfo {
                    name: NAME.to_string(),
                    version: Some(VERSION.to_string()),
                }),
                capabilities: ServerCapabilities {
                    text_document_sync: Some(TextDocumentSyncCapability::Options(
                        TextDocumentSyncOptions {
                            open_close: Some(true),
                            change: Some(TextDocumentSyncKind::FULL),
                            will_save: None,
                            will_save_wait_until: None,
                            save: Some(SaveOptions::default().into()),
                        },
                    )),
                    position_encoding: Some(PositionEncodingKind::UTF8),
                    selection_range_provider: None,
                    hover_provider: None,
                    completion_provider: None,
                    signature_help_provider: None,
                    definition_provider: None,
                    type_definition_provider: None,
                    implementation_provider: None,
                    references_provider: None,
                    document_highlight_provider: None,
                    document_symbol_provider: None,
                    workspace_symbol_provider: None,
                    code_action_provider: None,
                    code_lens_provider: None,
                    document_formatting_provider: None,
                    document_range_formatting_provider: None,
                    document_on_type_formatting_provider: None,
                    rename_provider: None,
                    document_link_provider: None,
                    color_provider: None,
                    folding_range_provider: None,
                    declaration_provider: None,
                    execute_command_provider: None,
                    workspace: None,
                    call_hierarchy_provider: None,
                    semantic_tokens_provider: None,
                    moniker_provider: None,
                    linked_editing_range_provider: None,
                    inline_value_provider: None,
                    inlay_hint_provider: None,
                    diagnostic_provider: None,
                    experimental: None,
                },
            })
            .unwrap(),
        ),
    }
}

async fn did_open(message: &str, nvim_tx: Sender<NvimMessage>) {
    let notification: DidOpenNotification = match serde_json::from_str(message) {
        Ok(notif) => notif,
        Err(e) => {
            // exit(1);
            return
        }
    };
    let filename = get_file_name(&notification.params.textDocument.uri).unwrap_or("unknown");

    let blacklist = vec![
        "cmp_docs",
        "TelescopeResults",
        "TelescopePrompt",
        "cmp_menu",
    ];

    if let Some(_) = blacklist
        .iter()
        .find(|n| n == &&notification.params.textDocument.language_id)
    {
        return;
    }

    if filename == "" {
        return;
    }

    nvim_tx.send(NvimMessage::FileOpened { filename: filename.to_owned() }).await.expect("channel to be open");
}

async fn did_change(message: &str, nvim_tx: Sender<NvimMessage>) {
    let notification: DidChangeNotification = match serde_json::from_str(message) {
        Ok(notif) => notif,
        Err(e) => {
            logger::log(&e.to_string(), logger::MessageType::Error);
            return;
        }
    };
    let filename = get_file_name(&notification.params.text_document.uri).unwrap_or("");

    if filename == "" {
        return;
    }


    nvim_tx.send(NvimMessage::FileChanged { filename: filename.to_owned(), contents: notification.params.content_changes[0].text.clone() }).await.expect("channel to be open");
}

fn get_file_name(uri: &lsp_types::Url) -> Option<&str> {
    Some(uri.path().split("/").last()?)
}

pub fn read(inp: &mut Stdin) -> io::Result<Option<String>> {
    // copied from rust analyzer
    fn invalid_data(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, error)
    }

    let mut size = None;
    let mut buf = String::new();
    loop {
        buf.clear();
        if inp.read_line(&mut buf)? == 0 {
            return Ok(None);
        }
        if !buf.ends_with("\r\n") {
            return Err(invalid_data(format!("malformed header: {:?}", buf)));
        }
        let buf = &buf[..buf.len() - 2];
        if buf.is_empty() {
            break;
        }
        let mut parts = buf.splitn(2, ": ");
        let header_name = parts.next().unwrap();
        let header_value = parts
            .next()
            .ok_or_else(|| invalid_data(format!("malformed header: {:?}", buf)))?;
        if header_name.eq_ignore_ascii_case("Content-Length") {
            size = Some(header_value.parse::<usize>().map_err(invalid_data)?);
        }
    }
    let size: usize = size.ok_or_else(|| invalid_data("no Content-Length".to_owned()))?;
    let mut buf = buf.into_bytes();
    buf.resize(size, 0);
    inp.read_exact(&mut buf)?;
    let buf = String::from_utf8(buf).map_err(invalid_data)?;
    Ok(Some(buf))
}

//TODO: change to write_all
pub fn send(message: &str) {
    print!("Content-Length: {}\r\n\r\n", message.len());
    print!("{message}");
    if let Err(e) = io::stdout().flush() {
        // ghetto_log(&e.to_string());
    }
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
    pub textDocument: TextDocumentItem,
}
