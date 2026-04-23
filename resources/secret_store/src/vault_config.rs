// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types::{VaultConfig, Authentication, Interaction};
use rust_i18n::t;
use std::process::Command;

/// Run a PowerShell command and return its stdout, or an error with stderr.
fn run_pwsh(script: &str) -> Result<String, String> {
    let output = Command::new("pwsh")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| t!("vault_config.pwshNotFound").to_string() + ": " + &e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Check whether a PowerShell module is installed.
fn is_module_installed(module_name: &str) -> Result<bool, String> {
    let script = format!(
        "if (Get-Module -ListAvailable -Name '{module_name}') {{ 'true' }} else {{ 'false' }}"
    );
    let result = run_pwsh(&script)?;
    Ok(result.trim().eq_ignore_ascii_case("true"))
}

/// Get the current SecretStore vault configuration.
pub fn get_config(input: &VaultConfig) -> Result<VaultConfig, String> {
    let _ = input; // input is accepted for schema consistency but not used for get

    let sm_installed = is_module_installed("Microsoft.PowerShell.SecretManagement")
        .unwrap_or(false);
    let ss_installed = is_module_installed("Microsoft.PowerShell.SecretStore")
        .unwrap_or(false);

    if !sm_installed || !ss_installed {
        return Ok(VaultConfig {
            secret_management_installed: Some(sm_installed),
            secret_store_installed: Some(ss_installed),
            _exist: Some(false),
            ..Default::default()
        });
    }

    // Check if vault is registered
    let vault_registered = run_pwsh(
        "if (Get-SecretVault -Name 'SecretStore' -ErrorAction SilentlyContinue) { 'true' } else { 'false' }"
    ).map(|r| r.trim().eq_ignore_ascii_case("true"))
    .unwrap_or(false);

    // Get configuration
    let config_json = run_pwsh(
        "Get-SecretStoreConfiguration | ConvertTo-Json -Compress"
    ).map_err(|e| t!("vault_config.getConfigFailed", error = e).to_string())?;

    // Parse the PowerShell JSON output
    let ps_config: serde_json::Value = serde_json::from_str(&config_json)
        .map_err(|e| t!("vault_config.getConfigFailed", error = e.to_string()).to_string())?;

    let authentication = ps_config.get("Authentication")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "None" => Authentication::None,
            _ => Authentication::Password,
        });

    let password_timeout = ps_config.get("PasswordTimeout")
        .and_then(|v| v.as_i64())
        .map(|v| v as i32);

    let interaction = ps_config.get("Interaction")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "Prompt" => Interaction::Prompt,
            _ => Interaction::None,
        });

    Ok(VaultConfig {
        authentication,
        password_timeout,
        interaction,
        secret_management_installed: Some(true),
        secret_store_installed: Some(true),
        vault_registered: Some(vault_registered),
        _exist: Some(true),
    })
}

/// Set the SecretStore vault configuration to the desired state.
pub fn set_config(input: &VaultConfig) -> Result<VaultConfig, String> {
    // Ensure modules are installed
    let sm_installed = is_module_installed("Microsoft.PowerShell.SecretManagement")
        .unwrap_or(false);
    let ss_installed = is_module_installed("Microsoft.PowerShell.SecretStore")
        .unwrap_or(false);

    if !sm_installed {
        run_pwsh("Install-Module -Name Microsoft.PowerShell.SecretManagement -Force -Scope CurrentUser")
            .map_err(|e| t!("vault_config.moduleNotInstalled", module = "Microsoft.PowerShell.SecretManagement").to_string() + ": " + &e)?;
    }

    if !ss_installed {
        run_pwsh("Install-Module -Name Microsoft.PowerShell.SecretStore -Force -Scope CurrentUser")
            .map_err(|e| t!("vault_config.moduleNotInstalled", module = "Microsoft.PowerShell.SecretStore").to_string() + ": " + &e)?;
    }

    // Build the Set-SecretStoreConfiguration command
    let mut params = Vec::new();

    if let Some(ref auth) = input.authentication {
        params.push(format!("-Authentication {auth}"));
    }

    if let Some(timeout) = input.password_timeout {
        params.push(format!("-PasswordTimeout {timeout}"));
    }

    if let Some(ref interaction) = input.interaction {
        params.push(format!("-Interaction {interaction}"));
    }

    if !params.is_empty() {
        let script = format!(
            "Set-SecretStoreConfiguration {} -Confirm:$false -Force",
            params.join(" ")
        );
        run_pwsh(&script)
            .map_err(|e| t!("vault_config.setConfigFailed", error = e).to_string())?;
    }

    // Ensure the vault is registered
    let vault_registered = run_pwsh(
        "if (Get-SecretVault -Name 'SecretStore' -ErrorAction SilentlyContinue) { 'true' } else { 'false' }"
    ).map(|r| r.trim().eq_ignore_ascii_case("true"))
    .unwrap_or(false);

    if !vault_registered {
        run_pwsh(
            "Register-SecretVault -Name 'SecretStore' -ModuleName 'Microsoft.PowerShell.SecretStore' -DefaultVault"
        ).map_err(|e| t!("vault_config.registerVaultFailed", error = e).to_string())?;
    }

    // Return current state after applying changes
    get_config(input)
}

/// Test whether the current vault configuration matches the desired state.
pub fn test_config(input: &VaultConfig) -> Result<bool, String> {
    let current = get_config(input)?;

    if let Some(ref desired_auth) = input.authentication {
        if current.authentication.as_ref() != Some(desired_auth) {
            return Ok(false);
        }
    }

    if let Some(desired_timeout) = input.password_timeout {
        if current.password_timeout != Some(desired_timeout) {
            return Ok(false);
        }
    }

    if let Some(ref desired_interaction) = input.interaction {
        if current.interaction.as_ref() != Some(desired_interaction) {
            return Ok(false);
        }
    }

    // If we get here, all specified properties match
    Ok(true)
}
