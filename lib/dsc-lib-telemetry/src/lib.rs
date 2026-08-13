// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This library provides telemetry support for DSC.
//!
//! It includes functionality for OpenTelemetry tracing, metrics, and exporting telemetry data to
//! various backends. Other crates can take a dependency on this crate to leverage its OTel
//! capabilities and keep a consistent implementation and dependencies.
//!
//! This crate also supports basic tracing functionality for applications that do not require full
//! OpenTelemetry support. This functionality bundles the [`tracing`] crate and related crates to
//! provide a simple way to emit trace events to stderr in a human-readable format.

pub mod basic;

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
