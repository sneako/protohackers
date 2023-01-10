use log::info;
use std::collections::HashMap;
use std::io;
use tokio::net::UdpSocket;

#[derive(Debug, PartialEq)]
enum Request {
    Insert(String, String),
    Query(String),
    Clear,
    Version,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    let mut buffer: Vec<u8> = vec![0; 1024];
    let mut db: HashMap<String, String> = HashMap::new();
    let addr = "fly-global-services:7878";
    // let addr = "0.0.0.0:7878";
    let socket = UdpSocket::bind(addr).await?;
    info!("Listening on: {}", socket.local_addr()?);

    loop {
        let (size, client_addr) = socket.recv_from(&mut buffer).await?;
        info!("recv {} bytes from {:?}", size, client_addr);

        match handle_request(&mut db, &buffer[..size]).await {
            Some(string) => {
                info!("response: {}", string);
                socket.send_to(string.as_bytes(), &client_addr).await?
            }
            None => 0 as usize,
        };
    }
}

async fn handle_request(db: &mut HashMap<String, String>, buffer: &[u8]) -> Option<String> {
    match parse_request(&buffer) {
        Some(Request::Version) => Some("version=SneakoDB-1.0".to_string()),
        Some(Request::Query(key)) => Some(query(db, &key)),
        Some(Request::Insert(key, value)) => {
            db.insert(key, value);
            None
        }
        Some(Request::Clear) => {
            db.clear();
            None
        }
        None => None,
    }
}

fn query(db: &HashMap<String, String>, key: &String) -> String {
    match db.get(key) {
        Some(value) => format!("{}={}", key, value),
        None => format!("{}=", key),
    }
}

fn parse_request(buffer: &[u8]) -> Option<Request> {
    let input = String::from_utf8_lossy(buffer);
    info!("request: {}", input);
    match &input.split_once("=") {
        Some(("version", _)) => None,
        Some((key, value)) => Some(Request::Insert(key.to_string(), value.to_string())),
        None => match input.trim() {
            "version" => Some(Request::Version),
            "***CLEAR***" => Some(Request::Clear),
            key => Some(Request::Query(key.to_string())),
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse_request, Request};

    #[test]
    fn parse_request_test() {
        let one = "version".as_bytes();
        let expected_one = Request::Version;
        assert_eq!(expected_one, parse_request(&one).unwrap())
    }
}
