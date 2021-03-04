use crate::nsm_test::test_nsm;
use crate::protocol_helpers::{recv_loop, recv_u64};
use nix::sys::socket::listen as listen_vsock;
use nix::sys::socket::{accept, bind, socket, AddressFamily, SockAddr, SockFlag, SockType};

const VMADDR_CID_ANY: u32 = 0xFFFFFFFF;
const BUF_MAX_LEN: usize = 8192;
// Maximum number of outstanding connections in the socket's
// listen queue
const BACKLOG: usize = 128;

/// Accept connections on a certain port and print
/// the received data
pub fn server_with_action<F>(port: u32, mut action: F) -> Result<(), String>
where
    F: FnMut(Vec<u8>) -> Result<(), String>,
{
    let socket_fd = socket(
        AddressFamily::Vsock,
        SockType::Stream,
        SockFlag::empty(),
        None,
    )
    .map_err(|err| format!("Create socket failed: {:?}", err))?;

    let sockaddr = SockAddr::new_vsock(VMADDR_CID_ANY, port);

    bind(socket_fd, &sockaddr).map_err(|err| format!("Bind failed: {:?}", err))?;

    listen_vsock(socket_fd, BACKLOG).map_err(|err| format!("Listen failed: {:?}", err))?;

    // test_nsm();

    loop {
        let fd = accept(socket_fd).map_err(|err| format!("Accept failed: {:?}", err))?;

        // TODO: Replace this with your server code
        let len = recv_u64(fd)?;
        let mut buf = [0u8; BUF_MAX_LEN];
        recv_loop(fd, &mut buf, len)?;
        action(buf.to_vec())?;
    }
}
