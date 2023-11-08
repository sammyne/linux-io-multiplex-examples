use std::io::{self, ErrorKind, Read, Write};
use std::mem;
use std::net::{TcpListener, TcpStream};
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd};

use macros::syscall;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    listener.set_nonblocking(true)?;
    let listener_fd = listener.as_raw_fd();

    let mut fds: [libc::pollfd; 1024] = unsafe { mem::zeroed() };
    for v in fds.iter_mut() {
        v.fd = -1;
        v.events = libc::POLLIN;
    }
    fds[0].fd = listener_fd;

    let mut fd_max = 0;
    loop {
        let _ = syscall!(poll(fds.as_mut_ptr(), fd_max + 1, -1))?;

        if (fds[0].revents & libc::POLLIN) != 0 {
            let (stream, addr) = listener.accept()?;
            println!("connection from '{addr}'");

            let (i, fd) = fds
                .iter_mut()
                .enumerate()
                .find(|(_, v)| v.fd == -1)
                .unwrap();
            fd.fd = stream.into_raw_fd();
            fd_max = fd_max.max(i as u64);
        }

        for v in fds[1..=(fd_max as usize)].iter_mut() {
            // 新建的 fd 的 revents 为 0，不会通过这个检查
            if (v.revents & libc::POLLIN) == 0 {
                continue;
            }

            unsafe { feedback(v)? };
        }
    }
}

unsafe fn feedback(fd: &mut libc::pollfd) -> io::Result<()> {
    let mut conn = TcpStream::from_raw_fd(fd.fd);
    conn.set_nonblocking(true)?;

    let mut buf = [0u8; 10];
    let mut keep = false;
    match conn.read(&mut buf) {
        Ok(0) => {
            println!("客户端关闭了连接");
            fd.fd = -1;
            fd.revents = 0;
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
