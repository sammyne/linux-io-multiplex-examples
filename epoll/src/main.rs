use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::fd::{FromRawFd, IntoRawFd};
use std::os::unix::io::AsRawFd;
use std::{io, ptr};

mod macros;

fn main() -> io::Result<()> {
    let mut listener = TcpListener::bind("127.0.0.1:8000")?;
    listener.set_nonblocking(true)?;
    let listener_fd = listener.as_raw_fd();

    let ep_fd = macros::syscall!(epoll_create1(0))?;

    let mut event = libc::epoll_event {
        events: libc::EPOLLIN as u32,
        u64: listener_fd as u64,
    };

    macros::syscall!(epoll_ctl(
        ep_fd,
        libc::EPOLL_CTL_ADD,
        listener_fd,
        &mut event
    ))?;

    let mut events = [new_event(0, 0); 1024];
    // let mut events = Vec::with_capacity(1024);

    loop {
        let n = macros::syscall!(epoll_wait(
            ep_fd,
            events.as_mut_ptr(),
            events.len() as i32,
            -1
        ))?;

        for v in &events[..(n as usize)] {
            let fd = v.u64 as i32;
            if fd == listener_fd {
                accept(&mut listener, ep_fd)?;
                continue;
            }

            recv(fd, ep_fd)?;
        }
    }
}

fn accept(l: &mut TcpListener, ep_fd: i32) -> io::Result<()> {
    let (stream, addr) = match l.accept() {
        Ok(v) => v,
        Err(err) => {
            eprintln!("couldn't accept: {}", err);
            return Err(err);
        }
    };
    stream.set_nonblocking(true)?;

    println!("connection from {addr}");

    // @warn：转化为裸 fd 避免连接被关闭
    let stream = stream.into_raw_fd();

    let mut event = new_event(libc::EPOLLIN as u32, stream as u64);
    let _ = macros::syscall!(epoll_ctl(ep_fd, libc::EPOLL_CTL_ADD, stream, &mut event))?;

    Ok(())
}

fn new_event(events: u32, data: u64) -> libc::epoll_event {
    libc::epoll_event { events, u64: data }
}

fn recv(conn: i32, ep_fd: i32) -> io::Result<()> {
    let mut stream = unsafe { TcpStream::from_raw_fd(conn) };
    stream.set_nonblocking(true)?;

    let mut buf = [0u8; 1024];
    let mut keep = false;
    match stream.read(&mut buf) {
        Ok(0) => {
            println!("客户端已经断开连接");
            macros::syscall!(epoll_ctl(ep_fd, libc::EPOLL_CTL_DEL, conn, ptr::null_mut()))?;
        }
        Ok(n) => {
            let msg = unsafe { std::str::from_utf8_unchecked(&buf[..n]) };
            println!("客户端说：{msg}");
            let _ = stream.write_all(msg.as_bytes())?;
            keep = true;
        }
        Err(err) if err.kind() == ErrorKind::WouldBlock => keep = true,
        Err(err) => return Err(err),
    }

    if keep {
        let _ = stream.into_raw_fd();
    }

    Ok(())
}
