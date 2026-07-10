// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use clap::ValueEnum;
use rust_i18n::t;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, RwLock},
};
use tracing::{debug, warn};

use crate::dscresources::command_resource::TraceLevel;
use crate::util::get_exe_path;

/// Name of the settings file used for the install, user, workspace, and policy scopes.
pub const SETTINGS_FILE_NAME: &str = "dsc.settings.json";
/// Name of the built-in defaults file shipped next to the `dsc` executable.
pub const DEFAULT_SETTINGS_FILE_NAME: &str = "dsc_default.settings.json";
/// Root key of the built-in defaults file identifying the settings schema version.
const DEFAULT_SETTINGS_SCHEMA_VERSION: &str = "1";

static RESOLVED_SETTINGS: LazyLock<RwLock<Option<Arc<ResolvedSettings>>>> = LazyLock::new(|| RwLock::new(None));

/// The format to use for trace output.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ValueEnum)]
pub enum TraceFormat {
    /// Human-readable output with ANSI colors.
    #[default]
    Default,
    /// Human-readable output without ANSI colors.
    Plaintext,
    /// Newline-delimited JSON objects.
    Json,
    /// Pass through trace messages from resources unmodified.
    #[clap(hide = true)]
    PassThrough,
}

/// Setting controlling trace output for DSC.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TracingSetting {
    /// Trace level to use.
    pub level: TraceLevel,
    /// Trace format to use.
    pub format: TraceFormat,
    /// Whether `level` can be overridden by the `DSC_TRACE_LEVEL` environment variable.
    pub allow_override: bool,
}

impl Default for TracingSetting {
    fn default() -> Self {
        Self {
            level: TraceLevel::Warn,
            format: TraceFormat::Default,
            allow_override: true,
        }
    }
}

/// Setting controlling where DSC discovers resources.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePathSetting {
    /// Whether to allow overriding with the `DSC_RESOURCE_PATH` environment variable.
    pub allow_env_override: bool,
    /// Whether to append the `PATH` environment variable to the list of resource directories.
    pub append_env_path: bool,
    /// Directories that DSC should search for non-built-in resources.
    pub directories: Vec<String>,
}

impl Default for ResourcePathSetting {
    fn default() -> Self {
        Self {
            allow_env_override: true,
            append_env_path: true,
            directories: vec![],
        }
    }
}

/// The settings document a user or administrator can define in a settings file.
///
/// Every field is optional; undefined fields fall back to lower-precedence scopes
/// or the code default.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DscSettings {
    /// Controls trace output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracing: Option<TracingSetting>,
    /// Controls resource discovery paths.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_path: Option<ResourcePathSetting>,
}

/// The scope a resolved setting field was defined in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SettingsScope {
    /// The code default; the field isn't defined in any settings file.
    Default,
    /// The built-in defaults file next to the executable.
    BuiltIn,
    /// The install settings file next to the executable.
    Install,
    /// The user settings file.
    User,
    /// The workspace settings file in the current directory.
    Workspace,
    /// The machine policy file. Fields defined as policy cannot be overridden.
    Policy,
}

/// A resolved setting field value with the scope it was defined in.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedField<T> {
    /// The resolved value for the field.
    pub value: T,
    /// The scope the value was defined in.
    pub scope: SettingsScope,
}

impl<T> ResolvedField<T> {
    /// Returns true if the field is enforced by policy and must not be overridden
    /// by environment variables or CLI options.
    #[must_use]
    pub fn is_policy(&self) -> bool {
        self.scope == SettingsScope::Policy
    }
}

/// The fully-resolved settings for the current invocation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedSettings {
    /// The resolved tracing setting.
    pub tracing: ResolvedField<TracingSetting>,
    /// The resolved resource path setting.
    pub resource_path: ResolvedField<ResourcePathSetting>,
}

impl Default for ResolvedSettings {
    fn default() -> Self {
        Self {
            tracing: ResolvedField {
                value: TracingSetting::default(),
                scope: SettingsScope::Default,
            },
            resource_path: ResolvedField {
                value: ResourcePathSetting::default(),
                scope: SettingsScope::Default,
            },
        }
    }
}

impl ResolvedSettings {
    /// Resolve settings from the given sources.
    ///
    /// Sources must be ordered from lowest to highest precedence. Each defined
    /// top-level field in a higher-precedence source replaces the value from
    /// lower-precedence sources.
    #[must_use]
    pub fn resolve(sources: &[(SettingsScope, DscSettings)]) -> Self {
        let mut resolved = Self::default();
        for (scope, settings) in sources {
            if let Some(tracing) = &settings.tracing {
                resolved.tracing = ResolvedField {
                    value: tracing.clone(),
                    scope: *scope,
                };
            }
            if let Some(resource_path) = &settings.resource_path {
                resolved.resource_path = ResolvedField {
                    value: resource_path.clone(),
                    scope: *scope,
                };
            }
        }
        resolved
    }
}

/// Get the resolved settings for the current invocation.
///
/// The settings files are read and resolved once; subsequent calls return the
/// cached result.
///
/// # Panics
///
/// Panics if the settings cache lock is poisoned.
#[must_use]
pub fn get_settings() -> Arc<ResolvedSettings> {
    if let Some(settings) = RESOLVED_SETTINGS.read().unwrap().as_ref() {
        return Arc::clone(settings);
    }

    let resolved = Arc::new(ResolvedSettings::resolve(&gather_sources()));
    *RESOLVED_SETTINGS.write().unwrap() = Some(Arc::clone(&resolved));
    resolved
}

/// Gather the settings documents from all file scopes, ordered from lowest to
/// highest precedence.
fn gather_sources() -> Vec<(SettingsScope, DscSettings)> {
    let mut sources = Vec::new();

    if let Ok(exe_path) = get_exe_path() {
        if let Some(exe_home) = exe_path.parent() {
            if let Some(settings) = load_default_settings_file(&exe_home.join(DEFAULT_SETTINGS_FILE_NAME)) {
                sources.push((SettingsScope::BuiltIn, settings));
            }
            if let Some(settings) = load_settings_file(&exe_home.join(SETTINGS_FILE_NAME)) {
                sources.push((SettingsScope::Install, settings));
            }
        }
    } else {
        debug!("{}", t!("settings.failedToGetExePath"));
    }

    if let Some(path) = user_settings_path()
        && let Some(settings) = load_settings_file(&path) {
        sources.push((SettingsScope::User, settings));
    }

    if let Some(settings) = load_settings_file(&workspace_settings_path()) {
        sources.push((SettingsScope::Workspace, settings));
    }

    if let Some(path) = policy_settings_path()
        && let Some(settings) = load_settings_file(&path) {
        sources.push((SettingsScope::Policy, settings));
    }

    sources
}

/// Load a settings document from a flat settings file.
///
/// Returns `None` if the file doesn't exist or can't be parsed. Parse failures
/// are logged as warnings so a corrupt settings file doesn't break DSC.
fn load_settings_file(path: &Path) -> Option<DscSettings> {
    let Ok(file) = File::open(path) else {
        debug!("{}", t!("settings.notFoundSettingsFile", path = path.to_string_lossy()));
        return None;
    };
    match serde_json::from_reader::<_, DscSettings>(BufReader::new(file)) {
        Ok(settings) => {
            debug!("{}", t!("settings.loadedSettingsFile", path = path.to_string_lossy()));
            Some(settings)
        },
        Err(err) => {
            warn!("{}", t!("settings.invalidSettingsFile", path = path.to_string_lossy(), error = err));
            None
        }
    }
}

/// Load the built-in defaults file, which nests the settings document under a
/// root key identifying the settings schema version.
fn load_default_settings_file(path: &Path) -> Option<DscSettings> {
    let Ok(file) = File::open(path) else {
        debug!("{}", t!("settings.notFoundSettingsFile", path = path.to_string_lossy()));
        return None;
    };
    let root: serde_json::Value = match serde_json::from_reader(BufReader::new(file)) {
        Ok(root) => root,
        Err(err) => {
            warn!("{}", t!("settings.invalidSettingsFile", path = path.to_string_lossy(), error = err));
            return None;
        }
    };
    let Some(document) = root.get(DEFAULT_SETTINGS_SCHEMA_VERSION) else {
        warn!("{}", t!("settings.missingSettingsSchemaVersion", path = path.to_string_lossy(), version = DEFAULT_SETTINGS_SCHEMA_VERSION));
        return None;
    };
    match serde_json::from_value::<DscSettings>(document.clone()) {
        Ok(settings) => {
            debug!("{}", t!("settings.loadedSettingsFile", path = path.to_string_lossy()));
            Some(settings)
        },
        Err(err) => {
            warn!("{}", t!("settings.invalidSettingsFile", path = path.to_string_lossy(), error = err));
            None
        }
    }
}

/// Get the path to the machine policy settings file.
///
/// The policy file location is writable only by administrators but readable by
/// all users:
///
/// - **Windows**: `%PROGRAMDATA%\dsc\dsc.settings.json`
/// - **macOS**: `/Library/dsc/dsc.settings.json`
/// - **Linux**: `/etc/dsc/dsc.settings.json`
#[must_use]
pub fn policy_settings_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let program_data = env::var_os("ProgramData")?;
        return Some(Path::new(&program_data).join("dsc").join(SETTINGS_FILE_NAME));
    }
    #[cfg(target_os = "macos")]
    {
        Some(Path::new("/Library").join("dsc").join(SETTINGS_FILE_NAME))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Some(Path::new("/etc").join("dsc").join(SETTINGS_FILE_NAME))
    }
}

/// Get the path to the user settings file.
///
/// If `XDG_CONFIG_HOME` is defined it's respected on every platform, otherwise
/// the platform convention is used:
///
/// - **Windows**: `%APPDATA%\dsc\dsc.settings.json`
/// - **macOS**: `$HOME/Library/Application Support/dsc/dsc.settings.json`
/// - **Linux**: `$HOME/.config/dsc/dsc.settings.json`
#[must_use]
pub fn user_settings_path() -> Option<PathBuf> {
    if let Some(xdg_config_home) = env::var_os("XDG_CONFIG_HOME") {
        return Some(Path::new(&xdg_config_home).join("dsc").join(SETTINGS_FILE_NAME));
    }
    #[cfg(target_os = "windows")]
    {
        let app_data = env::var_os("APPDATA")?;
        return Some(Path::new(&app_data).join("dsc").join(SETTINGS_FILE_NAME));
    }
    #[cfg(target_os = "macos")]
    {
        let home = env::var_os("HOME")?;
        return Some(Path::new(&home).join("Library").join("Application Support").join("dsc").join(SETTINGS_FILE_NAME));
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let home = env::var_os("HOME")?;
        return Some(Path::new(&home).join(".config").join("dsc").join(SETTINGS_FILE_NAME));
    }
    #[allow(unreachable_code)]
    None
}

/// Get the path to the workspace settings file in the current directory.
#[must_use]
pub fn workspace_settings_path() -> PathBuf {
    Path::new(".").join(SETTINGS_FILE_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct TempFile(PathBuf);

    impl TempFile {
        fn new(name: &str, content: &str) -> Self {
            let path = env::temp_dir().join(format!("dsc-settings-test-{}-{name}", std::process::id()));
            fs::write(&path, content).unwrap();
            Self(path)
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    #[test]
    fn resolve_with_no_sources_uses_defaults() {
        let resolved = ResolvedSettings::resolve(&[]);
        assert_eq!(resolved.tracing.scope, SettingsScope::Default);
        assert_eq!(resolved.tracing.value, TracingSetting::default());
        assert_eq!(resolved.resource_path.scope, SettingsScope::Default);
        assert_eq!(resolved.resource_path.value, ResourcePathSetting::default());
    }

    #[test]
    fn higher_scope_setting_overrides_lower_scope() {
        let install = DscSettings {
            tracing: Some(TracingSetting {
                level: TraceLevel::Info,
                format: TraceFormat::Default,
                allow_override: true,
            }),
            resource_path: None,
        };
        let workspace = DscSettings {
            tracing: Some(TracingSetting {
                level: TraceLevel::Debug,
                format: TraceFormat::Json,
                allow_override: false,
            }),
            resource_path: None,
        };
        let resolved = ResolvedSettings::resolve(&[
            (SettingsScope::Install, install),
            (SettingsScope::Workspace, workspace),
        ]);
        assert_eq!(resolved.tracing.scope, SettingsScope::Workspace);
        assert_eq!(resolved.tracing.value.level, TraceLevel::Debug);
        assert_eq!(resolved.tracing.value.format, TraceFormat::Json);
        assert!(!resolved.tracing.value.allow_override);
        // undefined field falls back to the default
        assert_eq!(resolved.resource_path.scope, SettingsScope::Default);
    }

    #[test]
    fn policy_overrides_all_settings() {
        let workspace = DscSettings {
            tracing: Some(TracingSetting {
                level: TraceLevel::Debug,
                format: TraceFormat::Json,
                allow_override: true,
            }),
            resource_path: None,
        };
        let policy = DscSettings {
            tracing: Some(TracingSetting {
                level: TraceLevel::Error,
                format: TraceFormat::Plaintext,
                allow_override: false,
            }),
            resource_path: None,
        };
        let resolved = ResolvedSettings::resolve(&[
            (SettingsScope::Workspace, workspace),
            (SettingsScope::Policy, policy),
        ]);
        assert!(resolved.tracing.is_policy());
        assert_eq!(resolved.tracing.value.level, TraceLevel::Error);
        assert_eq!(resolved.tracing.value.format, TraceFormat::Plaintext);
    }

    #[test]
    fn fields_resolve_independently() {
        let user = DscSettings {
            tracing: Some(TracingSetting::default()),
            resource_path: None,
        };
        let workspace = DscSettings {
            tracing: None,
            resource_path: Some(ResourcePathSetting {
                allow_env_override: false,
                append_env_path: false,
                directories: vec!["/custom".to_string()],
            }),
        };
        let resolved = ResolvedSettings::resolve(&[
            (SettingsScope::User, user),
            (SettingsScope::Workspace, workspace),
        ]);
        assert_eq!(resolved.tracing.scope, SettingsScope::User);
        assert_eq!(resolved.resource_path.scope, SettingsScope::Workspace);
        assert_eq!(resolved.resource_path.value.directories, vec!["/custom".to_string()]);
    }

    #[test]
    fn settings_document_deserializes_camel_case() {
        let json = r#"{
            "resourcePath": {
                "allowEnvOverride": true,
                "appendEnvPath": true,
                "directories": []
            },
            "tracing": {
                "level": "WARN",
                "format": "Default",
                "allowOverride": true
            }
        }"#;
        let settings: DscSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.tracing, Some(TracingSetting::default()));
        assert_eq!(settings.resource_path, Some(ResourcePathSetting::default()));
    }

    #[test]
    fn settings_document_roundtrips() {
        let settings = DscSettings {
            tracing: Some(TracingSetting {
                level: TraceLevel::Debug,
                format: TraceFormat::Json,
                allow_override: false,
            }),
            resource_path: Some(ResourcePathSetting {
                allow_env_override: false,
                append_env_path: false,
                directories: vec!["/one".to_string(), "/two".to_string()],
            }),
        };
        let json = serde_json::to_value(&settings).unwrap();
        assert_eq!(json["tracing"]["level"], "DEBUG");
        assert_eq!(json["tracing"]["format"], "Json");
        assert_eq!(json["tracing"]["allowOverride"], false);
        assert_eq!(json["resourcePath"]["allowEnvOverride"], false);
        assert_eq!(json["resourcePath"]["appendEnvPath"], false);
        assert_eq!(json["resourcePath"]["directories"][0], "/one");
        let roundtripped: DscSettings = serde_json::from_value(json).unwrap();
        assert_eq!(roundtripped, settings);
    }

    #[test]
    fn undefined_fields_deserialize_as_none() {
        let settings: DscSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings, DscSettings::default());
        // unknown fields are ignored so future settings don't break older versions
        let settings: DscSettings = serde_json::from_str(r#"{"futureSetting": true}"#).unwrap();
        assert_eq!(settings, DscSettings::default());
    }

    #[test]
    fn load_settings_file_reads_flat_document() {
        let file = TempFile::new(
            "flat.json",
            r#"{"tracing": {"level": "INFO", "format": "Plaintext", "allowOverride": false}}"#,
        );
        let settings = load_settings_file(&file.0).unwrap();
        let tracing = settings.tracing.unwrap();
        assert_eq!(tracing.level, TraceLevel::Info);
        assert_eq!(tracing.format, TraceFormat::Plaintext);
        assert!(!tracing.allow_override);
        assert_eq!(settings.resource_path, None);
    }

    #[test]
    fn load_settings_file_returns_none_for_missing_or_invalid() {
        assert!(load_settings_file(Path::new("nonexistent-dsc-settings.json")).is_none());
        let file = TempFile::new("invalid.json", "not json");
        assert!(load_settings_file(&file.0).is_none());
        // valid JSON but wrong shape is also rejected instead of panicking
        let file = TempFile::new("wrong-shape.json", r#"{"tracing": "yes"}"#);
        assert!(load_settings_file(&file.0).is_none());
    }

    #[test]
    fn load_default_settings_file_requires_version_root() {
        let file = TempFile::new(
            "versioned.json",
            r#"{"1": {"resourcePath": {"allowEnvOverride": false, "appendEnvPath": false, "directories": ["/default"]}}}"#,
        );
        let settings = load_default_settings_file(&file.0).unwrap();
        let resource_path = settings.resource_path.unwrap();
        assert_eq!(resource_path.directories, vec!["/default".to_string()]);

        // a document without the expected version root is rejected
        let file = TempFile::new("unversioned.json", r#"{"resourcePath": {}}"#);
        assert!(load_default_settings_file(&file.0).is_none());
    }

    #[test]
    fn get_settings_returns_cached_instance() {
        let first = get_settings();
        let second = get_settings();
        assert!(Arc::ptr_eq(&first, &second));
    }
}
