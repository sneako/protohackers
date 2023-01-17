use fancy_regex::Regex;
use lazy_static::lazy_static;
use log::{error, info};
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const UPSTREAM: &str = "chat.protohackers.com:16963";
// const UPSTREAM: &str = "localhost:7879";
const TONY_ADDR: &str = "7YWHMfk9JZe0LM0g1ZauHuiSxhI";

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    let listener = TcpListener::bind("0.0.0.0:7878").await?;

    loop {
        let (stream, address) = listener.accept().await?;
        info!("accept from {:?}", address);
        tokio::spawn(async move { handle_stream(stream).await });
    }
}

async fn handle_stream(mut downstream: TcpStream) -> io::Result<()> {
    let (mut downstream_rx, mut downstream_tx) = downstream.split();
    let mut upstream = TcpStream::connect(UPSTREAM).await?;
    let (mut upstream_rx, mut upstream_tx) = upstream.split();
    let mut upstream_buf: Vec<u8> = vec![0; 4096];
    let mut downstream_buf: Vec<u8> = vec![0; 4096];

    // proxy the welcome message from the upstream
    match upstream_rx.read(&mut upstream_buf).await {
        Ok(num_bytes) => {
            info!("welcome");
            info!("upstream -> downstream: welcome msg, {} bytes", num_bytes);
            downstream_tx
                .write_all(&mut upstream_buf[..num_bytes])
                .await?
        }
        error => error!("failed to read welcome message: {:?}", error),
    }

    // proxy the name
    match downstream_rx.read(&mut downstream_buf).await {
        Ok(num_bytes) => {
            if num_bytes > 0 {
                info!("downstream -> upstream: name, {} bytes", num_bytes);
                upstream_tx
                    .write_all(&mut downstream_buf[..num_bytes])
                    .await?
            }
        }
        _ => {
            error!("failed reading the name!");
        }
    }

    // proxy the room contains message
    match upstream_rx.read(&mut upstream_buf).await {
        Ok(num_bytes) => {
            if num_bytes > 0 {
                info!("joined");
                info!(
                    "upstream -> downstream: room contains msg, {} bytes",
                    num_bytes
                );
                downstream_tx
                    .write_all(&mut upstream_buf[..num_bytes])
                    .await?
            }
        }
        error => error!("failed reading room contains message: {:?}", error),
    }

    let mut downstream_msg_buf: Vec<u8> = Vec::new();
    // chat loop
    loop {
        tokio::select! {
            // listen for downstream
            Ok(num_bytes) = downstream_rx.read(&mut downstream_buf) => {
                if num_bytes > 0 {
                    if downstream_buf[..num_bytes].contains(&b'\n') {
                        if downstream_msg_buf.len() > 0 {
                            let string = String::from_utf8_lossy(&downstream_msg_buf);
                            let updated = replace_coin_addrs(string.to_string());
                            upstream_tx.write_all(&updated.as_bytes()).await?;
                            downstream_msg_buf.clear()
                        }
                        info!("upstream <- downstream: message {} bytes", num_bytes);
                        let string = String::from_utf8_lossy(&downstream_buf[..num_bytes]);
                        let updated = replace_coin_addrs(string.to_string());
                        upstream_tx.write_all(updated.as_bytes()).await?
                    } else {
                        downstream_msg_buf.extend_from_slice(&downstream_buf[..num_bytes])
                    }
                } else {
                    break;
                }
            }
            // listen for upstream
            Ok(num_bytes) = upstream_rx.read(&mut upstream_buf) => {
                if num_bytes > 0 {
                    info!("upstream -> downstream: replies, {} bytes", num_bytes);
                    let string = String::from_utf8_lossy(&upstream_buf[..num_bytes]);
                    let updated = replace_coin_addrs(string.to_string());
                    downstream_tx
                        .write_all(updated.as_bytes())
                        .await?
                }
            }
        }
    }
    info!("client disconnected");
    io::Result::Ok(())
}

fn replace_coin_addrs(message: String) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new("\\b(7[a-zA-Z0-9]{25,34})(?![\\w-])").unwrap();
    }

    RE.captures_iter(&message.to_string()).fold(
        message,
        |response, captures_res| match captures_res {
            Ok(captures) => captures
                .iter()
                .fold(response, |acc, capture| match capture {
                    Some(cap) => acc.replace(cap.as_str(), TONY_ADDR),
                    None => acc,
                }),
            _ => response,
        },
    )
}

#[cfg(test)]
mod tests {
    use crate::replace_coin_addrs;

    #[test]
    fn replace_coin_addr_test() {
        let input = "".to_string();
        let expected = "".to_string();
        assert_eq!(expected, replace_coin_addrs(input));

        let input =
            "Hi alice, please send payment to 7iKDZEwPZSqIvDnHvVN2r0hUWXD5rHX\n".to_string();
        let expected = "Hi alice, please send payment to 7YWHMfk9JZe0LM0g1ZauHuiSxhI\n".to_string();
        assert_eq!(expected, replace_coin_addrs(input));

        // with period
        let input = "Hi alice, please send payment to 7iKDZEwPZSqIvDnHvVN2r0hUWXD5rHX.".to_string();
        let expected = "Hi alice, please send payment to 7YWHMfk9JZe0LM0g1ZauHuiSxhI.".to_string();
        assert_eq!(expected, replace_coin_addrs(input));

        let input = "please send 750 coins to 7iKDZEwPZSqIvDnHvVN2r0hUWXD5rHX\n".to_string();
        let expected = "please send 750 coins to 7YWHMfk9JZe0LM0g1ZauHuiSxhI\n".to_string();
        assert_eq!(expected, replace_coin_addrs(input));

        let input = "Send refunds to 7wLWEPxI9ta4H46wssWIc2FfGd please.".to_string();
        let expected = "Send refunds to 7YWHMfk9JZe0LM0g1ZauHuiSxhI please.".to_string();
        assert_eq!(expected, replace_coin_addrs(input));

        let input = "This is too long:
7sR97DfSFVSrsva9QHjQjbUmL3pgPuFRfG9l"
            .to_string();
        let expected = input.clone();
        assert_eq!(expected, replace_coin_addrs(input));

        let input = "This is a product ID, not a Boguscoin: 7nu8pGmR4XwrT2ENbJqEwPDqQmEqJJ_w7eizuzpNeRhWMzcKYp0qQmbr5sl_1234".to_string();
        let expected = input.clone();
        assert_eq!(expected, replace_coin_addrs(input));

        let input = "This is a product ID, not a Boguscoin: 7nu8pGmR4XwrT2ENbJqEwPDqQmEqJJ-w7eizuzpNeRhWMzcKYp0qQmbr5sl-1234".to_string();
        let expected = input.clone();
        assert_eq!(expected, replace_coin_addrs(input))
    }
}
