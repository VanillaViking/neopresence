use std::time::{Duration, SystemTime};

use discord_presence::{
    models::{ActivityTimestamps, EventData},
    Client,
};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::logger;

pub struct DiscordData {
    pub additions: u32,
    pub deletions: u32,
    pub num_files: u32,
    pub filename: Option<String>,
    pub remote_url: Option<String>,
    pub start_time: u64,
}

pub enum DiscordMessage {
    DiscordError(String),
    StateUpdate(DiscordData),
}

pub fn discord_init(discord_client_id: u64, tx: Sender<DiscordMessage>) -> Client {
    let mut drpc = Client::new(discord_client_id);
    let tx2 = tx.clone();

    drpc.on_ready(move |_ctx| {
        // let _ = tx.send(SessionEvent::DiscordReady);
    })
    .persist();
    drpc.on_error(move |err| {
        if let EventData::Error(err) = err.event {
            let msg = err.message.unwrap_or_default();
            let _ = tx2.send(DiscordMessage::DiscordError(msg));
        }
    })
    .persist();
    drpc.start();

    return drpc;
}

pub async fn discord_runner(drpc: &mut Client, mut rx: Receiver<DiscordMessage>) {
    loop {
        match rx.recv().await {
            Some(DiscordMessage::StateUpdate(data)) => {
                update_discord_status(data, drpc);
            }
            Some(DiscordMessage::DiscordError(_e)) => {
                tokio::time::sleep(Duration::from_secs(5)).await;
                drpc.start();
            }
            None => break,
        }
    }
}

fn update_discord_status(data: DiscordData, drpc: &mut Client) {
    // Set the activity
    let details = match data.filename {
        Some(name) => format!("Editing {}", name),
        None => "Idling".to_string(),
    };
    // TODO: handle this result
    let _ = drpc.set_activity(|act| {
        act.state(format!(
            "{} additions, {} deletions in {} files",
            data.additions, data.deletions, data.num_files
        ))
        .timestamps(|_| ActivityTimestamps::new().start(data.start_time))
        .details(details)
        .assets(|ass| ass.large_image("nvim"))
    });
}
