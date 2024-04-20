use std::{error, fs, io::{self, BufRead, Read, Stdin, Write}};
use discord_presence::{Client, Event};
use nvim_discord_rich_presence::{stdio, types::get_method};

fn main() {

    let mut stdin = io::stdin();
    let mut drpc = Client::new(1231109585633284168);
    drpc.on_ready(|_ctx| {
        println!("ready?");
    })
    .persist();

    drpc.start();

    drpc.block_until_event(Event::Ready).unwrap();

    assert!(Client::is_ready());
    
    loop {
        if let Some(message) = stdio::read(&mut stdin).unwrap() {
            let method = get_method(&message);
            nvim_discord_rich_presence::message_handler(&message, &method, &mut drpc);
        }
    }
}
