// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use regex::{Regex, RegexBuilder};
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, Process, ProcessRefreshKind, RefreshKind, System, UpdateKind};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmdline: Option<Vec<String>>,
}

impl ProcessInfo {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.as_ref().is_some_and(String::is_empty) {
            return Err(t!("input.emptyName").to_string());
        }
        Ok(())
    }

    pub fn validate_identity(&self) -> Result<(), String> {
        self.validate()?;
        if self.pid.is_none() && self.name.is_none() {
            return Err(t!("input.missingIdentity").to_string());
        }
        if self.name.as_ref().is_some_and(|name| name.contains('*')) {
            return Err(t!("input.wildcardOnlyExport").to_string());
        }
        Ok(())
    }
}

fn process_info(pid: Pid, process: &Process) -> ProcessInfo {
    ProcessInfo {
        pid: Some(pid.as_u32()),
        name: Some(process.name().to_string_lossy().into_owned()),
        cmdline: Some(
            process
                .cmd()
                .iter()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect(),
        ),
    }
}

pub fn snapshot() -> Result<System, String> {
    System::new_with_specifics(
        RefreshKind::nothing().with_processes(
            ProcessRefreshKind::nothing()
                .without_tasks()
                .with_cmd(UpdateKind::Always),
        ),
    )
    .map_err(|error| t!("process.enumerationFailed", error = error).to_string())
}

fn name_filter(pattern: &str) -> Result<Regex, String> {
    let pattern = regex::escape(pattern).replace("\\*", ".*");
    RegexBuilder::new(&format!(r"\A{pattern}\z"))
        .case_insensitive(cfg!(windows))
        .dot_matches_new_line(true)
        .build()
        .map_err(|error| t!("input.invalidFilter", error = error).to_string())
}

pub fn matching(system: &System, filter: &ProcessInfo) -> Result<Vec<ProcessInfo>, String> {
    let name = filter.name.as_deref().map(name_filter).transpose()?;
    let mut processes = Vec::new();
    for (pid, process) in system.processes() {
        if filter.pid.is_some_and(|value| value != pid.as_u32()) {
            continue;
        }
        let info = process_info(*pid, process);
        if name
            .as_ref()
            .is_some_and(|pattern| !pattern.is_match(info.name.as_deref().unwrap_or_default()))
            || filter
                .cmdline
                .as_ref()
                .is_some_and(|cmdline| info.cmdline.as_ref() != Some(cmdline))
        {
            continue;
        }
        processes.push(info);
    }
    processes.sort_by_key(|process| process.pid);
    Ok(processes)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TerminateInput {
    pub pid: Option<u32>,
    pub name: Option<String>,
}

impl TerminateInput {
    pub fn validate(&self) -> Result<(), String> {
        if self.pid.is_some() == self.name.is_some() {
            return Err(t!("input.exactlyOneIdentity").to_string());
        }
        let filter = self.filter();
        filter.validate_identity()?;
        if self.pid == Some(0) || self.pid == Some(std::process::id()) {
            return Err(t!("terminate.unsafeTarget").to_string());
        }
        Ok(())
    }

    pub fn filter(&self) -> ProcessInfo {
        ProcessInfo {
            pid: self.pid,
            name: self.name.clone(),
            cmdline: None,
        }
    }
}

pub fn terminate(system: &System, input: &TerminateInput) -> Result<Vec<ProcessInfo>, String> {
    let processes = matching(system, &input.filter())?;
    if processes.is_empty() {
        return Err(t!("process.notFound").to_string());
    }
    if processes
        .iter()
        .any(|process| process.pid == Some(0) || process.pid == Some(std::process::id()))
    {
        return Err(t!("terminate.unsafeTarget").to_string());
    }
    let mut failed = Vec::new();
    for info in &processes {
        if let Some(pid) = info.pid {
            let process = system
                .process(Pid::from_u32(pid))
                .ok_or_else(|| t!("process.notFound").to_string())?;
            if !process.kill() {
                failed.push(pid.to_string());
            }
        }
    }
    if !failed.is_empty() {
        return Err(t!("terminate.failed", pids = failed.join(", ")).to_string());
    }
    Ok(processes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_matches_only_asterisks() {
        for (pattern, text, expected) in [
            ("*", "process", true),
            ("p*ss", "process", true),
            ("*roc*", "process", true),
            ("p**r*s", "process", true),
            ("process", "process-extra", false),
            ("p?ocess", "process", false),
            ("p[ro]*", "process", false),
            ("process*process", "process", false),
            ("*\u{e9}*", "proc\u{e9}s", true),
            ("PROC*", "process", cfg!(windows)),
        ] {
            assert_eq!(
                name_filter(pattern).unwrap().is_match(text),
                expected,
                "{pattern}"
            );
        }
    }

    #[test]
    fn validates_identities_without_terminating_anything() {
        for json in [
            "{}",
            r#"{"pid":0}"#,
            r#"{"name":""}"#,
            r#"{"name":"*"}"#,
            r#"{"pid":1,"name":"process"}"#,
            r#"{"pid":-1}"#,
            r#"{"pid":4294967296}"#,
            r#"{"pid":1,"cmdline":[]}"#,
        ] {
            assert!(
                serde_json::from_str::<TerminateInput>(json)
                    .map_or(true, |input| input.validate().is_err()),
                "{json}"
            );
        }
        assert!(
            TerminateInput {
                pid: Some(std::process::id()),
                name: None
            }
            .validate()
            .is_err()
        );
        assert!(
            TerminateInput {
                pid: None,
                name: Some("example".to_string())
            }
            .validate()
            .is_ok()
        );
    }

    #[test]
    fn filters_snapshot_by_pid_and_name() {
        let system = snapshot().unwrap();
        let filter = ProcessInfo {
            pid: Some(std::process::id()),
            ..Default::default()
        };
        let result = matching(&system, &filter).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pid, filter.pid);
        let filter = ProcessInfo {
            name: Some("dsc-nonexistent-process-36d97d78".to_string()),
            ..filter
        };
        assert!(matching(&system, &filter).unwrap().is_empty());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn snapshot_excludes_threads() {
        let barrier = std::sync::Barrier::new(2);
        std::thread::scope(|scope| {
            let (sender, receiver) = std::sync::mpsc::channel();
            let barrier = &barrier;
            scope.spawn(move || {
                let tid = std::fs::read_link("/proc/thread-self")
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .parse::<u32>()
                    .unwrap();
                sender.send(tid).unwrap();
                barrier.wait();
            });
            let tid = receiver.recv().unwrap();
            let system = snapshot();
            barrier.wait();
            let system = system.unwrap();
            assert!(system.process(Pid::from_u32(std::process::id())).is_some());
            assert!(system.process(Pid::from_u32(tid)).is_none());
        });
    }
}
