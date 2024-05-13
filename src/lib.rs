use core::time;
use std::{error::Error, process::{exit, Command}, sync::mpsc::Receiver, thread, time::{SystemTime, UNIX_EPOCH}};

use discord_presence::{models::{ActivityTimestamps, EventData}, Client, Event};
use logger::ghetto_log;
use lsp_types::{InitializeResult, PositionEncodingKind, SaveOptions, ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions};

use types::{get_method, Context, DidChangeNotification, DidOpenNotification, DiscordData};

use crate::types::{InitializeRequest, Response};

pub mod types;
mod logger;
pub mod stdio;


pub fn discord_runner(discord_client_id: u64, rx: Receiver<DiscordData>) {
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


        loop {
            let mut discord_data = None;
            while let Ok(d) = rx.recv_timeout(time::Duration::from_secs(5)) {
                discord_data = Some(d);
            }

            if let Some(data) = discord_data {
                // Set the activity
                let details = match data.filename {
                    Some(name) => format!("Editing {}", name),
                    None => "Idling".to_string(),
                };
                drpc.set_activity(|act| {
                    act.state(format!("{} additions, {} deletetions in {} files", data.additions, data.deletions, data.num_files))
                        .timestamps(|_| {
                            ActivityTimestamps::new().start(start_time)
                        })
                    .details(details)
                        .assets(|ass| {
                            ass.large_image("nvim")
                        })
                    .append_buttons(|mut button| {
                        if let Some(url) = data.remote_url {
                            button = button.label("Repository Link").url(url);
                        }
                        button
                    })
                })
                .expect("Failed to set activity");
                thread::sleep(time::Duration::from_secs(5));
            }
        }


}

pub fn message_handler(message: &str, context: &mut Context) {
    let response = match get_method(message).as_str() {
        "initialize" => {
            serde_json::to_string(&initialize(message)).ok()
        },
        "textDocument/didOpen" => {
            did_open(message, context);
            None
        },
        "textDocument/didChange" => {
            did_change(message, context);
            None
        }
        "shutdown" => {
            // ghetto_log("received shutdown");
            exit(0);
        },
        _ => None,
    };
    
    if let Some(res) = response {
        // logger::log(&res, logger::MessageType::Error);
        stdio::send(&res);
    }
}

fn initialize(message: &str) -> Response {
    let request: InitializeRequest = types::decode(message.trim());
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    const NAME: &str = env!("CARGO_PKG_NAME");

    Response {
        id: request.id,
        result: Some(serde_json::to_value(InitializeResult {
                server_info: Some(ServerInfo {
                    name: NAME.to_string(),
                    version: Some(VERSION.to_string()),
                }),
                capabilities: ServerCapabilities {

                    text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        will_save: None,
                        will_save_wait_until: None,
                        save: Some(SaveOptions::default().into()),
                    })),
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
                }
            }).unwrap())
    }
}

fn did_open(message: &str, context: &mut Context) {
    let notification: DidOpenNotification = match serde_json::from_str(message) {
        Ok(notif) => notif,
        Err(e) => {
            ghetto_log(&e.to_string());
            exit(1);
        },
    };
    let filename = get_file_name(&notification.params.textDocument.uri).unwrap_or("unknown");

    let blacklist = vec!["cmp_docs", "TelescopeResults", "TelescopePrompt", "cmp_menu"];

    if let Some(_) = blacklist.iter().find(|n| n == &&notification.params.textDocument.language_id) {
        return
    }

    if filename == "" {
        return;
    }

    context.current_file = Some(filename.to_owned()); 
    context.send_discord();

}

fn did_change(message: &str, context: &mut Context) {
    let notification: DidChangeNotification = match serde_json::from_str(message) {
        Ok(notif) => notif,
        Err(e) => {
            ghetto_log(&e.to_string());
            exit(1);
        },
    };
    let filename = get_file_name(&notification.params.text_document.uri).unwrap_or("");

    // let blacklist = vec!["cmp_docs", "TelescopeResults", "TelescopePrompt", "cmp_menu"];
    // if let Some(_) = blacklist.iter().find(|n| n == &&notification.params.textDocument.language_id) {
    //     return
    // }

    if filename == "" {
        return;
    }

    let _ = context.update_file_contents(filename, &notification.params.content_changes[0].text);
}

fn get_file_name(uri: &lsp_types::Url) -> Option<&str> {
    Some(uri.path().split("/").last()?)
}

fn get_diff(old: &str, new: &str) -> (u32, u32) {
    let mut old_lines: Vec<&str> = old.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    // TODO: myers diff algorithm
    
    // present in old but not in new
    let deletions = old_lines.iter().filter(|ol| new_lines.iter().find(|nl| nl == ol).is_none()).count();

    let additions = new_lines.iter().filter(|nl| old_lines.iter().find(|ol| {
        ol == nl
    }).is_none()).count();
    (deletions as u32, additions as u32)
}

fn get_remote_url() -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .arg("config")
        .arg("--get")
        .arg("remote.origin.url")
        .output()?;

    let raw_url = String::from_utf8(output.stdout.as_slice().to_owned())?;

    // means it is a github ssh url (probably)
    if raw_url.contains("@") {
        let (_, trunc_url) = raw_url.split_once("@").unwrap();
        let mut url = trunc_url.replace(":", "/");
        url = url.replace(".git", "");
        return Ok(format!("https://{}", url))
    }

    Ok(raw_url)
}

#[cfg(test)]
mod tests {
    use lsp_types::Url;

    use crate::{get_diff, get_file_name};

    #[test]
    fn get_file_name_works() {
        let uri = Url::parse("file:///home/vanilla/projects/nix-vim/README.md").unwrap();
        assert_eq!("README.md", get_file_name(&uri).unwrap());
    }

    #[test]
    fn diff_works() {
        let file1 = "the
quick
brown
fox";

        let file2 = "teh
quick
brown
fox
jumps";
        assert_eq!((1, 2), get_diff(file1, file2));
    }
}
