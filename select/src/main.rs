use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::fd::{FromRawFd, IntoRawFd};
use std::os::unix::io::AsRawFd;
use std::{io, mem, ptr};

use libc::fd_set;
use macros::syscall;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    listener.set_nonblocking(true)?;
    let listener_fd = listener.as_raw_fd();

    let mut fd_max = listener_fd;

    let mut reads: libc::fd_set = unsafe { mem::zeroed() };
    let mut temp: libc::fd_set;

    unsafe { libc::FD_ZERO(&mut reads) };
    unsafe { libc::FD_SET(listener_fd, &mut reads) };

    loop {
        temp = reads;
        let _ = syscall!(select(
            fd_max + 1,
            &mut temp,
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut()
        ))?;
        if unsafe { libc::FD_ISSET(listener_fd, &temp) } {
            let (stream, addr) = listener.accept()?;
            stream.set_nonblocking(true)?;

            println!("connection from '{addr}'");

            let client_fd = stream.into_raw_fd();
            unsafe { libc::FD_SET(client_fd, &mut reads) };
            fd_max = fd_max.max(client_fd);
        }

        for i in 0..(fd_max + 1) {
            if i == listener_fd || !unsafe { libc::FD_ISSET(i, &temp) } {
                continue;
            }

            unsafe { feedback(i, &mut reads) }?;
        }
    }
}

unsafe fn feedback(fd: i32, reads: &mut fd_set) -> io::Result<()> {
    let mut conn = TcpStream::from_raw_fd(fd);
    conn.set_nonblocking(true)?;

    let mut buf = [0u8; 10];

    let mut keep = false;
    match conn.read(&mut buf) {
        Ok(0) => {
            println!("客户端关闭了连接");
            libc::FD_CLR(fd, reads);
        }
        Ok(n) => {
            let msg = std::str::from_utf8_unchecked(&buf[..n]);
            println!("feedback '{}'", msg);
            conn.write(msg.as_bytes())?;
            keep = true;
        }
        Err(err) if err.kind() == ErrorKind::WouldBlock => keep = true,
        Err(err) => {
            eprintln!("read");
            return Err(err);
        }
    }

    if keep {
        let _ = conn.into_raw_fd();
    }

    Ok(())
}

mod macros;
