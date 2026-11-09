// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Represents the different formats that can be used for stderr output in tracing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StderrFormat {
    /// Emit trace events as human-readable text to stderr with ANSI color codes.
    Default,
    /// Emit trace events as human-readable text to stderr without ANSI color codes.
    Plaintext,
    /// Emit trace events as JSON to stderr.
    Json,
}