use std::io::{self, Read, Write};
use std::net::TcpStream;

fn main() -> io::Result<()> {
    let mut c = TcpStream::connect("127.0.0.1:8000")?;

    let request = "hello world";

    c.write_all(request.as_bytes())?;

    let mut buf = [0u8; 128];
    let n = c.read(&mut buf)?;

    let reply = unsafe { std::str::from_utf8_unchecked(&buf[..n]) };
    println!("feedback = '{reply}'");

    Ok(())
}
