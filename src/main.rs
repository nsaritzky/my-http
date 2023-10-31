use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").await?;
    loop {
        let (mut stream, _addr) = listener.accept().await?;
        tokio::spawn(async move { process(&mut stream).await.expect("process failed") });
    }
}

async fn process(stream: &mut TcpStream) -> Result<()> {
    let mut buf = [0; 1024];
    stream.read(&mut buf).await?;

    stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await?;
    Ok(())
}
