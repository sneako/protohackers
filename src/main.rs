extern crate primal;

use serde::{Deserialize, Serialize};
use std::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Request {
    method: String,
    number: f64,
}

#[derive(Serialize, Deserialize)]
struct PrimeResponse {
    method: String,
    prime: bool,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // echo server
    let listener = TcpListener::bind("0.0.0.0:7878").await?;

    loop {
        let (stream, _address) = listener.accept().await?;
        // tokio::spawn(async move { echo(stream) });
        // tokio::spawn(async move { rpc_server(stream) });
        tokio::spawn(async move {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            while let Ok(num_bytes) = reader.read_line(&mut line).await {
                println!("read line");
                if num_bytes == 0 {
                    break;
                }
                println!("{}", line.trim());
                let request: Request = serde_json::from_str(&line.trim()).unwrap();

                let result = match request.method.as_str() {
                    "isPrime" => handle_is_prime(&request.number),
                    _ => panic!("unknown method!"),
                };

                let json = serde_json::to_string(&result).unwrap();
                reader.write_all(&json.as_bytes()).await?;
                reader.write_u8(10).await?;
                line.clear();
            }

            io::Result::Ok(())
        });
    }
}

// async fn echo(mut stream: TcpStream) -> io::Result<()> {
//     let (mut reader, mut writer) = stream.split();
//     copy(&mut reader, &mut writer).await?;
//     io::Result::Ok(())
// }

// async fn rpc_server(stream: TcpStream) -> io::Result<()> {
//     let mut buffer = BufReader::new(stream);
//     let mut line = String::new();
//     while let Ok(num_bytes) = buffer.read_line(&mut line).await {
//         if num_bytes == 0 {
//             break;
//         }
//         let request: Request = serde_json::from_str(&line).unwrap();
//
//         match request.method.as_str() {
//             "isPrime" => handle_is_prime(stream, &request.number),
//             _ => panic!("unknown method!"),
//         };
//     }
//
//     io::Result::Ok(())
// }

fn handle_is_prime(number: &f64) -> PrimeResponse {
    PrimeResponse {
        method: String::from("isPrime"),
        prime: primal::is_prime(*number as u64),
    }
}

#[cfg(test)]
mod tests {
    use crate::Request;

    #[test]
    fn serde_test() {
        let input = "{\"method\":\"isPrime\",\"number\":123}";
        let request: Request = serde_json::from_str(input).unwrap();
        let expected = Request {
            method: String::from("isPrime"),
            number: 123.0,
        };
        assert_eq!(expected, request);
    }
}
