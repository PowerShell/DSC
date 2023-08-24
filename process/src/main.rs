// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod process_info;

#[cfg(windows)]
use tasklist;
#[cfg(not(windows))]
use procfs;

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
        let proc = prc.as_ref().expect("proc not found");
        p.pid = proc.pid() as u32;
        p.name = match proc.exe() {
            Ok(exe) => { String::from(exe.file_name().expect("Can't get filename").to_str().unwrap()) }
            Err(_) => { String::from("") }
        };


        p.cmdline = match proc.cmdline() {
            Ok(cmdline_vector) => {
                //for arg in cmdline_vector
                //{ println!("{arg}"); 
                //}

                cmdline_vector.join(" ")
            }
            Err(_) => { String::from("") }
        };

	if p.name != ""
        {
           let json = serde_json::to_string(&p).unwrap();
           println!("{json}");
        };
    }
}

fn main() {
    print_task_list();
}
