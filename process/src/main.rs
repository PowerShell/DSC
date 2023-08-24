// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod process_info;
use std::env;
use std::process::exit;
use sysinfo::{ProcessExt, System, SystemExt, PidExt};

fn print_task_list() {

    let mut s = System::new();
    s.refresh_processes();
    for (pid, process) in s.processes() {
        let mut p = process_info::ProcessInfo::new();
        p.pid = pid.as_u32();
        p.name = String::from(process.name());
        p.cmdline = format!("{:?}", process.cmd());

        let json = serde_json::to_string(&p).unwrap();
        println!("{json}");
    }
}

fn help() {
    println!("usage: process list");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        // one argument passed
        match args[1].as_str() {
            "list" => {
                print_task_list();
                exit(0);
            },
            "set" => { // used for testing only
                println!("{{\"result\":\"Ok\"}}");
                exit(0);
            },
            _ => {
                help();
                exit(1);
            },
        }
    }
    else {
        help();
        exit(1);
    }
}