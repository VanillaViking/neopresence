use std::{collections::HashMap, io, process::{exit, Command}, time::{Duration, SystemTime, UNIX_EPOCH}};
use diff::get_diff;
use discord::{discord_init, discord_runner, DiscordData, DiscordMessage};
use logger::ghetto_log;
use nvim::NvimMessage;
use tokio::{
    sync::
        mpsc::{self}
    ,
    time::sleep,
};
use anyhow::{bail, Result};

mod discord;
mod nvim;
mod logger;
mod diff;

#[tokio::main]
async fn main() -> Result<()> {
    let (discord_tx, discord_rx) = mpsc::channel(16);
    let discord_tx2 = discord_tx.clone();
    let (nvim_tx, mut nvim_rx) = mpsc::channel(16);
    let (update_clock_tx, mut update_clock_rx) = mpsc::channel(16);
    let mut drpc = discord_init(1231109585633284168, discord_tx);
    let mut state = SessionState::new();

    tokio::spawn(async move {
        discord_runner(&mut drpc, discord_rx).await;
    });

    tokio::spawn(async move {
        let mut stdin = io::stdin();
        loop {
            if let Some(message) = nvim::read(&mut stdin).unwrap() {
                nvim::message_handler(&message, nvim_tx.clone()).await;
            }
        }
    });

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(5)).await;
            update_clock_tx.send(ClockMessage::Update).await.expect("channel to be open");
        }
    });
    
    loop {
        tokio::select! {
            Some(nvim_msg) = nvim_rx.recv() => {
                match nvim_msg {
                    NvimMessage::NvimError(_) => todo!(),
                    NvimMessage::FileOpened { filename } => state.current_file = Some(filename),
                    NvimMessage::FileChanged { filename, contents } => update_file_contents(&mut state, filename, contents),
                    NvimMessage::Shutdown => exit(0),
                }
            }
            Some(ClockMessage::Update) = update_clock_rx.recv() => {
                discord_tx2.send(DiscordMessage::StateUpdate(construct_data(&state))).await?;
            }
        }
    }

}


pub fn construct_data(state: &SessionState) -> DiscordData {
    // TODO: do this better, maybe set activity to "Idling"
    let mut additions = 0;
    let mut deletions = 0;

    for file_data in state.changed_files.values() {
        let (del, add) = get_diff(&file_data.original_contents, &file_data.latest_contents);
        additions += add;
        deletions += del;
    }


    DiscordData {
        additions,
        deletions,
        num_files: state.changed_files.len() as u32,
        filename: state.current_file.to_owned(),
        remote_url: state.remote_url.to_owned(),
        start_time: state.start_time,
    }
}

fn update_file_contents(
    state: &mut SessionState,
    filename: String,
    new_contents: String,
) {
    if filename == "" {
        return
    }
    
    state.current_file = Some(filename.clone());

    let file_data = state.changed_files.entry(filename).or_insert(FileData {
        original_contents: new_contents.to_owned(),
        latest_contents: String::from(""),
    });

    file_data.latest_contents = new_contents;
}


fn get_remote_url() -> Result<String> {
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
        return Ok(format!("https://{}", url));
    }

    Ok(raw_url)
}


fn clamp(mut str: String, len: usize) -> String {
    const ELLIPSES_LEN: usize = 3;
    if str.len() > len {
        while str.len() > (len - ELLIPSES_LEN) {
            str.pop();
        }
        str.push_str("...");
    }
    str
}

pub struct SessionState {
    pub changed_files: HashMap<String, FileData>,
    pub current_file: Option<String>,
    pub remote_url: Option<String>,
    pub start_time: u64,
}

impl SessionState {
    pub fn new() -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get system time")
            .as_secs();
        let changed_files = HashMap::new();

        Self {
            changed_files,
            current_file: None,
            remote_url: None,
            start_time,
        }
    }
}

pub struct FileData {
    pub original_contents: String,
    pub latest_contents: String,
}

pub enum ClockMessage {
    Update,
}

