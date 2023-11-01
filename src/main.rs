use anyhow::{bail, Result};
use nom::bytes::complete::take_till;
use nom::character::complete::line_ending;
use nom::character::is_space;
use nom::multi::many0;
use nom::sequence::{delimited, preceded, separated_pair, terminated};
use nom::{bytes::complete::tag, IResult};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").await?;
    let dir_arg = Arc::new(std::env::args().nth(2));

    loop {
        let (mut stream, _addr) = listener.accept().await?;
        let dir_arg = Arc::clone(&dir_arg);
        tokio::spawn(async move { process(&mut stream, dir_arg).await.expect("process failed") });
    }
}

async fn process(stream: &mut TcpStream, dir_arg: Arc<Option<String>>) -> Result<()> {
    let mut buf = [0; 1024];
    stream.read(&mut buf).await?;
    let (rest, path) = match parse_path(&buf) {
        Ok((rest, path)) => (rest, path),
        Err(e) => bail!("{e}"),
    };
    match &path[..] {
        [b""] => {
            stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await?;
        }
        [b"echo", rest @ ..] => {
            let s = rest.join(&b'/');
            stream.write_all(b"HTTP/1.1 200 OK\r\n").await?;
            stream.write_all(b"Content-Type: text/plain\r\n").await?;
            stream
                .write_all(format!("Content-Length: {}\r\n\r\n", s.len()).as_bytes())
                .await?;
            stream.write_all(&s).await?;
        }
        [b"files", filename] => {
            let target_filename = String::from_utf8(filename.to_vec()).unwrap();
            println!("target_filename: {}", target_filename);
            if let Some(dir) = dir_arg.as_ref() {
                if let Some(path) = search_directory_for_file(&dir, &target_filename) {
                    println!("path: {}", path);
                    let content = std::fs::read(path.clone())?;
                    stream.write_all(b"HTTP/1.1 200 OK\r\n").await?;
                    stream
                        .write_all(b"Content-Type: application/octet-stream\r\n")
                        .await?;
                    stream
                        .write_all(format!("Content-Length: {}\r\n\r\n", content.len()).as_bytes())
                        .await?;
                    stream.write_all(&content).await?;
                } else {
                    stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\n").await?;
                }
            }
        }
        [b"user-agent"] => {
            let (_rest0, headers) = parse_headers(&rest).map_err(|e| anyhow::anyhow!("{e}"))?;
            let i = headers
                .iter()
                .position(|(k, _)| k == b"User-Agent")
                .ok_or(anyhow::anyhow!("User-Agent not found"))?;
            let (_, ua) = headers[i];
            stream.write_all(b"HTTP/1.1 200 OK\r\n").await?;
            stream.write_all(b"Content-Type: text/plain\r\n").await?;
            stream
                .write_all(format!("Content-Length: {}\r\n\r\n", ua.len()).as_bytes())
                .await?;
            stream.write_all(ua).await?;
        }
        &_ => {
            stream.write_all(b"HTTP/1.1 404 NOT FOUND\r\n\r\n").await?;
        }
    }
    Ok(())
}

fn search_directory_for_file(directory: &str, target_filename: &str) -> Option<String> {
    println!("directory: {}", directory);
    if let Ok(entries) = std::fs::read_dir(directory) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Some(filename) = entry.file_name().to_str() {
                    if filename == target_filename {
                        return Some(entry.path().to_string_lossy().into_owned());
                    }
                }
            }
        }
    }
    None
}

fn parse_header(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    terminated(
        separated_pair(
            take_till(|c: u8| c == b':'),
            tag(": "),
            take_till(|c: u8| c == b'\r' || c == b'\n'),
        ),
        many0(line_ending),
    )(input)
}

fn parse_headers(input: &[u8]) -> IResult<&[u8], Vec<(&[u8], &[u8])>> {
    many0(parse_header)(input)
}

fn parse_path_segment(input: &[u8]) -> IResult<&[u8], &[u8]> {
    preceded(tag("/"), take_till(|c: u8| c == b'/' || is_space(c)))(input)
}

fn parse_path<'a>(input: &'a [u8]) -> IResult<&'a [u8], Vec<&'a [u8]>> {
    delimited(tag("GET "), many0(parse_path_segment), tag(" HTTP/1.1\r\n"))(input)
}
