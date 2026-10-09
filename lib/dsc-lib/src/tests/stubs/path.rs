// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::ops::Deref;
use std::sync::{Mutex, OnceLock};

/// Context for a stubbed path, used to simulate file system behavior in tests.
#[derive(Debug)]
pub struct StubbedPathContext {
    /// Indicates whether the [`exists`] method would return `true` for this path.
    ///
    /// [`exists`]: Path::exists
    pub should_exist: bool,
    /// Indicates whether the [`is_file`] method would return `true` for this path.
    ///
    /// [`is_file`]: Path::is_file
    pub is_file: bool,
    /// Indicates whether the [`is_dir`] method would return `true` for this path.
    ///
    /// [`is_dir`]: Path::is_dir
    pub is_dir: bool,
    /// Indicates the result to return from [`fs::read_to_string`] for this path.
    ///
    /// [`fs::read_to_string`]: crate::tests::stubs::fs::read_to_string
    pub read_result: Result<String, std::io::Error>,
}

impl Clone for StubbedPathContext {
    fn clone(&self) -> Self {
        Self {
            should_exist: self.should_exist,
            is_file: self.is_file,
            is_dir: self.is_dir,
            read_result: match self.read_result {
                Ok(ref s) => Ok(s.clone()),
                Err(ref e) => Err(std::io::Error::new(e.kind(), e.to_string())),
            }
        }
    }
}

impl StubbedPathContext {
    /// Converts the stubbed path context into a JSON representation.
    ///
    /// # Returns
    ///
    /// A [`serde_json::Value`] object representing the stubbed path context.
    fn as_json(&self) -> serde_json::Value {
        let read_result = match &self.read_result {
            Ok(s) => serde_json::json!({"ok": s}),
            Err(e) => serde_json::json!({
                "err": {
                    "kind": e.kind().to_string(),
                    "message": e.to_string(),
                }
            }),
        };
        serde_json::json!({
            "should_exist": self.should_exist,
            "is_file": self.is_file,
            "is_dir": self.is_dir,
            "read_result": read_result,
        })
    }
}

impl Default for StubbedPathContext {
    fn default() -> Self {
        Self {
            should_exist: false,
            is_file: false,
            is_dir: false,
            read_result: Err(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Stub not initialized"
                )
            ),
        }
    }
}

/// A map of stubbed path contexts, keyed by path strings.
///
/// This structure provides methods for managing and querying stubbed path contexts,
/// which are used to simulate file system behavior in tests.
#[derive(Debug, Clone)]
pub struct StubbedPathContextMap(HashMap<String, StubbedPathContext>);

// Static management API
impl StubbedPathContextMap {
    /// Retrieves the global stubbed path context map, initializing it if necessary.
    ///
    /// # Returns
    ///
    /// A reference to a [`std::sync::Mutex`] guarding the global [`StubbedPathContextMap`].
    pub fn get() -> &'static std::sync::Mutex<StubbedPathContextMap> {
        STUB_CONTEXT
            .get_or_init(|| Mutex::new(StubbedPathContextMap::new()))
    }
    /// Resets the global stubbed path context map, clearing all entries.
    pub fn reset() {
        let mut context = Self::get().lock().unwrap();
        context.clear();
    }
    /// Adds a stubbed path context to the global map.
    ///
    /// # Arguments
    ///
    /// - `path` - The path string to associate with the stubbed context.
    /// - `context` - The stubbed path context to add.
    pub fn add(path: String, context: StubbedPathContext) {
        let mut context_map = Self::get().lock().unwrap();
        context_map.insert(path, context);
    }
    /// Finds the stubbed path context associated with the given path.
    ///
    /// # Arguments
    ///
    /// - `path` - The path to look up in the global stubbed context map.
    ///
    /// # Returns
    ///
    /// The stubbed path context associated with the path, or the default context if not found.
    pub fn find(path: &Path) -> StubbedPathContext {
        let context_map = Self::get().lock().unwrap();
        if let Some(context) = context_map.0.get(path.to_string_lossy().as_ref()) {
            return context.clone();
        }
        StubbedPathContext::default()
    }

    /// Converts the stubbed path context map into a JSON representation.
    ///
    /// # Returns
    ///
    /// A [`serde_json::Value`] object representing the stubbed path context map.
    pub fn as_json(&self) -> serde_json::Value {
        serde_json::json!(self.0.iter().map(|(k, v)| {
            (k.clone(), v.as_json())
        }).collect::<serde_json::Map<String, serde_json::Value>>())
    }
}

// Internal management API
impl StubbedPathContextMap {
    /// Creates a new, empty stubbed path context map.
    ///
    /// # Returns
    ///
    /// A new instance of [`StubbedPathContextMap`] with no entries.
    fn new() -> Self {
        Self(HashMap::new())
    }

    /// Inserts a stubbed path context into the map.
    ///
    /// # Arguments
    ///
    /// - `path` - The path string to associate with the stubbed context.
    /// - `context` - The stubbed path context to insert.
    fn insert(&mut self, path: String, context: StubbedPathContext) {
        self.0.insert(path, context);
    }

    /// Clears all entries from the stubbed path context map.
    fn clear(&mut self) {
        self.0.clear();
    }
}

/// Macro to initialize the stubbed path context map for testing purposes.
///
/// # Example
///
/// ```rust, ignore
/// #[cfg(test)] mod tests {
///
/// }
///
/// stubbed_path_context_map! {
///     "/some/path" => StubbedPathContext::default(),
///     "/another/path" => StubbedPathContext {
///         should_exist: true,
///         is_file: true,
///         is_dir: false,
///         read_result: Ok(String::from("Text in the file\n"))
///     }
/// };
/// ```
#[allow(unused_macros)]
macro_rules! stubbed_path_context_map {
    ($($path:expr => $context:expr),+ $(,)?) => {
        {
            $crate::tests::stubs::path::StubbedPathContextMap::reset();
            $(
                $crate::tests::stubs::path::StubbedPathContextMap::add($path.to_string(), $context);
            )+
        }
    };
}

static STUB_CONTEXT: OnceLock<Mutex<StubbedPathContextMap>> = OnceLock::new();

/// A transparent stand-in for [`std::path::Path`].
#[repr(transparent)]
pub struct Path {
    pub inner: std::path::Path,
}

// Public Stub API - these methods provide the minimal interface required for testing purposes.
impl Path {
    /// Indicates whether the path exists according to the stubbed context.
    ///
    /// # Returns
    ///
    /// `true` if the path exists according to the stubbed context, `false` otherwise.
    #[must_use]
    pub fn exists(&self) -> bool {
        StubbedPathContextMap::find(self).should_exist
    }

    /// Indicates whether the path is a file according to the stubbed context.
    ///
    /// # Returns
    ///
    /// `true` if the path is a file according to the stubbed context, `false` otherwise.
    #[must_use]
    pub fn is_file(&self) -> bool {
        StubbedPathContextMap::find(self).is_file
    }

    /// Indicates whether the path is a directory according to the stubbed context.
    ///
    /// # Returns
    ///
    /// `true` if the path is a directory according to the stubbed context, `false` otherwise.
    #[must_use]
    pub fn is_dir(&self) -> bool {
        StubbedPathContextMap::find(self).is_dir
    }
}

// Private Stub API - these methods are used internally and are not exposed publicly.
impl Path {
    fn from_std(value: &std::path::Path) -> &Self {
        // `Path` is transparent over `std::path::Path`, including its metadata.
        unsafe { &*(std::ptr::from_ref(value) as *const Self) }
    }
}

// Public Passthru API - these methods simply delegate to the inner `std::path::Path` instance.
impl Path {
    #[must_use]
    pub fn new<S: AsRef<OsStr> + ?Sized>(value: &S) -> &Self {
        Self::from_std(std::path::Path::new(value))
    }

    #[must_use]
    pub fn as_os_str(&self) -> &OsStr {
        self.inner.as_os_str()
    }

    #[must_use]
    pub fn to_str(&self) -> Option<&str> {
        self.inner.to_str()
    }

    #[must_use]
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        self.inner.to_string_lossy()
    }

    #[must_use]
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf {
            inner: self.inner.to_path_buf(),
        }
    }

    #[must_use]
    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        PathBuf {
            inner: self.inner.join(&path.as_ref().inner),
        }
    }

    #[must_use]
    pub fn parent(&self) -> Option<&Self> {
        self.inner.parent().map(Self::from_std)
    }

    #[must_use]
    pub fn file_name(&self) -> Option<&OsStr> {
        self.inner.file_name()
    }

    #[must_use]
    pub fn file_stem(&self) -> Option<&OsStr> {
        self.inner.file_stem()
    }

    #[must_use]
    pub fn extension(&self) -> Option<&OsStr> {
        self.inner.extension()
    }

    #[must_use]
    pub fn is_absolute(&self) -> bool {
        self.inner.is_absolute()
    }

    #[must_use]
    pub fn is_relative(&self) -> bool {
        self.inner.is_relative()
    }

    #[must_use]
    pub fn starts_with<P: AsRef<Path>>(&self, base: P) -> bool {
        self.inner.starts_with(&base.as_ref().inner)
    }

    #[must_use]
    pub fn ends_with<P: AsRef<Path>>(&self, child: P) -> bool {
        self.inner.ends_with(&child.as_ref().inner)
    }

    pub fn strip_prefix<P: AsRef<Path>>(
        &self,
        base: P,
    ) -> Result<&Self, std::path::StripPrefixError> {
        self.inner
            .strip_prefix(&base.as_ref().inner)
            .map(Self::from_std)
    }

    #[must_use]
    pub fn display(&self) -> std::path::Display<'_> {
        self.inner.display()
    }
}

impl AsRef<Path> for Path {
    fn as_ref(&self) -> &Path {
        &self
    }
}

impl AsRef<OsStr> for Path {
    fn as_ref(&self) -> &OsStr {
        self.as_os_str()
    }
}

impl AsRef<Path> for OsStr {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for OsString {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for str {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for String {
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl fmt::Debug for Path {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(formatter)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.display().fmt(formatter)
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for Path {}

/// An owned stand-in for [`std::path::PathBuf`].
#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PathBuf {
    pub inner: std::path::PathBuf,
}

impl PathBuf {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: std::path::PathBuf::new(),
        }
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: std::path::PathBuf::with_capacity(capacity),
        }
    }

    #[must_use]
    pub fn as_path(&self) -> &Path {
        Path::from_std(self.inner.as_path())
    }

    pub fn push<P: AsRef<Path>>(&mut self, path: P) {
        self.inner.push(&path.as_ref().inner);
    }

    pub fn pop(&mut self) -> bool {
        self.inner.pop()
    }

    pub fn set_file_name<S: AsRef<OsStr>>(&mut self, file_name: S) {
        self.inner.set_file_name(file_name);
    }

    pub fn set_extension<S: AsRef<OsStr>>(&mut self, extension: S) -> bool {
        self.inner.set_extension(extension)
    }

    #[must_use]
    pub fn into_os_string(self) -> OsString {
        self.inner.into_os_string()
    }

    #[must_use]
    pub fn into_std(self) -> std::path::PathBuf {
        self.inner
    }
}

impl Deref for PathBuf {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        self.as_path()
    }
}

impl<T: AsRef<OsStr> + ?Sized> From<&T> for PathBuf {
    fn from(value: &T) -> Self {
        Self {
            inner: std::path::PathBuf::from(value.as_ref()),
        }
    }
}

impl From<OsString> for PathBuf {
    fn from(value: OsString) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl From<std::path::PathBuf> for PathBuf {
    fn from(value: std::path::PathBuf) -> Self {
        Self { inner: value }
    }
}

impl From<PathBuf> for std::path::PathBuf {
    fn from(value: PathBuf) -> Self {
        value.inner
    }
}

impl AsRef<Path> for PathBuf {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl AsRef<std::path::Path> for PathBuf {
    fn as_ref(&self) -> &std::path::Path {
        self.inner.as_path()
    }
}

impl AsRef<OsStr> for PathBuf {
    fn as_ref(&self) -> &OsStr {
        self.inner.as_os_str()
    }
}

impl fmt::Debug for PathBuf {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(formatter)
    }
}

impl fmt::Display for PathBuf {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.display().fmt(formatter)
    }
}
