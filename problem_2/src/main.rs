use log::info;
use std::collections::BTreeMap;
use std::io;
use std::ops::Bound::Included;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();
    let listener = TcpListener::bind("0.0.0.0:7878").await?;

    loop {
        let (stream, _address) = listener.accept().await?;
        info!("accept");
        tokio::spawn(async move { handle_stream(stream).await });
    }
}

enum Message {
    Insert { price: i32, timestamp: i32 },
    Query { mintime: i32, maxtime: i32 },
    Invalid,
}

impl Message {
    pub fn cast(bytes: [u8; 9]) -> Message {
        match bytes[0] {
            73 => Message::Insert {
                timestamp: Self::parse_int(&bytes[1..5]),
                price: Self::parse_int(&bytes[5..9]),
            },
            81 => Message::Query {
                mintime: Self::parse_int(&bytes[1..5]),
                maxtime: Self::parse_int(&bytes[5..9]),
            },
            _ => Message::Invalid,
        }
    }

    fn parse_int(bytes: &[u8]) -> i32 {
        i32::from_be_bytes(bytes.try_into().unwrap())
    }
}

async fn handle_stream(mut stream: TcpStream) -> io::Result<()> {
    let mut db: BTreeMap<i32, i32> = BTreeMap::new();
    let (read_half, mut writer) = stream.split();
    let mut reader = BufReader::new(read_half);
    let mut message_bytes: [u8; 9] = [0; 9];
    while let Ok(_num_bytes) = reader.read_exact(&mut message_bytes).await {
        match Message::cast(message_bytes) {
            Message::Insert { price, timestamp } => {
                info!("{:?} - insert {} @ {}", message_bytes, price, timestamp);
                db.insert(timestamp, price);
                ()
            }
            Message::Query { mintime, maxtime } => {
                info!("{:?} - query {} to {}", message_bytes, mintime, maxtime);
                if mintime > maxtime {
                    writer.write_i32(0).await?;
                } else {
                    let mut count: i32 = 0;
                    let mut sum: i32 = 0;
                    for (_timestamp, &amount) in db.range((Included(mintime), Included(maxtime))) {
                        count = count + 1;
                        sum = sum + amount;
                    }
                    let mean = if count > 0 { sum / count } else { 0 };
                    info!("query result: sum {}, count {}, mean {}", sum, count, mean);
                    writer.write(&mean.to_be_bytes()).await?;
                }
                ()
            }
            Message::Invalid => {
                info!("got invalid message: {:?}", message_bytes);
                message_bytes = [0; 9];
                ()
            }
        };
        info!("");
    }

    io::Result::Ok(())
}
