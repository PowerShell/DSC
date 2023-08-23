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

}

fn main() {
    print_task_list();
}
