use std::io;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:7878").await?;

    loop {
        let (stream, _address) = listener.accept().await?;
        tokio::spawn(async move { handle_stream(stream).await });
    }
}

async fn handle_stream(mut stream: TcpStream) -> io::Result<()> {
    let (mut reader, mut writer) = stream.split();
    copy(&mut reader, &mut writer).await?;
    io::Result::Ok(())
}
