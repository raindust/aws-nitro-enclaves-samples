pub mod command_parser;
pub mod protocol_helpers;
pub mod utils;

use command_parser::{ClientArgs, ServerArgs};
use protocol_helpers::{recv_loop, recv_u64, send_loop, send_u64};

use nix::sys::socket::listen as listen_vsock;
use nix::sys::socket::{accept, bind, connect, shutdown, socket};
use nix::sys::socket::{AddressFamily, Shutdown, SockAddr, SockFlag, SockType};
use nix::unistd::close;
use std::convert::TryInto;
use std::os::unix::io::{AsRawFd, RawFd};
use tempfile;

const VMADDR_CID_ANY: u32 = 0xFFFFFFFF;
const BUF_MAX_LEN: usize = 8192;
// Maximum number of outstanding connections in the socket's
// listen queue
const BACKLOG: usize = 128;
// Maximum number of connection attempts
const MAX_CONNECTION_ATTEMPTS: usize = 5;

struct VsockSocket {
    socket_fd: RawFd,
}

impl VsockSocket {
    fn new(socket_fd: RawFd) -> Self {
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

/// Initiate a connection on an AF_VSOCK socket
fn vsock_connect(cid: u32, port: u32) -> Result<VsockSocket, String> {
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

/// Send 'Hello, world!' to the server
pub fn client(args: ClientArgs) -> Result<(), String> {
    let vsocket = vsock_connect(args.cid, args.port)?;
    let fd = vsocket.as_raw_fd();

    // TODO: Replace this with your client code
    let data = "Hello, world!".to_string();
    let buf = data.as_bytes();
    let len: u64 = buf.len().try_into().map_err(|err| format!("{:?}", err))?;
    send_u64(fd, len)?;
    send_loop(fd, &buf, len)?;

    Ok(())
}

/// Accept connections on a certain port and print
/// the received data
pub fn server(args: ServerArgs) -> Result<(), String> {
    let socket_fd = socket(
        AddressFamily::Vsock,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .map_err(|err| format!("Create socket failed: {:?}", err))?;

    let sockaddr = SockAddr::new_vsock(VMADDR_CID_ANY, args.port);

    bind(socket_fd, &sockaddr).map_err(|err| format!("Bind failed: {:?}", err))?;

    listen_vsock(socket_fd, BACKLOG).map_err(|err| format!("Listen failed: {:?}", err))?;

    test_socket();

    loop {
        let fd = accept(socket_fd).map_err(|err| format!("Accept failed: {:?}", err))?;

        // TODO: Replace this with your server code
        let len = recv_u64(fd)?;
        let mut buf = [0u8; BUF_MAX_LEN];
        recv_loop(fd, &mut buf, len)?;
        println!(
            "{}",
            String::from_utf8(buf.to_vec())
                .map_err(|err| format!("The received bytes are not UTF-8: {:?}", err))?
        );
    }
}

fn test_socket() {
    println!("begin of test socket");

    use nix::sys::socket::{SockType, SockFlag};
    use nix::sys::socket::{bind, socket, connect, listen, accept, SockAddr};
    use nix::unistd::{read, write, close};
    use std::thread;

    let tempdir = tempfile::tempdir().unwrap();
    println!("get temp dir: {}", &tempdir);
    let sockname = tempdir.path().join("sock");
    let s1 = socket(AddressFamily::Unix, SockType::Stream,
                    SockFlag::empty(), None).expect("socket failed");
    let sockaddr = SockAddr::new_unix(&sockname).unwrap();
    bind(s1, &sockaddr).expect("bind failed");
    println!("bind socket successfully");
    listen(s1, 10).expect("listen failed");
    println!("listen socket successfully");

    let thr = thread::spawn(move || {
        println!("begin of client");
        let s2 = socket(AddressFamily::Unix, SockType::Stream, SockFlag::empty(), None)
            .expect("socket failed");
        connect(s2, &sockaddr).expect("connect failed");
        write(s2, b"hello").expect("write failed");
        close(s2).unwrap();
        println!("end of client");
    });

    let s3 = accept(s1).expect("accept failed");
    println!("access socket successfully");

    let mut buf = [0;5];
    read(s3, &mut buf).unwrap();
    println!("read message: {:?}", &buf);
    close(s3).unwrap();
    close(s1).unwrap();
    thr.join().unwrap();

    assert_eq!(&buf[..], b"hello");

    println!("end of test socket");
}