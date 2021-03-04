use crate::client::{send_data, vsock_connect};
use crate::socket::VsockSocket;
use std::os::unix::io::AsRawFd;

pub struct LogWriter {
    socket: VsockSocket,
}

impl LogWriter {
    pub fn new(port: u32) -> Result<Self, String> {
        Ok(LogWriter {
            socket: vsock_connect(libc::VMADDR_CID_HOST, port)?,
        })
    }
}

impl std::io::Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let fd = self.socket.as_raw_fd();
        let len = send_data(fd, buf).unwrap() as usize;
        Ok(len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // do nothing here
        Ok(())
    }
}
