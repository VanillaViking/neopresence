mod types;

use std::{fs, io::{self, BufRead, Read}};
use std::str;
use log::{error, info, warn};
use types::RequestMessage;

fn main() {

    fs::write("/home/vanilla/projects/rust/nvim-discord-rich-presence/log", "hello ").unwrap();

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buf = String::new();
    let mut msg_buf = [0_u8; 4000];

    if let Err(e) = handle.read_line(&mut buf) {
        fs::write("/home/vanilla/projects/rust/nvim-discord-rich-presence/log", format!("could not read line {e}")).unwrap();
    }

    if buf.contains("Content-Length") {
        let (_, c_len_str) = buf.split_once(" ").unwrap();
        let content_length: u32 = c_len_str.trim().parse().unwrap();
        fs::write("/home/vanilla/projects/rust/nvim-discord-rich-presence/log", format!("{content_length}")).unwrap();
        if let Err(e) = handle.read(&mut msg_buf) {
            fs::write("/home/vanilla/projects/rust/nvim-discord-rich-presence/log", format!("could not read contents {e}")).unwrap();
        }
        let request = decode(str::from_utf8(&msg_buf).unwrap());
        fs::write("/home/vanilla/projects/rust/nvim-discord-rich-presence/log", request.method).unwrap();
    }
    
}

fn decode(message: &str) -> RequestMessage {
    serde_json::from_str(message).unwrap()
}
