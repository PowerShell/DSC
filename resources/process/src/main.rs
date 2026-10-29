// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod process_info;
use std::env;
use std::process::ExitCode;
use std::io::{self, Read};
use sysinfo::{ProcessRefreshKind, RefreshKind, System, UpdateKind};
use crate::process_info::ProcessInfo;

fn get_task_list() -> Result<Vec<ProcessInfo>, ExitCode>
{
    let mut result = Vec::new();
    let Ok(s) = System::new_with_specifics(RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing().with_cmd(UpdateKind::Always))) else {
        return Err(ExitCode::FAILURE);
    };
    let mut count = 0;
    for (pid, process) in s.processes() {
        let mut p = ProcessInfo::new();
        p.pid = pid.as_u32();
        p.name = process.name().to_string_lossy().to_string();
        p.cmdline = format!("{:?}", process.cmd());
        result.push(p);
        count += 1;
        if count > 3 {
            // limit to 3 processes as this is just for testing
            break;
        }
    }

    Ok(result)
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

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        // one argument passed
        match args[1].as_str() {
            "list" => {
                match get_task_list() {
                    Ok(processes) => {
                        for p in processes {
                            let json = serde_json::to_string(&p).unwrap();
                            println!("{json}");
                        }
                        ExitCode::SUCCESS
                    }
                    Err(code) => code,
                }
            },
            "get" | "set" | "test" => { // used for testing only
                print_input();
                ExitCode::SUCCESS
            },
            _ => {
                help();
                ExitCode::FAILURE
            },
        }
    }
    else {
        help();
        ExitCode::FAILURE
    }
}