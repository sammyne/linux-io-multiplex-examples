use std::net::TcpListener;
use std::os::raw::c_void;
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
    //let (stream, addr) = match l.accept() {
    //    Ok(v) => v,
    //    Err(err) => {
    //        eprintln!("couldn't accept: {}", err);
    //        return Err(err);
    //    }
    //};
    //stream.set_nonblocking(true)?;
    let stream = macros::syscall!(accept(l.as_raw_fd(), ptr::null_mut(),ptr::null_mut()))?;

    //println!("connection from {addr}");

    //let mut event = new_event(libc::EPOLLIN as u32, stream.as_raw_fd() as u64);
    let mut event = new_event(libc::EPOLLIN as u32, stream as u64);
    let _ = macros::syscall!(epoll_ctl(
        ep_fd,
        libc::EPOLL_CTL_ADD,
        //stream.as_raw_fd(),
        stream,
        &mut event
    ))?;

    Ok(())
}

fn new_event(events: u32, data: u64) -> libc::epoll_event {
    libc::epoll_event { events, u64: data }
}

fn recv(conn: i32, ep_fd: i32) -> io::Result<()> {
    let mut buf = [0u8; 1024];

    match macros::syscall!(recv(conn, buf.as_mut_ptr() as *mut c_void, buf.len(), 0))? {
        0 => {
            println!("客户端已经断开连接");
            macros::syscall!(epoll_ctl(ep_fd, libc::EPOLL_CTL_DEL, conn, ptr::null_mut()))?;
            macros::syscall!(close(conn))?;
        }
        v if v > 0 => {
            let msg = unsafe { std::str::from_utf8_unchecked(&buf[..(v as usize)]) };
            println!("客户端说：{msg}");
            macros::syscall!(send(conn, buf.as_ptr() as *const c_void, v as usize, 0))?;
        }
        _ => eprintln!("recv"),
    }

    Ok(())
}
