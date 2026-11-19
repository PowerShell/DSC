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

// Re-export the tracing* and opentlemetry* crates for convenience so that
// other crates can use them by taking a dependency only on the telemetry crate.
pub use opentelemetry as otel;
pub use opentelemetry_sdk as otel_sdk;
pub use opentelemetry_semantic_conventions as otel_conventions;
pub use tracing;
pub use tracing_indicatif;
pub use tracing_log;
pub use tracing_opentelemetry as tracing_otel;
pub use tracing_subscriber;

mod formatters {
    pub(super) mod tracing_event_formatter;
}
pub use formatters::tracing_event_formatter::DscTracingEventFormatter;

mod options;
pub use options::DscTracingOptions;

mod stderr_format;
pub use stderr_format::StderrFormat;
