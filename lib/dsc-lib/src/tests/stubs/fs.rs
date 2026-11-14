// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::tests::stubs::path::{Path, StubbedPathContextMap};

/// Reads the contents of a file at the given path according to the stubbed
/// context.
///
/// This function simulates [`std::fs::read_to_string`] relying on the
/// stubbed path context provided by [`StubbedPathContextMap`].
///
/// # Arguments
///
/// - `path` - The path to the file to read.
///
/// # Returns
///
/// A [`std::io::Result<String>`] containing the file contents if successful,
/// or an error otherwise.
pub(crate) fn read_to_string<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    StubbedPathContextMap::find(path.as_ref()).read_result
}
