use futures::sink::SinkExt;
use futures::StreamExt;
use log::{error, info};
use std::io;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};

const UPSTREAM: &str = "chat.protohackers.com:16963";
const MAX_LENGTH: usize = 1024;
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

async fn handle_stream(stream: TcpStream) -> io::Result<()> {
    let upstream_stream = TcpStream::connect(UPSTREAM).await?;
    let mut upstream = Framed::new(upstream_stream, LinesCodec::new_with_max_length(MAX_LENGTH));
    let mut downstream = Framed::new(stream, LinesCodec::new_with_max_length(MAX_LENGTH));

    // proxy the welcome message from the upstream
    match upstream.next().await {
        Some(Ok(welcome)) => {
            downstream.send(welcome).await.unwrap();
            ()
        }
        _ => {
            error!("failed reading welcome message!");
        }
    }

    // proxy the "room contains" message
    match upstream.next().await {
        Some(Ok(room_contains_msg)) => {
            downstream.send(room_contains_msg).await.unwrap();
            ()
        }
        _ => {
            error!("failed reading the room contains message!");
        }
    }

    // chat loop
    loop {
        // read line by line from the downstream
        match downstream.next().await {
            Some(Ok(mut input)) => {
                info!("received {} from downstream, proxying!", input);
                // proxy the line upstream
                upstream.send(&mut input).await.unwrap();
                // read everything the upstream has sent
                match upstream.next().await {
                    Some(Ok(msg)) => {
                        // send everything the upstream sent to the downstream
                        // downstream.send_all(&mut upstream_buffer[..read_bytes]);
                        downstream.send(msg).await.unwrap();
                        ()
                    }
                    _ => {
                        error!("failed reading from upstream");
                        ()
                    }
                }
                ()
            }

            _ => {
                error!("failed reading from downstream!");
                break;
            }
        }
    }

    // fn process_replies(buffer: Vec<u8>) -> &[u8] {
    //     buffer.
    // }

    io::Result::Ok(())
}

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
