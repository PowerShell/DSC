// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod process_info;
use std::env;
use std::process::exit;

#[cfg(windows)]
fn print_task_list() {

    unsafe{
        let tl = tasklist::Tasklist::new();
        for i in tl{

            let mut p = process_info::ProcessInfo::new();
            p.pid = i.get_pid();
            p.name = i.get_pname();
            p.cmdline = i.get_cmd_params();

            let json = serde_json::to_string(&p).unwrap();
            println!("{json}");
        }
    }
}

#[cfg(not(windows))]
fn print_task_list() {
    for prc in procfs::process::all_processes().unwrap() {
        
        let mut p = process_info::ProcessInfo::new();
        let proc = prc.as_ref().expect("Can't get all_processes");
        p.pid = proc.pid() as u32;
        p.name = match proc.exe() {
            Ok(exe) => { String::from(exe.file_name().expect("Can't get process filename").to_str().unwrap()) }
            Err(_) => { String::from("") }
        };

        if p.name.is_empty() { continue; };

        p.cmdline = match proc.cmdline() {
            Ok(cmdline_vector) => { cmdline_vector.join(" ") }
            Err(_) => { String::from("") }
        };

        let json = serde_json::to_string(&p).unwrap();
        println!("{json}");
    }
}

fn help() {
    println!("usage: process list");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        // one argument passed
        2 => {
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
        },
        // all the other cases
        _ => {
            help();
            exit(1);
        }
    }
}