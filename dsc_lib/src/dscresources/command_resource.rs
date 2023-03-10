use serde_json::Value;

use crate::dscerror::DscError;
use super::resource_manifest::ResourceManifest;
use std::{process::Command, io::Write};
use crate::dscresources::invoke_result::GetResult;

pub const EXIT_PROCESS_TERMINATED: i32 = 0x102;

pub fn invoke_get(resource: &ResourceManifest, filter: &str) -> Result<GetResult, DscError> {
    let mut command = Command::new(&resource.get.executable);
    if let Some(args) = &resource.get.args {
        command.args(args);
    }

    let mut child = command.spawn()?;
    let exit_status = child.wait()?;
    // pipe to child stdin
    let mut child_stdin = child.stdin.take().unwrap();
    child_stdin.write_all(filter.as_bytes())?;

    let output = command.output()?;
    let stdout = String::from_utf8(output.stdout).unwrap_or(String::new());
    let stderr = String::from_utf8(output.stderr).unwrap_or(String::new());
    let exit_code = exit_status.code().unwrap_or(EXIT_PROCESS_TERMINATED) as i32;
    if exit_code != 0 || !stderr.is_empty(){
        return Err(DscError::Command(exit_code, stderr));
    }

    let result: Value = serde_json::from_str(&stdout)?;
    Ok(GetResult {
        actual_state: result,
    })
}
