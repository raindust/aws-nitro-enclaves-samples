pub mod client;
pub mod command_parser;
pub mod nsm_test;
pub mod protocol_helpers;
pub mod server;
pub mod socket;
pub mod utils;

use crate::client::{send_data, vsock_connect};
use crate::server::server_with_action;
use command_parser::{ClientArgs, ServerArgs};
use std::os::unix::io::AsRawFd;

/// Send 'Hello, world!' to the server
pub fn client(args: ClientArgs) -> Result<(), String> {
    let vsocket = vsock_connect(args.cid, args.port)?;
    let fd = vsocket.as_raw_fd();

    // TODO: Replace this with your client code
    let data = "Hello, from client!".to_string();
    send_data(fd, data.as_bytes())?;

    server_with_action(args.log_port, move |buf| {
        println!(
            "{}",
            String::from_utf8(buf)
                .map_err(|err| format!("The received bytes are not UTF-8: {:?}", err))?
        );
        Ok(())
    })
}

/// Accept connections on a certain port and print
/// the received data
pub fn server(args: ServerArgs) -> Result<(), String> {
    let cid = args.cid;
    let log_port = args.log_port;
    server_with_action(args.port, move |buf| {
        println!(
            "{}",
            String::from_utf8(buf)
                .map_err(|err| format!("The received bytes are not UTF-8: {:?}", err))?
        );

        let vsocket = vsock_connect(cid, log_port)?;
        let fd = vsocket.as_raw_fd();
        send_data(fd, "Hello, from server!".as_bytes())?;

        Ok(())
    })
}
