// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::types::FullyQualifiedTypeName;

use clap::ValueEnum;
use indicatif::ProgressStyle;
use rust_i18n::t;
use serde::Serialize;
use serde_json::Value;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use tracing::{trace, warn_span};
use tracing::span::Span;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ProgressFormat {
    /// If interactive, use a progress bar. If not interactive, no progress is shown.
    Default,
    /// No progress is shown.
    None,
    /// Show progress as JSON.
    Json,
}

#[derive(Default, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Failure {
    pub message: String,
    pub exit_code: i32,
}

#[derive(Default, Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    /// The unique identifier for the operation.
    pub id: String,
    /// The activity being performed.
    pub activity: Option<String>,
    /// The total number of items to process.
    pub total_items: u64,
    /// The number of items processed.
    pub completed_items: u64,
    /// The status of the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// The number of seconds remaining in the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds_remaining: Option<u64>,
    /// The name of the resource being operated on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_name: Option<String>,
    /// The type of the resource being operated on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type: Option<FullyQualifiedTypeName>,
    /// The result of the operation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Failure information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<Failure>,
}

impl Progress {
    #[must_use]
    pub fn new(total_items: u64) -> Progress {
        Progress {
            id: Uuid::new_v4().to_string(),
            total_items,
            ..Default::default()
        }
    }
}

pub struct ProgressBar {
    progress_value:  Progress,
    console_bar: Span,
    format: ProgressFormat
}

impl ProgressBar {

    /// Create a `ProgressBar` object to update progress
    ///
    /// # Arguments
    ///
    /// * `item_count` - Total number of steps to complete.  Use '1' if unknown and increment when complete.
    /// * `format` - The `ProgressFormat` to render.
    ///
    /// # Returns
    ///
    /// A `ProgressBar` oject to update progress
    ///
    /// # Errors
    ///
    /// Fails if progress style for console rendering can't be set.
    ///
    pub fn new(total_items: u64, format: ProgressFormat) -> Result<ProgressBar, DscError> {
        let bar = warn_span!("");
        if format == ProgressFormat::Default {
            bar.pb_set_style(&ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise:.cyan}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg:.yellow}"
            )?);
            bar.pb_set_length(total_items);
            let _guard = bar.enter();
        }

        Ok(ProgressBar {
            progress_value: Progress::new(total_items),
            console_bar: bar,
            format
        })
    }

    /// Increment the progress bar by the specified amount and write the progress
    ///
    /// # Arguments
    ///
    /// * `delta` - The amount to increment the progress bar by
    ///
    pub fn write_increment(&mut self, delta: u64) {
        if self.format == ProgressFormat::None {
            return;
        }

        self.progress_value.completed_items += delta;
        if self.format == ProgressFormat::Json {
            self.write_json();
        } else {
            self.console_bar.pb_inc(delta);
        }
    }

    /// Set the resource being operated on
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the resource being operated on
    /// * `resource_type` - The type of the resource being operated on
    /// * `result` - The result of the operation
    ///
    pub fn set_resource(&mut self, name: &str, resource_type: &FullyQualifiedTypeName) {
        self.progress_value.resource_name = Some(name.to_string());
        self.progress_value.resource_type = Some(resource_type.clone());
        self.progress_value.result = None;
        self.progress_value.failure = None;
    }

    /// Set the result of the operation. This will clear any error.
    ///
    /// # Arguments
    ///
    /// * `result` - The result of the operation
    ///
    pub fn set_result(&mut self, result: &Value) {
        self.progress_value.failure = None;
        self.progress_value.result = Some(result.clone());
    }

    /// Indicate that the operation failed.  This will clear any result.
    ///
    pub fn set_failure(&mut self, failure: Option<Failure>) {
        self.progress_value.result = None;
        self.progress_value.failure = failure;
    }

    /// Set the status of the operation and write the progress
    ///
    /// # Arguments
    ///
    /// * `status` - The status of the operation
    ///
    pub fn write_activity(&mut self, activity: &str) {
        match self.format {
            ProgressFormat::Json => {
                self.progress_value.activity = Some(activity.to_string());
                self.write_json();
            },
            ProgressFormat::Default => {
                self.console_bar.pb_set_message(activity);
            },
            ProgressFormat::None => {}
        }
    }

    /// Set the number of total items to complete
    ///
    /// # Arguments
    ///
    /// * `len` - The number of total items to complete
    ///
    pub fn set_total_items(&mut self, len: u64) {
        match self.format {
            ProgressFormat::Json => {
                self.progress_value.total_items = len;
            },
            ProgressFormat::Default => {
                self.console_bar.pb_set_length(len);
            },
            ProgressFormat::None => {}
        }
    }

    fn write_json(&mut self) {
        if let Ok(json) = serde_json::to_string(&self.progress_value) {
            eprintln!("{json}");
        } else {
            trace!("{}", t!("progress.failedToSerialize", json = self.progress_value : {:?}));
        }
    }
}
