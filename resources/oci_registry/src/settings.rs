// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib_security_context::{SecurityContext, get_security_context};
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub const SETTINGS_FILE: &str = "Microsoft.Registry.OpenContainerInitaitive.settings.json";
pub const POLICY_FILE: &str = "Microsoft.Registry.OpenContainerInitaitive.policy.json";

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    #[default]
    CurrentUser,
    AllUsers,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Authentication {
    pub secret_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_vault: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<Authentication>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub registries: Vec<BTreeMap<String, Registry>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RegistryList {
    #[serde(rename = "_name", skip_serializing_if = "Option::is_none")]
    #[schemars(extend("readOnly" = true))]
    pub name: Option<String>,
    #[serde(default)]
    pub scope: Scope,
    #[serde(default)]
    pub policy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registries: Option<Vec<BTreeMap<String, Registry>>>,
}

impl Settings {
    pub fn entries(&self) -> Result<BTreeMap<String, Registry>, String> {
        let mut entries = BTreeMap::new();
        for item in &self.registries {
            if item.len() != 1 {
                return Err(t!("settings.invalid").to_string());
            }
            for (name, registry) in item {
                if name.eq_ignore_ascii_case("local") {
                    return Err(t!("settings.reservedName", name = name).to_string());
                }
                if name.trim().is_empty()
                    || entries.contains_key(name)
                    || registry.authentication.as_ref().is_some_and(|auth| {
                        auth.secret_name.trim().is_empty()
                            || auth
                                .secret_vault
                                .as_ref()
                                .is_some_and(|vault| vault.trim().is_empty())
                    })
                {
                    return Err(t!("settings.invalid").to_string());
                }
                crate::oci::registry_url(&registry.location)?;
                entries.insert(name.clone(), registry.clone());
            }
        }
        Ok(entries)
    }
}

pub fn settings_path(scope: Scope, policy: bool) -> Result<PathBuf, String> {
    if policy && scope != Scope::AllUsers {
        return Err(t!("settings.scope").to_string());
    }
    let root = match scope {
        Scope::CurrentUser => {
            #[cfg(windows)]
            {
                environment_path("LOCALAPPDATA")?.join("dsc")
            }
            #[cfg(not(windows))]
            {
                environment_path("HOME")?.join(".dsc")
            }
        }
        Scope::AllUsers => {
            #[cfg(windows)]
            {
                environment_path("ProgramData")?.join("dsc")
            }
            #[cfg(target_os = "macos")]
            {
                PathBuf::from("/Library/Application Support/dsc")
            }
            #[cfg(not(any(windows, target_os = "macos")))]
            {
                PathBuf::from("/etc/dsc")
            }
        }
    };
    Ok(root.join(if policy { POLICY_FILE } else { SETTINGS_FILE }))
}

fn environment_path(name: &str) -> Result<PathBuf, String> {
    std::env::var_os(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| t!("settings.missingPath", name = name).to_string())
}

pub fn read(path: &Path) -> Result<Settings, String> {
    let content = match fs::read(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Settings::default());
        }
        Err(error) => {
            return Err(t!("settings.path", path = path.display(), error = error).to_string());
        }
    };
    let settings: Settings = serde_json::from_slice(&content).map_err(|error| {
        t!("settings.invalidFile", path = path.display(), error = error).to_string()
    })?;
    settings.entries().map_err(|error| {
        t!("settings.invalidFile", path = path.display(), error = error).to_string()
    })?;
    Ok(settings)
}

pub fn merge(paths: &[PathBuf]) -> Result<BTreeMap<String, Registry>, String> {
    let mut registries = BTreeMap::new();
    for path in paths {
        registries.extend(read(path)?.entries()?);
    }
    Ok(registries)
}

pub fn effective() -> Result<BTreeMap<String, Registry>, String> {
    merge(&[
        settings_path(Scope::AllUsers, false)?,
        settings_path(Scope::CurrentUser, false)?,
        settings_path(Scope::AllUsers, true)?,
    ])
}

pub fn get(input: &RegistryList) -> Result<RegistryList, String> {
    let settings = read(&settings_path(input.scope, input.policy)?)?;
    Ok(RegistryList {
        name: None,
        registries: Some(settings.registries),
        ..input.clone()
    })
}

pub fn set(input: &RegistryList) -> Result<RegistryList, String> {
    let path = settings_path(input.scope, input.policy)?;
    let settings = Settings {
        registries: input
            .registries
            .clone()
            .ok_or_else(|| t!("settings.required").to_string())?,
    };
    settings.entries()?;
    if input.scope == Scope::AllUsers && get_security_context() != SecurityContext::Admin {
        return Err(t!("settings.elevated").to_string());
    }
    write(&path, &settings, input.scope)?;
    get(input)
}

fn write(path: &Path, settings: &Settings, scope: Scope) -> Result<(), String> {
    #[cfg(not(unix))]
    let _ = scope;
    let write_file = || -> Result<(), Box<dyn std::error::Error>> {
        let parent = path
            .parent()
            .ok_or_else(|| std::io::Error::other(t!("settings.invalid").to_string()))?;
        fs::create_dir_all(parent)?;
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        serde_json::to_writer_pretty(&mut file, settings)?;
        file.write_all(b"\n")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            file.as_file().set_permissions(fs::Permissions::from_mode(
                if scope == Scope::AllUsers {
                    0o644
                } else {
                    0o600
                },
            ))?;
        }
        file.as_file().sync_all()?;
        file.persist(path)?;
        Ok(())
    };
    write_file()
        .map_err(|error| t!("settings.path", path = path.display(), error = error).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn settings(value: serde_json::Value) -> Settings {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn policy_wins_and_distinct_registries_survive() {
        let dir = tempfile::tempdir().unwrap();
        let paths: Vec<_> = ["machine", "user", "policy"]
            .iter()
            .map(|name| dir.path().join(name))
            .collect();
        for (path, value) in paths.iter().zip([
            json!({"registries":[{"shared":{"location":"machine.example"}},{"machine":{"location":"machine.example"}}]}),
            json!({"registries":[{"shared":{"location":"user.example"}},{"user":{"location":"user.example"}}]}),
            json!({"registries":[{"shared":{"location":"policy.example"}},{"policy":{"location":"policy.example"}}]}),
        ]) {
            write(path, &settings(value), Scope::CurrentUser).unwrap();
        }
        let result = merge(&paths).unwrap();
        assert_eq!(result.len(), 4);
        assert_eq!(result["shared"].location, "policy.example");
        fs::remove_file(&paths[2]).unwrap();
        assert_eq!(merge(&paths).unwrap()["shared"].location, "user.example");
    }

    #[test]
    fn validates_registry_shape_and_references() {
        for value in [
            json!({"registries":[{}]}),
            json!({"registries":[{"local":{"location":"example.com"}}]}),
            json!({"registries":[{"a":{"location":"example.com"},"b":{"location":"example.com"}}]}),
            json!({"registries":[{"a":{"location":"example.com"}},{"a":{"location":"other.example"}}]}),
            json!({"registries":[{"a":{"location":"example.com","authentication":{"secretName":""}}}]}),
            json!({"registries":[{"a":{"location":"https://user:password@example.com"}}]}),
        ] {
            assert!(settings(value).entries().is_err());
        }
        assert!(
            serde_json::from_value::<Settings>(json!({"registries":[{"a":{
                "location":"example.com","authentication":{}
            }}]}))
            .is_err()
        );
    }

    #[test]
    #[cfg(unix)]
    fn machine_settings_remain_readable_but_not_writable_by_other_users() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SETTINGS_FILE);
        for (scope, expected) in [(Scope::CurrentUser, 0o600), (Scope::AllUsers, 0o644)] {
            write(&path, &Settings::default(), scope).unwrap();
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                expected
            );
        }
    }

    #[test]
    fn file_replacement_empty_and_malformed_settings() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(SETTINGS_FILE);
        assert!(read(&path).unwrap().registries.is_empty());
        write(
            &path,
            &settings(json!({"registries":[{"a":{"location":"example.com"}}]})),
            Scope::CurrentUser,
        )
        .unwrap();
        write(&path, &Settings::default(), Scope::CurrentUser).unwrap();
        assert!(read(&path).unwrap().registries.is_empty());
        fs::write(&path, "{").unwrap();
        assert!(read(&path).is_err());
        assert!(settings_path(Scope::CurrentUser, true).is_err());
    }
}
