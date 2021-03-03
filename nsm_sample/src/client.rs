use crate::protocol_helpers::{send_loop, send_u64};
use crate::socket::VsockSocket;
use nix::sys::socket::{connect, socket, AddressFamily, SockAddr, SockFlag, SockType};
use std::convert::TryInto;
use std::os::unix::io::{AsRawFd, RawFd};

// Maximum number of connection attempts
const MAX_CONNECTION_ATTEMPTS: usize = 5;

pub fn send_data(fd: RawFd, buf: &[u8]) -> Result<(), String> {
    let len: u64 = buf.len().try_into().map_err(|err| format!("{:?}", err))?;
    send_u64(fd, len)?;
    send_loop(fd, &buf, len)?;
    Ok(())
}

/// Initiate a connection on an AF_VSOCK socket
pub fn vsock_connect(cid: u32, port: u32) -> Result<VsockSocket, String> {
    let sockaddr = SockAddr::new_vsock(cid, port);
    let mut err_msg = String::new();

    for i in 0..MAX_CONNECTION_ATTEMPTS {
        let vsocket = VsockSocket::new(
            socket(
                AddressFamily::Vsock,
                SockType::Stream,
                SockFlag::empty(),
                None,
            )
            .map_err(|err| format!("Failed to create the socket: {:?}", err))?,
        );
        match connect(vsocket.as_raw_fd(), &sockaddr) {
            Ok(_) => return Ok(vsocket),
            Err(e) => err_msg = format!("Failed to connect: {}", e),
        }

        // Exponentially backoff before retrying to connect to the socket
        std::thread::sleep(std::time::Duration::from_secs(1 << i));
    }

    Err(err_msg)
}
