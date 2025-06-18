// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use registry_lib::{config::{Registry, RegistryValueData}, RegistryHelper};

use crate::args::DefaultShell;
use crate::error::SshdConfigError;

/// Invoke the get command.
///
/// # Errors
///
/// This function will return an error if the desired settings cannot be retrieved.
pub fn invoke_get() -> Result<(), SshdConfigError> {
    // TODO: distinguish between get commands for default shell, repeatable keywords, and sshd_config
    get_default_shell()?;
    Ok(())
}

fn get_default_shell() -> Result<(), SshdConfigError> {
    let registry_helper = RegistryHelper::new("HKLM\\SOFTWARE\\OpenSSH", Some("DefaultShell".to_string()), None)?;
    let default_shell: Registry = registry_helper.get()?;
    let mut shell = None;
    let mut shell_arguments = None;
    if let Some(value) = default_shell.value_data {
        match value {
            RegistryValueData::String(s) => {
                let parts: Vec<&str> = s.split_whitespace().collect();
                if parts.is_empty() {
                    return Err(SshdConfigError::InvalidInput("DefaultShell cannot be empty".to_string()));
                }
                shell = Some(parts[0].to_string());
                if parts.len() > 1 {
                    shell_arguments = Some(parts[1..].iter().map(|&s| s.to_string()).collect());
                }
            }
            _ => return Err(SshdConfigError::InvalidInput("DefaultShell must be a string".to_string())),
        }
    }

    let registry_helper = RegistryHelper::new("HKLM\\SOFTWARE\\OpenSSH", Some("DefaultShellCommandOption".to_string()), None)?;
    let option: Registry = registry_helper.get()?;
    let mut cmd_option = None;
    if let Some(value) = option.value_data {
        match value {
            RegistryValueData::String(s) => cmd_option = Some(s),
            _ => return Err(SshdConfigError::InvalidInput("DefaultShellCommandOption must be a string".to_string())),
        }
    }

    let registry_helper = RegistryHelper::new("HKLM\\SOFTWARE\\OpenSSH", Some("DefaultShellEscapeArguments".to_string()), None)?;
    let escape_args: Registry = registry_helper.get()?;
    let mut escape_arguments = None;
    if let Some(value) = escape_args.value_data {
        if let RegistryValueData::DWord(b) = value {
            if b == 0 || b == 1 {
                escape_arguments = if b == 1 { Some(true) } else { Some(false) };
            } else {
                return Err(SshdConfigError::InvalidInput("DefaultShellEscapeArguments must be a boolean".to_string()));
            }
        } else {
            return Err(SshdConfigError::InvalidInput("DefaultShellEscapeArguments must be a boolean".to_string()));
        }
    }

    let result = DefaultShell {
        shell,
        cmd_option,
        escape_arguments,
        shell_arguments
    };

    let output = serde_json::to_string_pretty(&result)?;
    println!("{output}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use registry_lib::config::RegistryValueData;

    #[test]
    fn test_parse_shell_command() {
        let (shell, args) = parse_shell_command(r#"C:\Program Files\PowerShell\pwsh.exe -NoProfile"#);
        assert_eq!(shell, r#"C:\Program Files\PowerShell\pwsh.exe"#);
        assert_eq!(args, vec!["-NoProfile"]);
    }

    #[test]
    fn test_parse_shell_command_with_quotes() {
        let (shell, args) = parse_shell_command(r#""C:\Program Files\PowerShell\pwsh.exe" -NoProfile -Command"#);
        assert_eq!(shell, r#"C:\Program Files\PowerShell\pwsh.exe"#);
        assert_eq!(args, vec!["-NoProfile", "-Command"]);
    }

    #[test]
    fn test_extract_string_value_string() {
        let value = RegistryValueData::String("test".to_string());
        let result = extract_string_value(value, "test_field").unwrap();
        assert_eq!(result, Some("test".to_string()));
    }

    #[test]
    fn test_extract_string_value_multistring() {
        let value = RegistryValueData::MultiString(vec!["first".to_string(), "second".to_string()]);
        let result = extract_string_value(value, "test_field").unwrap();
        assert_eq!(result, Some("first".to_string()));
    }
}