pub mod client;
pub mod command_parser;
pub mod nsm_test;
pub mod protocol_helpers;
pub mod server;
pub mod socket;
pub mod utils;
pub mod writer;

use crate::client::{send_data, vsock_connect};
use crate::server::server_with_action;
use crate::writer::LogWriter;
use command_parser::{ClientArgs, ServerArgs};
use simplelog::*;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::time::Duration;

#[macro_use]
extern crate log;

/// Send 'Hello, world!' to the server
pub fn client(args: ClientArgs) -> Result<(), String> {
    CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("my_rust_binary.log").unwrap(),
        ),
    ])
    .map_err(|e| format!("{:?}", e))?;

    let log_port = args.log_port;
    std::thread::spawn(move || {
        match server_with_action(log_port, move |buf| {
            println!(
                "log from server: {}",
                String::from_utf8(buf)
                    .map_err(|err| format!("The received bytes are not UTF-8: {:?}", err))?
            );
            Ok(())
        }) {
            Err(e) => println!("client server got error: {}", e),
            _ => {}
        }
    });

    let vsocket = vsock_connect(args.cid, args.port)?;
    send_data(vsocket.as_raw_fd(), "init".as_bytes())?;

    let vsocket = vsock_connect(args.cid, args.port)?;
    send_data(vsocket.as_raw_fd(), "echo".as_bytes())?;

    // TODO: Replace this with your client code
    let vsocket = vsock_connect(args.cid, args.port)?;
    let data = "Hello, from client!".to_string();
    send_data(vsocket.as_raw_fd(), data.as_bytes())?;

    std::thread::sleep(Duration::from_secs(3));
    println!("timeout, close socket now...");
    Ok(())
}

/// Accept connections on a certain port and print
/// the received data
pub fn server(args: ServerArgs) -> Result<(), String> {
    let log_port = args.log_port;
    let mut init = false;
    server_with_action(args.port, |buf| {
        let buf = String::from_utf8(buf)
            .map_err(|err| format!("The received bytes are not UTF-8: {:?}", err))?;

        if !init {
            init = true;

            CombinedLogger::init(vec![WriteLogger::new(
                LevelFilter::Info,
                Config::default(),
                LogWriter::new(log_port),
            )])
            .map_err(|e| format!("{:?}", e))?;
            println!("log init successfully");
            info!("log init successfully");
            return Ok(());
        }

        println!("server got message: {}", &buf);
        match buf.as_str() {
            "echo" => info!("{}", &buf),
            _ => debug!("{}", &buf),
        }

        Ok(())
    })
}
