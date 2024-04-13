use std::io::Stdin;
use std::{error, io::{self, BufRead, Read, Write}};


pub fn read(inp: &mut Stdin) -> Result<String, Box<dyn error::Error>> {
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
    buf.resize(content_length.ok_or("err")? + 2, 0);

    handle.read_exact(&mut buf)?;
    let buf = String::from_utf8(buf)?;
    Ok(buf)
}

pub fn send(message: &str) {
    print!("Content-Length: {}\r\n\r\n", message.len());
    print!("{message}\r\n\r\n");
    io::stdout().flush().expect("unable to flush");
}
