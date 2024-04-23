use std::io::{self};
use nvim_discord_rich_presence::{stdio, types::Context};


fn main() {
    let mut stdin = io::stdin();
    let mut context = Context::new(1231109585633284168);
    
    loop {
        if let Some(message) = stdio::read(&mut stdin).unwrap() {
            nvim_discord_rich_presence::message_handler(&message, &mut context);
        }
    }
}
