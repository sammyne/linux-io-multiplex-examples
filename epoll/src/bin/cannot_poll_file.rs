use std::fs::OpenOptions;
use std::os::fd::AsRawFd;
use std::{env, io};

use epoll;
use epoll::macros::syscall;

fn main() -> io::Result<()> {
    let path = env::temp_dir().join("hello.txt");

    let f = OpenOptions::new().create(true).read(true).write(true).open(path)?;

    let ep_fd = syscall!(epoll_create1(0))?;

    let mut interested = libc::epoll_event {
        events: libc::EPOLLIN as u32,
        u64: f.as_raw_fd() as u64,
    };

    syscall!(epoll_ctl(
        ep_fd,
        libc::EPOLL_CTL_ADD,
        f.as_raw_fd(),
        &mut interested
    )).expect("epoll_ctl");

    let mut events = [epoll::new_event(0, 0); 1024];
    syscall!(epoll_wait(
        ep_fd,
        events.as_mut_ptr(),
        events.len() as i32,
        -1
    ))?;

    Ok(())
}
