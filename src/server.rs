use super::common::{FromClientMessage, FromServerMessage};

use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self};

use std::collections::HashMap;
use std::net::SocketAddr;

struct ClientInfo {
    count: usize,
}

pub fn run(transport: Transport, addr: SocketAddr) {
    let (handler, listener) = node::split::<()>();

    let mut clients: HashMap<Endpoint, ClientInfo> = HashMap::new();

    match handler.network().listen(transport, addr) {
        Ok((_resource_id, real_addr)) => {
            println!("Server running at {} by {}", real_addr, transport)
        }
        Err(err) => return println!("Can not listen at {} by {} ({})", addr, transport, err),
    }

    listener.for_each(move |event| match event.network() {
        NetEvent::Message(endpoint, input_data) => {
            let mut status: i32 = 0;
            let message: FromClientMessage = match bincode::deserialize(&input_data) {
                Ok(val) => val,
                Err(_) => FromClientMessage::InvalidConnection,
            };
            match message {
                FromClientMessage::InvalidConnection => {
                    handler.network().remove(endpoint.resource_id());
                    clients.remove(&endpoint);
                    println!(
                        "Client ({}) severed because of an error (total clients: {})",
                        endpoint.addr(),
                        clients.len()
                    );
                }
                FromClientMessage::Ping => {
                    let message = match clients.get_mut(&endpoint) {
                        Some(client) => {
                            // For connection oriented protocols
                            if client.count >= 9 {
                                client.count += 1;

                                println!(
                                    "Ping from {}, {} times, severing connection...",
                                    endpoint.addr(),
                                    client.count
                                );

                                status = 1;

                                FromServerMessage::Pong(endpoint.addr().to_string(), client.count)
                            } else {
                                client.count += 1;
                                println!("Ping from {}, {} times", endpoint.addr(), client.count);
                                FromServerMessage::Pong(endpoint.addr().to_string(), client.count)
                            }
                        }
                        None => {
                            // For non-connection oriented protocols
                            println!("Ping from {}", endpoint.addr());
                            FromServerMessage::UnknownPong
                        }
                    };
                    let output_data = bincode::serialize(&message).unwrap();
                    handler.network().send(endpoint, &output_data);
                }
            }

            if status == 1 {
                handler.network().remove(endpoint.resource_id());
                clients.remove(&endpoint).unwrap();
                println!(
                    "Client ({}) disconnected (total clients: {})",
                    endpoint.addr(),
                    clients.len()
                );
            }
        }
        NetEvent::Connected(endpoint, _) => {
            // Only connection oriented protocols will generate this event
            clients.insert(endpoint, ClientInfo { count: 0 });
            println!(
                "Client ({}) connected (total clients: {})",
                endpoint.addr(),
                clients.len()
            );
        }
        NetEvent::Disconnected(endpoint) => {
            // Only connection oriented protocols will generate this event
            clients.remove(&endpoint).unwrap();
            println!(
                "Client ({}) disconnected (total clients: {})",
                endpoint.addr(),
                clients.len()
            );
        }
    });
}
