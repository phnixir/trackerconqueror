#![forbid(unsafe_code)]

use super::common::{ClientMessage, ServerMessage};

use message_io::network::{NetEvent, RemoteAddr, Transport};
use message_io::node::{self, NodeEvent};

use std::time::Duration;

#[derive(Clone)]
pub enum Signal {
    GetUnit,
    TakeUnit(u32, String), // unit_id, worker_name
    CompleteUnit(u32, String), // unti_id, worker_name
    Disconnect,
}

pub fn run(transport: Transport, remote_addr: RemoteAddr, worker_name: String, sig_op: Signal) {
    let (handler, listener) = node::split();
    let mut openunit = 0;
    let request = match sig_op {
        Signal::GetUnit => "getunit",
        Signal::CompleteUnit(_, _) => "complete",
        _ => "invalid",
    };
    let mut sv_status = "denied";

    let server_id = match handler.network().connect(transport, remote_addr.clone()) {
        Ok((server_id, local_addr)) => {
            println!(
                "Trying to connect to the server by {} at {}",
                transport,
                server_id.addr()
            );
            println!("Client identified by local port: {}", local_addr.port());
            server_id
        }
        Err(err) => {
            return println!(
                "Can not connect to the server by {} to {}. ({})",
                transport, remote_addr, err
            )
        }
    };

    listener.for_each(move |event| match event {
        NodeEvent::Network(net_event) => match net_event {
            NetEvent::Message(_, input_data) => {
                let message: ServerMessage = bincode::deserialize(&input_data).unwrap();
                match message {
                    ServerMessage::Unit(unit_id) => {
                        println!("Unit \"{}\" is reported vacant", unit_id);
                        handler.signals().send(Signal::TakeUnit(unit_id, worker_name.clone()));
                        println!("Requested work on unit \"{}\"", unit_id);
                        openunit = unit_id;
                    }
                    ServerMessage::Accepted => {
                        println!("Server accepted client request");
                        sv_status = "accepted";
                    }
                    ServerMessage::Denied => {
                        println!("Server denied client request, Nothing was changed server-side");
                        sv_status = "denied";
                    }
                    ServerMessage::Unknown => panic!("If code flow reaches this part file a bug report, pleaseee"),
                }
            }
            NetEvent::Accepted(_, _) => unreachable!(), // Only generated when a listener accepts
            NetEvent::Connected(_, _) => {
                println!("Connected");
                handler.signals().send(sig_op.clone());
                handler.signals().send_with_timer(Signal::Disconnect, Duration::from_secs(10));
            }
            NetEvent::Disconnected(_) => {
                println!("Server is disconnected");
                handler.stop();
            }
        },
        NodeEvent::Signal(signal) => match signal {
            Signal::GetUnit => {
                let message = ClientMessage::RequestOpenWorkUnit;
                let output_data = bincode::serialize(&message).unwrap();
                handler.network().send(server_id, &output_data);
            }
            Signal::TakeUnit(unit_id, worker_name) => {
                let message = ClientMessage::Take(unit_id, worker_name);
                let output_data = bincode::serialize(&message).unwrap();
                handler.network().send(server_id, &output_data);
            }
            Signal::CompleteUnit(unit_id, worker_name) => {
                let message = ClientMessage::Complete(unit_id, worker_name);
                let output_data = bincode::serialize(&message).unwrap();
                handler.network().send(server_id, &output_data);
            }
            Signal::Disconnect => {
                print!("{},{},{}", request, sv_status, openunit);
                std::process::exit(0);
            }
        },
    });
}
