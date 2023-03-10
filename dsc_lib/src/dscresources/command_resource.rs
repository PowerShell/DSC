use serde_json::Value;

use crate::dscerror::DscError;
use super::resource_manifest::ResourceManifest;
use std::{process::Command, io::{Write, Read}, process::Stdio};
use crate::dscresources::invoke_result::GetResult;

pub const EXIT_PROCESS_TERMINATED: i32 = 0x102;

pub fn invoke_get(resource: &ResourceManifest, filter: &str) -> Result<GetResult, DscError> {
    let mut command = Command::new(&resource.get.executable);
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    if let Some(args) = &resource.get.args {
        command.args(args);
    }

    let mut child = command.spawn()?;
    {
        // pipe to child stdin in a scope so that it is dropped before we wait
        let mut child_stdin = child.stdin.take().unwrap();
        child_stdin.write_all(filter.as_bytes())?;
        child_stdin.flush()?;
    }
    let exit_status = child.wait()?;

    let mut child_stdout = child.stdout.take().unwrap();
    let mut child_stderr = child.stderr.take().unwrap();
    let mut stdout_buf = Vec::new();
    child_stdout.read_to_end(&mut stdout_buf)?;
    let mut stderr_buf = Vec::new();
    child_stderr.read_to_end(&mut stderr_buf)?;

    let exit_code = exit_status.code().unwrap_or(EXIT_PROCESS_TERMINATED) as i32;
    if exit_code != 0 || !stderr_buf.is_empty(){
        return Err(DscError::Command(exit_code, String::from_utf8_lossy(&stderr_buf).to_string()));
    }

    let result: Value = serde_json::from_str(&String::from_utf8_lossy(&stdout_buf))?;
    Ok(GetResult {
        actual_state: result,
    })
}
