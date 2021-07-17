#![forbid(unsafe_code)]
use std::process::Command;
use trackermeta::scraper::requests;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::mem;

#[derive(Debug, PartialEq)]
enum SelfExecRequest {
    GetUnit,
    Complete,
}

#[derive(Debug, PartialEq)]
enum SelfExecStatus {
    Accepted,
    Denied,
}

#[derive(Debug, PartialEq)]
struct SelfExecResult {
    request: SelfExecRequest,
    status: SelfExecStatus,
    unit: u32,
}

fn string_to_ser(string: String) -> SelfExecResult {
    let mut ser = SelfExecResult { 
        request: SelfExecRequest::GetUnit,
        status: SelfExecStatus::Denied, 
        unit: 0,
    };

    ser.request = match string.split(',').nth(0).unwrap() {
       "getunit" => SelfExecRequest::GetUnit,
       "complete" => SelfExecRequest::Complete,
       _ => panic!("value other than \"getunit\" or \"complete\""),
    };

    ser.status = match string.split(',').nth(1).unwrap() {
        "accepted" => SelfExecStatus::Accepted,
        "denied" => SelfExecStatus::Denied,
        _ => panic!("value other than \"accepted\" or \"denied\""),
    };

    ser.unit = string.split(',').nth(2).unwrap().parse().unwrap();

    ser
}

fn get_ser_line(stdout: Vec<u8>) -> String {
    std::str::from_utf8(&stdout).unwrap().split('\n').last().unwrap().to_string()
}

fn getunit(exec_path: String, worker_name: String, remote_addr: String) -> SelfExecResult {
    let output = Command::new(exec_path)
        .args(&["client", &worker_name, &remote_addr, "getunit"])
        .output()
        .expect("this error should be impossible, please file a bug report if you see this");
    
    let ser = get_ser_line(output.stdout);
    let ser = string_to_ser(ser);

    ser
}

fn complete(exec_path: String, worker_name: String, remote_addr: String, unit_id: u32) -> SelfExecResult {
    let output = Command::new(exec_path)
        .args(&["client", &worker_name, &remote_addr, "complete", &format!("{}", unit_id)])
        .output()
        .expect("this error should be impossible, please file a bug report if you see this");
    
    let ser = get_ser_line(output.stdout);
    let ser = string_to_ser(ser);

    ser
}

pub fn run(worker_name: String, remote_addr: String, start_from: u32, custom_unit_id: u32) {
    let args: Vec<String> = std::env::args().collect();
    let mut unit_id;
    let mut mod_counter;
    let arg0 = args.get(0).unwrap().to_string();
    //println!("{}: {},{},{}", arg0, worker_name, remote_addr, start_from);

    if custom_unit_id != 0 {
        unit_id = custom_unit_id;

        let unit_min = ((unit_id - 1) * 1000) + 1;
        let unit_max = (unit_id * 1000) + 1;
        mod_counter = start_from;

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(format!("{}.unit", unit_id))
            .unwrap();

        while (mod_counter >= unit_min) && !(mod_counter == unit_max) {
            let record = requests::get_full_details_as_string(mod_counter);
            mod_counter += 1;
            println!("{}", record);
            writeln!(file, "{}", record).unwrap();
        }

        mem::drop(file);

        println!("unit {} completed, signaling server...", unit_id);

        let resp = complete(arg0.clone(), worker_name.clone(), remote_addr.clone(), unit_id);

        if (resp.request == SelfExecRequest::Complete) && (resp.status == SelfExecStatus::Accepted) {
            println!("server accepted complete request on unit \"{}\"", unit_id);
        }

        println!("emergency continuation job done, moving on...");
    }

    loop {

        let req = getunit(arg0.clone(), worker_name.clone(), remote_addr.clone());
        unit_id = match req.status {
            SelfExecStatus::Accepted => req.unit,
            SelfExecStatus::Denied => panic!("NO VACANT UNIT LEFT! / SERVER OFFLINE!"),
        };
        let unit_min = ((unit_id - 1) * 1000) + 1;
        let unit_max = (unit_id * 1000) + 1;
        mod_counter = unit_min;

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(format!("{}.unit", unit_id))
            .unwrap();

        while (mod_counter >= unit_min) && !(mod_counter == unit_max) {
            let record = requests::get_full_details_as_string(mod_counter);
            mod_counter += 1;
            println!("{}", record);
            writeln!(file, "{}", record).unwrap();
        }

        mem::drop(file);

        println!("unit {} completed, signaling server...", unit_id);

        let resp = complete(arg0.clone(), worker_name.clone(), remote_addr.clone(), unit_id);

        if (resp.request == SelfExecRequest::Complete) && (resp.status == SelfExecStatus::Accepted) {
            println!("server accepted complete request on unit \"{}\"", unit_id);
        }

    }
}
