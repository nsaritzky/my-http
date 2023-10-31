use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").await?;
    match listener.accept().await {
        Ok((mut stream, _)) => {
            println!("accepted new connection");
            stream.write_all(b"HTTP/1.1 200 OK/r/n/r/n").await?;
        }
        Err(e) => {
            println!("error: {}", e);
        }
    }
    Ok(())
}
