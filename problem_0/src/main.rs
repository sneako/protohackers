use std::io;
use std::net::SocketAddr;
use tokio_uring::buf::IoBuf;
use tokio_uring::net::{TcpListener, TcpStream};

fn main() {
    tokio_uring::start(async {
        let socket_addr: SocketAddr = "0.0.0.0:7878".parse().unwrap();
        let listener = TcpListener::bind(socket_addr).unwrap();

        loop {
            let (stream, _address) = listener.accept().await.unwrap();
            tokio_uring::spawn(async move { handle_stream(stream).await });
        }
    });
}

async fn handle_stream(stream: TcpStream) {
    let mut buf = vec![0u8; 4096];
    loop {
        let (result, nbuf) = stream.read(buf).await;
        buf = nbuf;
        let read = result.unwrap();
        if read == 0 {
            break;
        }

        let (res, slice) = stream.write_all(buf.slice(..read)).await;
        let _ = res.unwrap();
        buf = slice.into_inner();
    }
}
