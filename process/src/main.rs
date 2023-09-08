// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod process_info;
use std::env;
use std::process::exit;
use std::io::{self, Read};
use sysinfo::{ProcessExt, System, SystemExt, PidExt};
use crate::process_info::ProcessInfo;

fn get_task_list() -> Vec<ProcessInfo>
{
    let mut result = Vec::new();
    let mut s = System::new();
    s.refresh_processes();
    for (pid, process) in s.processes() {
        let mut p = ProcessInfo::new();
        p.pid = pid.as_u32();
        p.name = String::from(process.name());
        p.cmdline = format!("{:?}", process.cmd());
        result.push(p);
    }

    result
}

fn help() {
    println!("usage: process list");
}

fn print_input() {
    let mut buffer: Vec<u8> = Vec::new();
    io::stdin().read_to_end(&mut buffer).unwrap();
    let input = String::from_utf8(buffer);
    println!("{}", input.unwrap());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        // one argument passed
        match args[1].as_str() {
            "list" => {
                for p in get_task_list()
                {
                    let json = serde_json::to_string(&p).unwrap();
                    println!("{json}");
                }
                exit(0);
            },
            "get" | "set" | "test" => { // used for testing only
                print_input();
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