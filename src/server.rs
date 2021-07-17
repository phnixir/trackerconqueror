#![forbid(unsafe_code)]

use super::common::{ClientMessage, ServerMessage, WorkStatus, WorkUnit};

use message_io::network::{Endpoint, NetEvent, Transport};
use message_io::node::{self};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;

use colored::*;

const MAX_WORK_UNITS: u32 = 192;
const PKG_VER: &str = env!("CARGO_PKG_VERSION");

struct ClientInfo {}

fn progress_rewrite(csv_vec: Vec<WorkUnit>) -> Vec<WorkUnit> {
    let mut wtr = csv::Writer::from_path("progress.csv").unwrap();
    for item in csv_vec {
        wtr.serialize(item).unwrap();
    }
    wtr.flush().unwrap();

    let mut records: Vec<WorkUnit> = Vec::with_capacity(MAX_WORK_UNITS as usize);
    let mut rdr = csv::Reader::from_path("progress.csv").unwrap();
    for result in rdr.records() {
        let record: WorkUnit = result.unwrap().deserialize(None).unwrap();
        records.push(record);
    }
    records
}

pub fn run(transport: Transport, addr: SocketAddr) {
    let mut records: Vec<WorkUnit> = Vec::with_capacity(MAX_WORK_UNITS as usize);
    if Path::new("progress.csv").is_file() {
        let mut rdr = csv::Reader::from_path("progress.csv").unwrap();
        for result in rdr.records() {
            let record: WorkUnit = result.unwrap().deserialize(None).unwrap();
            records.push(record);
        }
    } else {
        let mut wtr = csv::Writer::from_path("progress.csv").unwrap();
        for i in 1..=MAX_WORK_UNITS {
            wtr.serialize(WorkUnit {
                id: i,
                status: WorkStatus::Vacant,
                current_worker: "nobody".to_string(),
            })
            .unwrap();
        }
        wtr.flush().unwrap();

        let mut rdr = csv::Reader::from_path("progress.csv").unwrap();
        for result in rdr.records() {
            let record: WorkUnit = result.unwrap().deserialize(None).unwrap();
            records.push(record);
        }
    }

    // println!("{:?}", records[60 - 1 /* the id you want to get - 1 */]);

    // records[10 - 1].current_worker = "fat gamer".to_string();
    // records = progress_rewrite(records.clone());

    let (handler, listener) = node::split::<()>();

    let mut clients: HashMap<Endpoint, ClientInfo> = HashMap::new();

    match handler.network().listen(transport, addr) {
        Ok((_resource_id, real_addr)) => {
            println!(
                "{} {}{} {} {} {} {}",
                "Server".bold().cyan(),
                "v".bold().green(),
                PKG_VER.bold(),
                "running at".bold().cyan(),
                real_addr,
                "by".bold().cyan(),
                transport
            )
        }
        Err(err) => {
            return println!(
                "{} {} {} {} ({})",
                "Can not listen at".bold().cyan(),
                addr,
                "by".bold().cyan(),
                transport,
                err
            )
        }
    }

    listener.for_each(move |event| match event.network() {
        NetEvent::Message(endpoint, input_data) => {
            let message: ClientMessage = match bincode::deserialize(&input_data) {
                Ok(val) => val,
                Err(_) => ClientMessage::InvalidConnection,
            };

            match message {
                ClientMessage::InvalidConnection => {
                    handler.network().remove(endpoint.resource_id());
                    clients.remove(&endpoint);
                    println!(
                        "[{}] {} {} {} (total clients: {})",
                        "Connections".bold().magenta(),
                        "Client".bold().cyan(),
                        endpoint.addr(),
                        "severed because of an error".bold().red(),
                        clients.len()
                    );
                }

                // client address: endpoint.addr().to_string()
                ClientMessage::RequestOpenWorkUnit => {
                    let message: ServerMessage = match clients.get_mut(&endpoint) {
                        Some(_client) => {
                            // real code goes here
                            println!(
                                "[{}]    {} ({}) {}",
                                "Requests".bold().blue(),
                                "Client".bold().cyan(),
                                endpoint.addr(),
                                "requested a work unit".bold().bright_green()
                            );
                            let mut found = 0;
                            for item in &records {
                                if item.status == WorkStatus::Vacant {
                                    found = item.id;
                                    println!(
                                        "[{}]   {} {} {} {}",
                                        "Responses".bold().red(),
                                        "Found Vacant unit".bold().green(),
                                        found,
                                        "for".bold().green(),
                                        endpoint.addr()
                                    );
                                    break;
                                }
                            }

                            if found != 0 {
                                println!(
                                    "[{}]   {} {} {} {}",
                                    "Responses".bold().red(),
                                    "Sent unit".bold().cyan(),
                                    found,
                                    "to".bold().cyan(),
                                    endpoint.addr()
                                );
                                ServerMessage::Unit(found)
                            } else {
                                println!(
                                    "[{}]   {} {}{}",
                                    "Responses".bold().red(),
                                    "Denied".bold().bright_red(),
                                    endpoint.addr(),
                                    "'s work unit request because no vacant unit was found"
                                        .bold()
                                        .bright_red()
                                );
                                ServerMessage::Denied
                            }
                        }
                        None => ServerMessage::Unknown,
                    };

                    let output_data = bincode::serialize(&message).unwrap();
                    handler.network().send(endpoint, &output_data);
                }
                ClientMessage::Take(work_unit_id, client_name) => {
                    let message = match clients.get_mut(&endpoint) {
                        Some(_client) => {
                            println!(
                                "[{}]    {} ({} a.k.a \"{}\") {} {}",
                                "Requests".bold().blue(),
                                "Client".bold().cyan(),
                                endpoint.addr(),
                                client_name.yellow(),
                                "wants to work on unit".bold().cyan(),
                                work_unit_id
                            );
                            if records[work_unit_id as usize - 1].status == WorkStatus::Vacant {
                                records[work_unit_id as usize - 1].status = WorkStatus::Working;
                                records[work_unit_id as usize - 1].current_worker =
                                    client_name.clone();
                                records = progress_rewrite(records.clone());

                                println!(
                                    "[{}]   {} ({} a.k.a \"{}\")",
                                    "Responses".bold().red(),
                                    "Accepted work request from client".bold().green(),
                                    endpoint.addr(),
                                    &client_name.yellow()
                                );
                                ServerMessage::Accepted
                            } else {
                                println!(
                                    "[{}]   {} ({} a.k.a \"{}\")",
                                    "Responses".bold().red(),
                                    "Denied work request from client".bold().bright_red(),
                                    endpoint.addr(),
                                    &client_name.yellow()
                                );
                                ServerMessage::Denied
                            }
                        }
                        None => ServerMessage::Unknown,
                    };

                    let output_data = bincode::serialize(&message).unwrap();
                    handler.network().send(endpoint, &output_data);
                }
                ClientMessage::Complete(work_unit_id, client_name) => {
                    let message = match clients.get_mut(&endpoint) {
                        Some(_client) => {
                            println!(
                                "[{}]    {} ({} a.k.a {}) {} {} {}",
                                "Requests".bold().blue(),
                                "Client".bold().cyan(),
                                endpoint.addr(),
                                client_name.yellow(),
                                "wants to declare unit".bold().cyan(),
                                work_unit_id,
                                "as completed".bold().cyan()
                            );
                            if (records[work_unit_id as usize - 1].status == WorkStatus::Working)
                                && (records[work_unit_id as usize - 1].current_worker
                                    == client_name)
                            {
                                records[work_unit_id as usize - 1].status = WorkStatus::Complete;
                                records = progress_rewrite(records.clone());

                                println!(
                                    "[{}]   {} ({} a.k.a \"{}\")",
                                    "Responses".bold().red(),
                                    "Accepted completion request from client".bold().green(),
                                    endpoint.addr(),
                                    &client_name.yellow()
                                );
                                ServerMessage::Accepted
                            } else {
                                println!(
                                    "[{}]   {} ({} a.k.a \"{}\")",
                                    "Responses".bold().red(),
                                    "Denied completion request from client".bold().bright_red(),
                                    endpoint.addr(),
                                    &client_name.yellow()
                                );
                                ServerMessage::Denied
                            }
                        }
                        None => ServerMessage::Unknown,
                    };

                    let output_data = bincode::serialize(&message).unwrap();
                    handler.network().send(endpoint, &output_data);
                }
            }
        }
        NetEvent::Accepted(endpoint, _) => {
            // Only connection oriented protocols will generate this event
            clients.insert(endpoint, ClientInfo {});
            println!(
                "[{}] {} ({}) {} (total clients: {})",
                "Connections".bold().magenta(),
                "Client".bold().cyan(),
                endpoint.addr(),
                "connected".bold().italic().green(),
                clients.len()
            );
        }
        NetEvent::Connected(_, _) => todo!(),
        NetEvent::Disconnected(endpoint) => {
            // Only connection oriented protocols will generate this event
            clients.remove(&endpoint).unwrap();
            println!(
                "[{}] {} ({}) {} (total clients: {})",
                "Connections".bold().magenta(),
                "Client".bold().cyan(),
                endpoint.addr(),
                "disconnected".bold().italic().bright_red(),
                clients.len()
            );
        }
    });
}
