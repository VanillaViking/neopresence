use serde::{Deserialize, Serialize};

fn decode<'a, T: Deserialize<'a>>(input: &'a str) -> T {
    serde_json::from_str(input.trim()).unwrap()
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct InitializeRequest {
    pub jsonrpc: String,
    pub id: u32,
    pub method: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct InitialMessage {
    pub method: String,
}
pub fn get_method(input: &str) -> String {
    let message: InitialMessage = serde_json::from_str(input.trim()).unwrap();
    message.method
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn decode_works() {
        let test_message = InitializeRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: String::from("initialize"),
        };

        assert_eq!(test_message, decode("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"asdf\":34}}"));
    }

    #[test]  
    fn get_method_works() {
        assert_eq!("initialize", get_method("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"asdf\":34}}"));
    }
}
