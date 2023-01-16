use lazy_static::lazy_static;
use log::{error, info};
use regex::Regex;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
// use tokio::net::tcp::{ReadHalf, WriteHalf};

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
    //
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

    // chat loop
    loop {
        tokio::select! {
            // listen for downstream
            Ok(num_bytes) = downstream_rx.read(&mut downstream_buf) => {
                if num_bytes == 0 {
                    break;
                }
                info!("upstream <- downstream: message {} bytes", num_bytes);
                let string = String::from_utf8_lossy(&downstream_buf[..num_bytes]);
                let updated = replace_coin_addrs(string.to_string());
                upstream_tx.write_all(updated.as_bytes()).await?
            }
            // listen for upstream
            Ok(num_bytes) = upstream_rx.read(&mut upstream_buf) => {
                if num_bytes == 0 {
                    break;
                }
                info!("upstream -> downstream: replies, {} bytes", num_bytes);
                let string = String::from_utf8_lossy(&downstream_buf[..num_bytes]);
                let updated = replace_coin_addrs(string.to_string());
                downstream_tx
                    .write_all(updated.as_bytes())
                    .await?
            }
        }
    }
    io::Result::Ok(())
}

fn replace_coin_addrs(message: String) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new("(7[a-zA-Z0-9]+)").unwrap();
    }

    RE.find_iter(&message.to_string())
        .fold(message, |response, possible| {
            let pos = possible.as_str();
            if pos.len() > 25 && pos.len() <= 36 {
                response.replace(pos, TONY_ADDR)
            } else {
                response
            }
        })
}

#[cfg(test)]
mod tests {
    use crate::replace_coin_addrs;

    #[test]
    fn replace_coin_addr_test() {
        let input = "".to_string();
        let expected = "".to_string();
        assert_eq!(expected, replace_coin_addrs(input));

        let input = "Hi alice, please send payment to 7iKDZEwPZSqIvDnHvVN2r0hUWXD5rHX".to_string();
        let expected = "Hi alice, please send payment to 7YWHMfk9JZe0LM0g1ZauHuiSxhI".to_string();
        assert_eq!(expected, replace_coin_addrs(input));

        let input = "please send 750 coins to 7iKDZEwPZSqIvDnHvVN2r0hUWXD5rHX".to_string();
        let expected = "please send 750 coins to 7YWHMfk9JZe0LM0g1ZauHuiSxhI".to_string();
        assert_eq!(expected, replace_coin_addrs(input));

        let input = "Send refunds to 7wLWEPxI9ta4H46wssWIc2FfGd please.".to_string();
        let expected = "Send refunds to 7YWHMfk9JZe0LM0g1ZauHuiSxhI please.".to_string();
        assert_eq!(expected, replace_coin_addrs(input));

        let input = "Send refunds to 7QyB2Y1ZaKPCI3YBFdwjGv0jO6je7b please.".to_string();
        let expected = "Send refunds to 7YWHMfk9JZe0LM0g1ZauHuiSxhI please.".to_string();
        assert_eq!(expected, replace_coin_addrs(input))
    }
}
// async fn proxy_message(
//     mut from: ReadHalf,
//     mut to: WriteHalf,
//     &mut buffer: Vec<u8>,
// ) -> io::Result<()> {
//     match from.read(&mut buffer).await {
//         Ok(num_bytes) => {
//             info!(
//                 "{:?} -> {:?}: welcome msg, {} bytes",
//                 from.peer_addr(),
//                 to.peer_addr(),
//                 num_bytes
//             );
//             to.write_all(&mut buffer[..num_bytes]).await?;
//             info!("sent welcome")
//         }
//         error => error!("failed to read welcome message: {:?}", error),
//     }
//     io::Result::Ok(())
// }
