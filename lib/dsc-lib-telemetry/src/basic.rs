// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! This module provides basic tracing functionality for applications that do not require full
//! OpenTelemetry support.
//! 
//! It bundles the [`tracing`] crate and related crates to provide a simple way to emit trace
//! events to stderr in a human-readable format.
//! 
//! This module simplifies the process of setting up a tracing subscriber with configurable options
//! for tracing level and stderr output format. It also provides a default guard for capturing
//! events emitted during application initialization, enabling developers to set up their own
//! subscriber later in the application lifecycle.

// Re-export the tracing crates so that other crates can use them without
// having to add them as separate dependencies.
pub use tracing;
pub use tracing_indicatif;
pub use tracing_subscriber;

use crate::StderrFormat;

use tracing::{Level, subscriber::DefaultGuard};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{
    EnvFilter,
    Layer,
    Registry,
    fmt::Layer as FormatLayer,
    layer::{Layered, SubscriberExt}
};

// Define type aliases for the boxed layer and the basic tracing subscriber to simplify their usage
// in the code. We don't provide documentation for these type aliases because that overrides the
// documentation for the underlying types, which is more useful to users of this crate.
pub type BoxedLayer = Box<dyn Layer<Registry> + Send + Sync>;
pub type BasicTracingSubscriber = Layered<
    IndicatifLayer<Layered<EnvFilter, Layered<BoxedLayer, Registry>>>,
    Layered<EnvFilter, Layered<BoxedLayer, Registry>>
>;

/// Represents the options for basic tracing functionality in DSC.
///
/// This struct allows users to configure the tracing level and the format of stderr output for
/// trace events. It's only used for the legacy basic tracing functionality, not for OpenTelemetry
/// tracing, logging, or metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasicTracingOptions {
    /// Indicates the minimum level of tracing events to be recorded. Events below this level are
    /// filtered out and not emitted. The default level is [`Level::INFO`].
    pub tracing_level: Level,
    /// Specifies the format of stderr output for trace events. The default format is
    /// [`StderrFormat::Default`].
    pub stderr_format: StderrFormat,
}

impl Default for BasicTracingOptions {
    fn default() -> Self {
        BasicTracingOptions {
            tracing_level: Level::INFO,
            stderr_format: StderrFormat::Default,
        }
    }
}

impl BasicTracingOptions {
    /// Returns a [`DefaultGuard`] that sets up a default tracing subscriber for capturing events
    /// emitted during the initialization of the application.
    ///
    /// The default subscriber is configured with a filter that captures [`Level::WARN`] events and
    /// higher, and it uses an [`IndicatifLayer`] for progress reporting. The subscriber is set as
    /// the default, allowing it to capture events emitted during initialization.
    ///
    /// # Example
    ///
    /// The following example demonstrates how to use this method to setup a default guard that you
    /// use during the initialization of your application before dropping it to use your own
    /// subscriber.
    ///
    /// ```rust
    /// # use dsc_lib_telemetry::basic::BasicTracingOptions;
    /// let default_guard = BasicTracingOptions::init_default_guard();
    ///
    /// // Perform initialization tasks that may emit tracing events here;
    /// // For this example, we're just assigning values to the tracing level
    /// // and stderr format.
    /// let tracing_level = tracing::Level::INFO;
    /// let stderr_format = dsc_lib_telemetry::StderrFormat::Default;
    ///
    /// // Initialize your own subscriber based on the desired tracing level
    /// // and stderr format.
    /// let subscriber = dsc_lib_telemetry::basic::BasicTracingOptions {
    ///     tracing_level,
    ///     stderr_format
    /// }.init_subscriber();
    ///
    /// // Drop the default guard to stop capturing events with the default
    /// // subscriber and set your own subscriber as the global default.
    /// drop(default_guard);
    /// if tracing::subscriber::set_global_default(subscriber).is_err() {
    ///     eprintln!("Failed to set tracing subscriber as global default.");
    /// }
    /// ```
    pub fn init_default_guard() -> DefaultGuard {
        let default_filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("warn"))
            .unwrap_or_default()
            .add_directive(Level::WARN.into());
        let default_indicatif_layer = IndicatifLayer::new();
        let default_layer = FormatLayer::default()
            .with_writer(default_indicatif_layer.get_stderr_writer());
        let default_fmt = default_layer
            .with_ansi(true)
            .with_level(true)
            .boxed();
        let default_subscriber = tracing_subscriber::Registry::default()
            .with(default_fmt)
            .with(default_filter)
            .with(default_indicatif_layer);
        let default_guard = tracing::subscriber::set_default(default_subscriber);

        default_guard
    }

    /// Initializes a tracing subscriber based on the provided [`BasicTracingOptions`].
    ///
    /// This method sets up a tracing subscriber that captures events at the specified tracing level and formats stderr
    /// output according to the specified format. The subscriber is configured with an `IndicatifLayer` for progress
    /// reporting.
    ///
    /// # Example
    ///
    /// The following example demonstrates how to use this method to initialize a tracing
    /// subscriber and set it as the global default subscriber.
    ///
    /// ```rust
    /// # use dsc_lib_telemetry::basic::{BasicTracingOptions, StderrFormat};
    /// # use tracing::Level;
    ///
    /// let tracing_options = BasicTracingOptions {
    ///    tracing_level: Level::INFO,
    ///    stderr_format: StderrFormat::Json,
    /// };
    /// let subscriber = tracing_options.init_subscriber();
    ///
    /// if tracing::subscriber::set_global_default(subscriber).is_err() {
    ///     eprintln!("Failed to set tracing subscriber as global default.");
    /// }
    /// ```
    ///
    /// # Returns
    ///
    /// [`BasicTracingSubscriber`] that can be set as the global default subscriber using
    /// [`tracing::subscriber::set_global_default`].
    pub fn init_subscriber(&self) -> BasicTracingSubscriber {
        let with_source = self.tracing_level == Level::DEBUG || self.tracing_level == Level::TRACE;
        let filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("warn"))
            .unwrap_or_default()
            .add_directive(self.tracing_level.into());
        let indicatif_layer = IndicatifLayer::new();
        let layer = FormatLayer::default()
            .with_writer(indicatif_layer.get_stderr_writer())
            .with_level(true)
            .with_target(with_source)
            .with_line_number(with_source);
        let format_layer = match self.stderr_format {
            StderrFormat::Default => layer.with_ansi(true).boxed(),
            StderrFormat::Plaintext => layer.with_ansi(false).boxed(),
            StderrFormat::Json => layer.with_ansi(false).json().boxed(),
        };

        Registry::default()
            .with(format_layer)
            .with(filter)
            .with(indicatif_layer)
    }
}
