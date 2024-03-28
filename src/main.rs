mod types;
mod logger;

use std::{error, fs, io::{self, BufRead, Read, Stdin}};
use std::str;
use log::{error, info, logger, warn};
use types::RequestMessage;

fn main() {

    // fs::write("/home/vanilla/projects/rust/nvim-discord-rich-presence/log", "hello ").unwrap();

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buf = String::new();
    let mut msg_buf = [0_u8; 4000];

    let buf = read(handle);

    if let Err(e) = handle.read_line(&mut buf) {
        // fs::write("/home/vanilla/projects/rust/nvim-discord-rich-presence/log", format!("could not read line {e}")).unwrap();
    }

    if buf.contains("Content-Length") {
        let (_, c_len_str) = buf.split_once(" ").unwrap();
        let content_length: u32 = c_len_str.trim().parse().unwrap();
        // fs::write("/home/vanilla/projects/rust/nvim-discord-rich-presence/log", format!("{content_length}")).unwrap();
        if let Err(e) = handle.read(&mut msg_buf) {
            // fs::write("/home/vanilla/projects/rust/nvim-discord-rich-presence/log", format!("could not read contents {e}")).unwrap();
        }
        let request = decode(str::from_utf8(&msg_buf).unwrap());
        // fs::write("/home/vanilla/projects/rust/nvim-discord-rich-presence/log", request.method).unwrap();
        logger::log(&request.method, logger::MessageType::Error);

    }
    
}

fn read(inp: &mut Stdin) -> Result<String, Box<dyn error::Error>> {
    let mut handle = inp.lock();
    let mut buf = String::new();
    let mut content_length = None;

    buf.clear();
    inp.read_line(&mut buf)?;
    if buf.contains("Content-Length") {
        let (_, c_len_str) = buf.split_once(" ").ok_or("invalid header")?;
        content_length = Some(c_len_str.trim().parse::<usize>()?);
    }

    let mut buf = buf.into_bytes();
    buf.resize(content_length.ok_or("err")?, 0);

    inp.read_exact(&mut buf)?;
    let buf = String::from_utf8(buf)?;
    Ok(buf)
}

fn decode(message: &str) -> RequestMessage {
    logger::log(message.trim(), logger::MessageType::Error);
    serde_json::from_str(message.trim()).unwrap()
}

fn send(message: &str) {
    print!("Content-Length: {}\r\n\r\n", message.len());
    print!("{message}");
}

// fn encode() -> String {
//     serde_json::to_string(message).unwrap()
// }
