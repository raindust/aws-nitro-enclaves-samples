use nix::sys::socket::{shutdown, Shutdown};
use nix::unistd::close;
use std::os::unix::io::{AsRawFd, RawFd};

pub struct VsockSocket {
    socket_fd: RawFd,
}

impl VsockSocket {
    pub fn new(socket_fd: RawFd) -> Self {
        VsockSocket { socket_fd }
    }
}

impl Drop for VsockSocket {
    fn drop(&mut self) {
        shutdown(self.socket_fd, Shutdown::Both)
            .unwrap_or_else(|e| eprintln!("Failed to shut socket down: {:?}", e));
        close(self.socket_fd).unwrap_or_else(|e| eprintln!("Failed to close socket: {:?}", e));
    }
}

impl AsRawFd for VsockSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.socket_fd
    }
}
