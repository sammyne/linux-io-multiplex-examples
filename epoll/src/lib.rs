pub mod macros;

pub fn new_event(events: u32, data: u64) -> libc::epoll_event {
    libc::epoll_event { events, u64: data }
}
