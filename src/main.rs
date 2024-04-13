mod types;
mod logger;
mod messages;
mod stdio;

#[cfg(tests)]
mod tests;

use std::{error, fs, io::{self, BufRead, Read, Stdin, Write}};
use std::str;
use types::RequestMessage;

fn main() {

    let mut stdin = io::stdin();
    
    loop {
        if let Ok(buf) = stdio::read(&mut stdin) {
            let request_message = RequestMessage::decode(&buf);
            message_handler(&request_message);
        }
    }
}

fn message_handler(message: &RequestMessage) {
    let response = match message.method.as_str() {
        "initialize" => messages::initialize(message),
        _ => None,
    };
    
    if let Some(res) = response {
        stdio::send(&res);
    }
}
