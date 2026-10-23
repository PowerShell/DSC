// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Diagnostics that point at the text they apply to.
//!
//! DSC reads documents and resource input as JSON or YAML text and validates the parsed value.
//! The types here keep the connection between a problem found in the value and the text it came
//! from, so a host can show the problem in place rather than only describe it:
//!
//! - [`SourceMap`] finds the byte span and line/column of any JSON pointer in JSON or YAML text.
//! - [`SchemaValidationError`] is a [`miette::Diagnostic`] for JSON schema violations, with a
//!   label on each violation when its source text is attached, and a [`JsonOrigin`] that says
//!   whether the value was input to a resource or output from one.
//! - [`DocumentSource`] carries the text and name of a document a host read, so library code
//!   can anchor the problems it finds to the author's text.

pub mod schema;
pub mod source_map;

pub use schema::{JsonOrigin, SchemaValidationError};
pub use source_map::SourceMap;

/// The text a document was read from, for diagnostics that point into it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSource {
    /// How to name the text in a report: the file it was read from when known, otherwise a
    /// description like `input`.
    pub name: String,
    /// The document text as the author wrote it, as JSON or YAML.
    pub text: String,
}

impl DocumentSource {
    /// Create a document source.
    ///
    /// # Arguments
    ///
    /// * `name` - How to name the text in a report.
    /// * `text` - The document text.
    #[must_use]
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        Self { name: name.into(), text: text.into() }
    }
}
