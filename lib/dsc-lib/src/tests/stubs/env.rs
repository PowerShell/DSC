// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::ffi::{OsStr, OsString};
use std::env::VarError;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Reads the value of an environment variable according to the stubbed context.
///
/// This function simulates [`std::env::var`] relying on the stubbed environment
/// context provided by [`StubbedEnvContext`].
///
/// # Arguments
///
/// - `key` - The name of the environment variable to read.
///
/// # Returns
///
/// A [`Result<String, VarError>`] containing the value of the environment
/// variable if present, or an error otherwise.
pub(crate) fn var<K: AsRef<OsStr>>(key: K) -> Result<String, VarError> {
    StubbedEnvContext::find(&key.as_ref().to_string_lossy().into_owned())
}

/// Reads the value of an environment variable as an `OsString` according to
/// the stubbed context.
///
/// # Arguments
///
/// - `key` - The name of the environment variable to read.
///
/// # Returns
///
/// An [`Option<OsString>`] containing the value of the environment variable if
/// present, or `None` otherwise.
#[allow(dead_code)]
pub(crate) fn var_os<K: AsRef<OsStr>>(key: K) -> Option<OsString> {
    StubbedEnvContext::find(&key.as_ref().to_string_lossy().into_owned())
        .map(|res| OsString::from(res))
        .ok()
}

/// Macro to set up a stubbed environment context map for testing.
///
/// Use this macro to define environment variable stubs when you need to read
/// them for testing purposes. Each environment variable must be either:
///
/// - `Ok("value")` - The environment variable is defined with the specified
///   value.
/// - `Err(VarError::NotPresent)` - The environment variable isn't defined.
///
/// # Example
///
/// ```rust, ignore
/// stubbed_env_context_map! {
///     "DSC_TRACE_LEVEL" => Err(VarError::NotPresent),
///     "DSC_TRACE_FORMAT" => Ok("json"),
/// };
/// ```
macro_rules! stubbed_env_context_map {
    ($($key:expr => $result:expr),* $(,)?) => {
        $crate::tests::stubs::env::StubbedEnvContext::reset();
        $(
            $crate::tests::stubs::env::StubbedEnvContext::add($key, $result);
        )*
    };
}

/// Static storage for the global stubbed environment context.
///
/// This is used internally to manage the singleton instance of the stubbed
/// environment context.
static STUB_CONTEXT: OnceLock<Mutex<StubbedEnvContext>> = OnceLock::new();

/// Represents the stubbed environment context used for testing.
///
/// The stubbed environment context stores a mapping of environment variable
/// names to the result for reading them from the environment.
#[derive(Debug)]
pub struct StubbedEnvContext(HashMap<String, Result<String, VarError>>);

/// Static management API
impl StubbedEnvContext {
    /// Retrieves the global stubbed environment context, initializing it if necessary.
    ///
    /// # Returns
    ///
    /// A reference to the global [`Mutex`]-protected [`StubbedEnvContext`].
    pub(crate) fn get() -> &'static std::sync::Mutex<StubbedEnvContext> {
        STUB_CONTEXT
            .get_or_init(|| Mutex::new(StubbedEnvContext::new()))
    }
    /// Resets the global stubbed environment context by clearing all entries.
    pub(crate) fn reset() {
        let mut context = Self::get().lock().unwrap();
        context.0.clear();
    }
    /// Adds a new entry to the global stubbed environment context.
    ///
    /// # Arguments
    ///
    /// - `variable` - The name of the environment variable.
    /// - `context` - The result to associate with the environment variable.
    pub(crate) fn add<K: AsRef<OsStr>>(variable: K, context: Result<String, VarError>) {
        let mut context_map = Self::get().lock().unwrap();
        context_map.0.insert(variable.as_ref().to_string_lossy().into_owned(), context);
    }
    /// Finds the result associated with the given environment variable in the
    /// global stubbed environment context.
    ///
    /// # Returns
    ///
    /// The string value for the environment variable or [`VarError::NotPresent`].
    pub(crate) fn find<K: AsRef<OsStr>>(variable: K) -> Result<String, VarError> {
        let context_map = Self::get().lock().unwrap();
        context_map.0
            .get(&variable.as_ref().to_string_lossy().into_owned())
            .cloned()
            .unwrap_or(Err(VarError::NotPresent))
    }
}

// Internal management API
impl StubbedEnvContext {
    /// Creates a new instance of the stubbed environment context.
    ///
    /// # Returns
    ///
    /// A new [`StubbedEnvContext`] with an empty internal map.
    pub(crate) fn new() -> Self {
        StubbedEnvContext(HashMap::new())
    }
    /// Converts the stubbed environment context into a JSON representation.
    ///
    /// # Returns
    ///
    /// A [`serde_json::Value`] object representing the stubbed environment
    /// context.
    pub(crate) fn as_json(&self) -> serde_json::Value {
        serde_json::json!(self.0.iter().map(|(k, v)| {
            let value = match v {
                Ok(val) => serde_json::json!(val),
                Err(e) => serde_json::json!({"error": e.to_string()}),
            };
            (k.clone(), value)
        }).collect::<serde_json::Map<String, serde_json::Value>>())
    }
}
