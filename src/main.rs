#![forbid(unsafe_code)]

mod client;
mod common;
mod server;
mod miner;

use message_io::network::{ToRemoteAddr, Transport};
use crate::client::Signal;

use std::net::ToSocketAddrs;

const HELP_MSG: &str = concat!(
    "Usage: trackerconqueror server <port>\n",
    "       trackerconqueror client <worker name> (<server ip>:<port> | url) <getunit | complete <unit id>>\n",
    "       trackerconqueror miner <worker name> (<server ip>:<port>)"
);

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    let transport = Transport::FramedTcp; // The non-streamed version of tcp.

    match args.get(1).unwrap_or(&"".into()).as_ref() {
        "client" => match args.get(3) {
            Some(remote_addr) => {
                let remote_addr = remote_addr.to_remote_addr().unwrap();
                match args.get(4).expect("No operation provided (4th argument)").to_lowercase().as_str() {
                    "getunit" => {
                        let signal = Signal::GetUnit;
                        let worker_name = args.get(2).unwrap().to_string();
                        client::run(transport, remote_addr, worker_name.clone(), signal)
                    }
                    "complete" => {
                        let worker_name = args.get(2).unwrap().to_string();
                        let unit_id = args.get(5).expect("No unit id provided").parse::<u32>().expect("Unit id is NOT a digit");
                        let signal = Signal::CompleteUnit(unit_id, worker_name.clone());
                        client::run(transport, remote_addr, worker_name.clone(), signal)
                    }
                    _ => return println!("{}", HELP_MSG),
                }
            }
            None => return println!("{}", HELP_MSG),
        },
        "miner" => match args.clone().get(3) {
            Some(remote_addr) => {
                let worker_name = args.get(2).unwrap().clone();
                let start_from: u32 = match args.get(4) {
                    Some(arg) => arg.parse().unwrap(),
                    None => 0,
                };
                let custom_unit_id: u32 = match args.get(5) {
                    Some(arg) => arg.parse().unwrap(),
                    None => 0,
                };
                miner::run(worker_name, remote_addr.to_string(), start_from, custom_unit_id);
            }
            None => return println!("{}", HELP_MSG),
        }
        "server" => {
            match args.get(2).unwrap_or(&"".into()).parse() {
                Ok(port) => {
                    let addr = ("0.0.0.0", port).to_socket_addrs().unwrap().next().unwrap();
                    server::run(transport, addr);
                }
                Err(_) => return println!("{}", HELP_MSG),
            };
        }
        _ => return println!("{}", HELP_MSG),
    }
}
