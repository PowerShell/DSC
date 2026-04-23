// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::types::Secret;
use rust_i18n::t;
use std::process::Command;

/// Run a PowerShell command and return its stdout, or an error with stderr.
fn run_pwsh(script: &str) -> Result<String, String> {
    let output = Command::new("pwsh")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

/// Get a secret's information from the SecretStore.
pub fn get_secret(input: &Secret) -> Result<Secret, String> {
    let name = input.name.as_deref()
        .ok_or_else(|| t!("secret.nameRequired").to_string())?;

    let vault_arg = input.vault_name.as_deref()
        .map(|v| format!(" -Vault '{v}'"))
        .unwrap_or_default();

    let script = format!(
        r#"
        $info = Get-SecretInfo -Name '{name}'{vault_arg} -ErrorAction SilentlyContinue
        if ($null -eq $info) {{
            @{{ name = '{name}'; _exist = $false }} | ConvertTo-Json -Compress
        }} else {{
            $meta = $null
            if ($info.Metadata.Count -gt 0) {{
                $meta = @{{}}
                foreach ($key in $info.Metadata.Keys) {{
                    $meta[$key] = $info.Metadata[$key].ToString()
                }}
            }}
            @{{
                name = $info.Name
                secretType = $info.Type.ToString()
                vaultName = $info.VaultName
                metadata = $meta
                _exist = $true
            }} | ConvertTo-Json -Compress
        }}
        "#
    );

    let result = run_pwsh(&script)
        .map_err(|e| t!("secret.getFailed", name = name, error = e).to_string())?;

    let secret: Secret = serde_json::from_str(&result)
        .map_err(|e| t!("secret.getFailed", name = name, error = e.to_string()).to_string())?;

    Ok(secret)
}

/// Set (create or update) a secret in the SecretStore.
pub fn set_secret(input: &Secret) -> Result<Secret, String> {
    let name = input.name.as_deref()
        .ok_or_else(|| t!("secret.nameRequired").to_string())?;

    let exist = input._exist.unwrap_or(true);

    if !exist {
        // Remove the secret
        let vault_arg = input.vault_name.as_deref()
            .map(|v| format!(" -Vault '{v}'"))
            .unwrap_or_default();

        run_pwsh(&format!(
            "Remove-Secret -Name '{name}'{vault_arg} -ErrorAction SilentlyContinue"
        )).map_err(|e| t!("secret.removeFailed", name = name, error = e).to_string())?;

        return Ok(Secret {
            name: Some(name.to_string()),
            _exist: Some(false),
            ..Default::default()
        });
    }

    let value = input.value.as_deref().unwrap_or("");
    let vault_arg = input.vault_name.as_deref()
        .map(|v| format!(" -Vault '{v}'"))
        .unwrap_or_default();

    // Set the secret value
    let script = format!(
        "Set-Secret -Name '{name}' -Secret '{value}'{vault_arg}"
    );
    run_pwsh(&script)
        .map_err(|e| t!("secret.setFailed", name = name, error = e).to_string())?;

    // Set metadata if provided
    if let Some(ref metadata) = input.metadata {
        if let Some(obj) = metadata.as_object() {
            let mut hashtable_entries = Vec::new();
            for (key, val) in obj {
                let val_str = match val {
                    serde_json::Value::String(s) => format!("'{s}'"),
                    other => other.to_string(),
                };
                hashtable_entries.push(format!("'{key}' = {val_str}"));
            }
            if !hashtable_entries.is_empty() {
                let ht = hashtable_entries.join("; ");
                let meta_script = format!(
                    "Set-SecretInfo -Name '{name}'{vault_arg} -Metadata @{{ {ht} }}"
                );
                run_pwsh(&meta_script)
                    .map_err(|e| t!("secret.setFailed", name = name, error = e).to_string())?;
            }
        }
    }

    // Return current state
    get_secret(input)
}

/// Export all secrets (metadata only, not values) from the SecretStore.
pub fn export_secrets(filter: Option<&Secret>) -> Result<Vec<Secret>, String> {
    let name_filter = filter
        .and_then(|f| f.name.as_deref())
        .unwrap_or("*");

    let vault_arg = filter
        .and_then(|f| f.vault_name.as_deref())
        .map(|v| format!(" -Vault '{v}'"))
        .unwrap_or_default();

    let script = format!(
        r#"
        $secrets = Get-SecretInfo -Name '{name_filter}'{vault_arg} -ErrorAction SilentlyContinue
        if ($null -eq $secrets) {{
            '[]'
        }} else {{
            $results = @()
            foreach ($info in $secrets) {{
                $meta = $null
                if ($info.Metadata.Count -gt 0) {{
                    $meta = @{{}}
                    foreach ($key in $info.Metadata.Keys) {{
                        $meta[$key] = $info.Metadata[$key].ToString()
                    }}
                }}
                $results += @{{
                    name = $info.Name
                    secretType = $info.Type.ToString()
                    vaultName = $info.VaultName
                    metadata = $meta
                    _exist = $true
                }}
            }}
            $results | ConvertTo-Json -Compress -AsArray
        }}
        "#
    );

    let result = run_pwsh(&script)
        .map_err(|e| t!("secret.exportFailed", error = e).to_string())?;

    let secrets: Vec<Secret> = serde_json::from_str(&result)
        .map_err(|e| t!("secret.exportFailed", error = e.to_string()).to_string())?;

    Ok(secrets)
}
