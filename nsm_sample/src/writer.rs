use crate::client::{send_data, vsock_connect};
use std::os::unix::io::AsRawFd;

pub struct LogWriter {
    port: u32,
}

impl LogWriter {
    pub fn new(port: u32) -> Self {
        LogWriter { port }
    }
}

impl std::io::Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match vsock_connect(libc::VMADDR_CID_HOST, self.port) {
            Ok(socket) => {
                let fd = socket.as_raw_fd();
                let len = send_data(fd, buf).unwrap() as usize;
                Ok(len)
            }
            Err(e) => {
                println!("write log got error: {}", e);
                Ok(0)
            }
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // do nothing here
        Ok(())
    }
}
