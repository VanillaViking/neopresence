mod types;
mod logger;
mod messages;

use std::{error, fs, io::{self, BufRead, Read, Stdin, Write}};
use std::str;
use log::{error, info, logger, warn};
use types::RequestMessage;

fn main() {

    let mut stdin = io::stdin();
    
    loop {
        if let Ok(buf) = read(&mut stdin) {
            let request_message = decode(&buf);
            message_handler(&request_message);
        }
    }
}

fn read(inp: &mut Stdin) -> Result<String, Box<dyn error::Error>> {
    let mut  handle = inp.lock();
    let mut buf = String::new();
    let mut content_length = None;
    buf.clear();
    handle.read_line(&mut buf)?;
    if buf.contains("Content-Length") {
        let (_, c_len_str) = buf.split_once(" ").ok_or("invalid header")?;
        content_length = Some(c_len_str.trim().parse::<usize>()?);
    }

    let mut buf = buf.into_bytes();
    buf.resize(content_length.ok_or("err")?, 0);

    handle.read_exact(&mut buf)?;
    let buf = String::from_utf8(buf)?;
    Ok(buf)
}

fn decode(message: &str) -> RequestMessage {
    logger::log(message.trim(), logger::MessageType::Error);
    serde_json::from_str(message.trim()).unwrap()
}

fn send(message: &str) {
    print!("Content-Length: {}\r\n\r\n", message.len());
    print!("{message}\r\n\r\n");
    io::stdout().flush().expect("unable to flush");
}

fn message_handler(message: &RequestMessage) {
    let response = match message.method.as_str() {
        "initialize" => messages::initialize(message),
        _ => None,
    };
    
    if let Some(res) = response {
        send(&res);
    }
}
