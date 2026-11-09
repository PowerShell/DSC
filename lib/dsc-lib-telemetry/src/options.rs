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

use crate::{
    StderrFormat,
    formatters::tracing_event_formatter::DscTracingEventFormatter,
    otel::trace::TracerProvider,
    tracing::{Level, subscriber::DefaultGuard},
    tracing_indicatif::{IndicatifLayer, style::ProgressStyle},
    tracing_subscriber::{
        EnvFilter, Layer, Registry, fmt::Layer as FormatLayer, layer::SubscriberExt,
    },
};

/// Represents the options for tracing functionality in DSC.
///
/// This struct allows users to configure the tracing level and the format of stderr output for
/// trace events. It also provides an option to enable OpenTelemetry tracing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DscTracingOptions {
    /// Indicates the minimum level of tracing events to be recorded. Events below this level are
    /// filtered out and not emitted. The default level is [`Level::INFO`].
    pub tracing_level: Level,
    /// Specifies the format of stderr output for trace events. The default format is
    /// [`StderrFormat::Default`].
    pub stderr_format: StderrFormat,
    /// Indicates whether to enable OpenTelemetry tracing. The default is `false`.
    pub enable_otel: bool,
}

impl Default for DscTracingOptions {
    fn default() -> Self {
        DscTracingOptions {
            tracing_level: Level::INFO,
            stderr_format: StderrFormat::Default,
            enable_otel: false,
        }
    }
}

impl DscTracingOptions {
    /// Returns a [`DefaultGuard`] that sets up a default tracing subscriber for capturing events
    /// emitted during the initialization of the application.
    ///
    /// The default subscriber is configured with a filter that captures [`Level::WARN`] events and
    /// higher, and it uses an [`IndicatifLayer`] for progress reporting. The subscriber is set as
    /// the default, allowing it to capture events emitted during initialization.
    /// Text events do not include span names or span fields.
    ///
    /// # Example
    ///
    /// The following example demonstrates how to use this method to setup a default guard that you
    /// use during the initialization of your application before dropping it to use your own
    /// subscriber.
    ///
    /// ```rust
    /// # use dsc_lib_telemetry::DscTracingOptions;
    /// let default_guard = DscTracingOptions::init_default_guard();
    ///
    /// // Perform initialization tasks that may emit tracing events here;
    /// // For this example, we're just assigning values to the tracing level,
    /// // stderr format, and enabling otel.
    /// let tracing_level = tracing::Level::INFO;
    /// let stderr_format = dsc_lib_telemetry::StderrFormat::Default;
    /// let enable_otel = true;
    ///
    /// // Initialize your own subscriber based on the desired tracing level
    /// // and stderr format.
    /// let subscriber = dsc_lib_telemetry::DscTracingOptions {
    ///     tracing_level,
    ///     stderr_format,
    ///     enable_otel
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
        let default_layer =
            FormatLayer::default().with_writer(default_indicatif_layer.get_stderr_writer());
        let default_fmt = default_layer.with_ansi(true).with_level(true).boxed();
        let default_subscriber = tracing_subscriber::Registry::default()
            .with(default_fmt)
            .with(default_filter)
            .with(default_indicatif_layer);

        tracing::subscriber::set_default(default_subscriber)
    }

    /// Initializes a tracing subscriber based on the provided [`DscTracingOptions`].
    ///
    /// This method sets up a tracing subscriber that captures events at the specified tracing level and formats stderr
    /// output according to the specified format. The subscriber is configured with an `IndicatifLayer` for progress
    /// reporting.
    /// Text events omit span names and span fields; JSON events retain span information.
    ///
    /// # Example
    ///
    /// The following example demonstrates how to use this method to initialize a tracing
    /// subscriber and set it as the global default subscriber.
    ///
    /// ```rust
    /// # use dsc_lib_telemetry::{DscTracingOptions, StderrFormat};
    /// # use tracing::Level;
    ///
    /// let tracing_options = DscTracingOptions {
    ///    tracing_level: Level::INFO,
    ///    stderr_format: StderrFormat::Json,
    ///    enable_otel: true,
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
    /// A `tracing::Subscriber` that can be set as the global default subscriber using
    /// [`tracing::subscriber::set_global_default`].
    pub fn init_subscriber(&self) -> Box<dyn tracing::Subscriber + Send + Sync> {
        // Box the returned subscriber to match the return type
        if self.enable_otel {
            self.init_otel_subscriber()
        } else {
            self.init_basic_subscriber()
        }
    }
}

// Private API
impl DscTracingOptions {
    /// Indicates whether to include source data in events and traces.
    /// 
    /// # Returns
    /// 
    /// Returns `true` if the tracing level is debug or trace and otherwise `false`.
    fn with_source(&self) -> bool {
        self.tracing_level == Level::DEBUG || self.tracing_level == Level::TRACE
    }
    /// Returns an `EnvFilter` configured with the tracing level and environment settings.
    /// 
    /// # Returns
    /// 
    /// An `EnvFilter` instance.
    fn filter(&self) -> EnvFilter {
        EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new("warn"))
            .unwrap_or_default()
            .add_directive(self.tracing_level.into())
    }
    /// Returns an `IndicatifLayer` configured with an empty prefix template.
    /// 
    /// This implementation gets around the formatting for the progress bar
    /// with spans that makes the progress bar multiline and provides context
    /// that will only distract users.
    /// 
    /// # Returns
    /// 
    /// An `IndicatifLayer` instance.
    fn indicatif_layer<S>(&self) -> IndicatifLayer<S>
    where
        S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    {
        IndicatifLayer::new().with_progress_style(
            ProgressStyle::default_bar().template("").unwrap(),
        )
    }
    /// Returns a `FormatLayer` configured with the appropriate event formatter and ANSI settings.
    /// 
    /// # Returns
    /// 
    /// A boxed `FormatLayer` instance.
    fn format_layer<S>(&self, indicatif_layer: &IndicatifLayer<S>) -> Box<dyn Layer<S> + Send + Sync>
    where
        S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a> + 'static,
    {
        let layer = FormatLayer::default().with_writer(indicatif_layer.get_stderr_writer());
        let event_format = DscTracingEventFormatter {
            with_source: self.with_source(),
            format: self.stderr_format,
        };
        match self.stderr_format {
            StderrFormat::Default => layer.with_ansi(true).event_format(event_format).boxed(),
            StderrFormat::Plaintext => layer.with_ansi(false).event_format(event_format).boxed(),
            StderrFormat::Json => layer
                .with_ansi(false)
                .json()
                .event_format(event_format)
                .boxed(),
        }
    }

    /// Initializes a tracing subscriber without OpenTelemetry.
    /// 
    /// # Returns
    /// 
    /// A boxed `tracing::Subscriber` instance.
    fn init_basic_subscriber(&self) -> Box<dyn tracing::Subscriber + Send + Sync> {
        let indicatif_layer = self.indicatif_layer();
        let format_layer = self.format_layer(&indicatif_layer);
        let filter = self.filter();

        Box::new(
            Registry::default()
                .with(format_layer.and_then(indicatif_layer))
                .with(filter)
        )
    }

    /// Initializes a tracing subscriber with OpenTelemetry.
    /// 
    /// # Returns
    /// 
    /// A boxed `tracing::Subscriber` instance.
    fn init_otel_subscriber(&self) -> Box<dyn tracing::Subscriber + Send + Sync> {
        let indicatif_layer = self.indicatif_layer();
        let format_layer = self.format_layer(&indicatif_layer);
        let filter = self.filter();

        let otel_provider = crate::otel_sdk::trace::SdkTracerProvider::builder()
            // For now, use noop exporter - need to wire for extension output
            .with_batch_exporter(crate::otel_sdk::testing::trace::NoopSpanExporter::default())
            .build();
        let tracer = otel_provider.tracer("dsc-lib-telemetry");
        let telemetry = crate::tracing_otel::layer()
            .with_tracer(tracer)
            .with_level(true)
            .with_target(self.with_source())
            .with_location(self.with_source());

        Box::new(
            Registry::default()
                .with(telemetry)
                .with(format_layer.and_then(indicatif_layer))
                .with(filter)
        )
    }
}
