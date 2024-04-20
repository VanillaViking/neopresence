use std::io::Stdin;
use std::{error, io::{self, BufRead, Read, Write}};

use serde::Serialize;

use crate::logger::{self, ghetto_log};


pub fn read(inp: &mut Stdin) -> io::Result<Option<String>> {
    // let mut buf = String::new();
    // let mut content_length = None;
    // buf.clear();
    // inp.read_line(&mut buf)?;
    // ghetto_log(&buf);
    // if buf.contains("Content-Length") {
    //     let (_, c_len_str) = buf.split_once(" ").ok_or("invalid header")?;
    //     content_length = Some(c_len_str.trim().parse::<usize>()?);
    // }

    // let mut buf = buf.into_bytes();
    // buf.resize(content_length.ok_or("err")? + 2, 0);

    // inp.read_exact(&mut buf)?;
    // let buf = String::from_utf8(buf)?;
    // Ok(buf)

    // copied from rust analyzer
    fn invalid_data(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, error)
    }
    
    let mut size = None;
    let mut buf = String::new();
    loop {
        buf.clear();
        if inp.read_line(&mut buf)? == 0 {
            return Ok(None);
        }
        if !buf.ends_with("\r\n") {
            return Err(invalid_data("malformed header"));
        }
        let buf = &buf[..buf.len() - 2];
        if buf.is_empty() {
            break;
        }
        let mut parts = buf.splitn(2, ": ");
        let header_name = parts.next().unwrap();
        let header_value =
            parts.next().ok_or_else(|| invalid_data(format!("malformed header: {:?}", buf)))?;
        if header_name.eq_ignore_ascii_case("Content-Length") {
            size = Some(header_value.parse::<usize>().map_err(invalid_data)?);
        }
    }
    let size: usize = size.ok_or_else(|| invalid_data("no Content-Length".to_owned()))?;
    ghetto_log(&format!("{size}"));
    let mut buf = buf.into_bytes();
    buf.resize(size, 0);
    inp.read_exact(&mut buf)?;
    let buf = String::from_utf8(buf).map_err(invalid_data)?;
    Ok(Some(buf))
}

//TODO: change to write_all
pub fn send(message: &str) {
    print!("Content-Length: {}\r\n\r\n", message.len());
    print!("{message}\r\n\r\n");
    if let Err(e) = io::stdout().flush() {
        ghetto_log(&e.to_string());
    }
}
