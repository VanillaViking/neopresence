use std::process::exit;

use discord_presence::models::ActivityTimestamps;
use logger::ghetto_log;
use lsp_types::{InitializeResult, PositionEncodingKind, SaveOptions, ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions};

use types::{get_method, Context, DidChangeNotification, DidOpenNotification};

use crate::types::{InitializeRequest, Response};

pub mod types;
mod logger;
pub mod stdio;

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
            ghetto_log("received shutdown");
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

    // TODO: do this better, maybe set activity to "Idling"
    let mut additions = 0;
    let mut deletions = 0;

    for file_data in context.changed_files.values() {
        let (del, add) = get_diff(&file_data.original_contents, &file_data.latest_contents);
        additions += add;
        deletions += del;
    }

    // Set the activity
    context.drpc.set_activity(|act| {
        act.state(format!("{} additions, {} deletetions in {} files", additions, deletions, context.changed_files.len()))
            .timestamps(|_| {
                ActivityTimestamps::new().start(context.start_time)
            })
            .details(format!("Editing {}", filename))
            .assets(|ass| {
                ass.large_image("nvim")
            })
    })
    .expect("Failed to set activity");
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
        dbg!(nl, ol);
        ol == nl
    }).is_none()).count();
    (deletions as u32, additions as u32)
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
