use std::{io, sync::mpsc, thread};
use nvim_discord_rich_presence::{discord_runner, stdio, types::Context};


fn main() {
    let mut stdin = io::stdin();
    let (discord_tx, discord_rx) = mpsc::channel();
    let mut context = Context::new(discord_tx);

    thread::spawn(move || {
        discord_runner(1231109585633284168, discord_rx);
    });
    
    loop {
        if let Some(message) = stdio::read(&mut stdin).unwrap() {
            nvim_discord_rich_presence::message_handler(&message, &mut context);
        }
    }
}
