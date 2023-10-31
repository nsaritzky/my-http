use anyhow::{bail, Result};
use nom::bytes::complete::take_till;
use nom::character::is_space;
use nom::multi::many0;
use nom::sequence::{delimited, preceded};
use nom::{bytes::complete::tag, IResult};
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
    let (_rest, path) = match parse_path(&buf) {
        Ok((rest, path)) => (rest, path),
        Err(e) => bail!("{e}"),
    };
    match &path[..] {
        [b"echo", s] => {
            stream.write_all(b"HTTP/1.1 200 OK\r\n").await?;
            stream.write_all(b"Content-Type: text/plain\r\n").await?;
            stream
                .write_all(format!("Content-Length: {}\r\n\r\n", s.len()).as_bytes())
                .await?;
            stream.write_all(s).await?;
        }
        &_ => {
            stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\n").await?;
        }
    }
    Ok(())
}

fn parse_path_segment(input: &[u8]) -> IResult<&[u8], &[u8]> {
    preceded(tag("/"), take_till(|c: u8| c == b'/' || is_space(c)))(input)
}

fn parse_path(input: &[u8]) -> IResult<&[u8], Vec<&[u8]>> {
    delimited(tag("GET "), many0(parse_path_segment), tag(" HTTP/1.1\r\n"))(input)
}
