use log::{error, info};
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
// use tokio::net::tcp::{ReadHalf, WriteHalf};

const UPSTREAM: &str = "chat.protohackers.com:16963";
// static TONY_ADDR: &str = "7YWHMfk9JZe0LM0g1ZauHuiSxhI";

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
            info!("upstream -> downstream: welcome msg, {} bytes", num_bytes);
            downstream_tx
                .write_all(&mut upstream_buf[..num_bytes])
                .await?;
            info!("sent welcome")
        }
        error => error!("failed to read welcome message: {:?}", error),
    }

    // proxy the name
    match downstream_rx.read(&mut downstream_buf).await {
        Ok(num_bytes) => {
            info!("downstream -> upstream: name, {} bytes", num_bytes);
            upstream_tx
                .write_all(&mut downstream_buf[..num_bytes])
                .await?;
            info!("sent name");
        }
        _ => {
            error!("failed reading the name!");
        }
    }

    // proxy the room contains message
    match upstream_rx.read(&mut upstream_buf).await {
        Ok(num_bytes) => {
            info!(
                "upstream -> downstream: room contains msg, {} bytes",
                num_bytes
            );
            downstream_tx
                .write_all(&mut upstream_buf[..num_bytes])
                .await?;
            info!("sent room contains")
        }
        error => error!("failed reading room contains message: {:?}", error),
    }

    // chat loop
    loop {
        info!("looop");
        tokio::select! {
            // listen for downstream
            Ok(num_bytes) = downstream_rx.read(&mut downstream_buf) => {
                info!("upstream <- downstream: message {} bytes", num_bytes);
                // proxy the line upstream
                upstream_tx
                    .write_all(&mut downstream_buf[..num_bytes])
                    .await?;
            }
            // listen for upstream
            Ok(num_bytes) = upstream_rx.read(&mut upstream_buf) => {
                if num_bytes == 0 {
                    break;
                }
                info!("upstream -> downstream: replies, {} bytes", num_bytes);
                // send everything the upstream sent to the downstream
                // downstream.send_all(&mut upstream_buffer[..read_bytes]);
                downstream_tx
                    .write_all(&mut upstream_buf[..num_bytes])
                    .await?;
                ()
            }
        }
    }
    io::Result::Ok(())
}

// async fn proxy_message(
//     mut from: ReadHalf<'_>,
//     mut to: WriteHalf<'_>,
//     mut buffer: Vec<u8>,
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
// fn username_valid(input: &String) -> bool {
//     let mut all_alphanumeric = true;
//     let mut any_alphabetic = false;
//
//     input.chars().for_each(|c| {
//         if !any_alphabetic && c.is_alphabetic() {
//             any_alphabetic = true;
//         }
//
//         if !c.is_alphanumeric() {
//             all_alphanumeric = false;
//         }
//     });
//     return all_alphanumeric && any_alphabetic;
// }
