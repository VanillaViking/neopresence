pub mod types;
mod logger;
pub mod stdio;

pub fn message_handler(message: &str, method: &str) {
    let response = match method {
        "initialize" => initialize(message),
        _ => None,
    };
    
    if let Some(res) = response {
        stdio::send(&res);
    }
}

fn initialize(message: &str) -> Option<String> {
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]  
    fn test() {
        assert_eq!(1, 1);
    }
}
