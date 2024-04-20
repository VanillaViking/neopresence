use std::{error, fs, io::{self, BufRead, Read, Stdin, Write}};
use nvim_discord_rich_presence::{stdio, types::get_method};

fn main() {

    let mut stdin = io::stdin();
    
    loop {
        if let Some(message) = stdio::read(&mut stdin).unwrap() {
            let method = get_method(&message);
            nvim_discord_rich_presence::message_handler(&message, &method);
        }
    }
}
