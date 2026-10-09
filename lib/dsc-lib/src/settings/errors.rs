// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use miette::Diagnostic;
use thiserror::Error;
use rust_i18n::t;

use crate::settings::{DscSettingsScope, fields::*};

#[cfg(doc)]
pub(in crate::settings) mod _implementation_guidance {
    //! Any failure state that can occur while working with DSC settings should have a
    //! corresponding error variant in `DscSettingsError`.
    //!
    //! # General Guidance
    //!
    //! Every variant should:
    //!
    //! 1. Be defined as a struct variant with named fields, even if it only contains a single
    //!    field. This improves discoverability and enables us to pass the context upward without
    //!    needing to return to the error construction site to discover what the arbitrary string
    //!    indicates.
    //!
    //! 1. Define the `error` attribute for the variant with a translated error. Use the `t!` macro
    //!    to provide the display string. Use the following pattern:
    //!
    //!    ```rust, ignore
    //!    #[error("{t}", t = t!("settings.errors.<camelCasedVariant>", ...))]
    //!    ```
    //!
    //!    Always define the translation lookup key as the camelCased variant name.
    //!
    //!    When you need to interpolate text from the error variant into the translation, always
    //!    pass it as a named argument to the `t!` macro. Don't append it to the error format
    //!    string itself.
    //!
    //!    For example, use this construction:
    //!
    //!    ```ignore
    //!     #[error("{t}", t = t!("settings.errors.invalidTraceLevel", text = text))]
    //!     InvalidTraceLevel {
    //!         text: String,
    //!     }
    //!    ```
    //!
    //!    Not this construction:
    //!
    //!    ```ignore
    //!     #[error("{t}: {text}", t = t!("settings.errors.invalidTraceLevel"))]
    //!     InvalidTraceLevel {
    //!         text: String,
    //!     }
    //!    ```
    //!
    //!    This ensures that all error messages are consistently translated and formatted and makes
    //!    the translation definition the source of truth for error message content.
    //!
    //! The rest of the guidance relates to specific kinds of error variants.
    //!
    //! # String enum errors
    //!
    //! These error variants are raised when parsing a string value into a specific enum type,
    //! like [`TracingLevelField`] or [`TracingFormatField`].
    //!
    //! The shape of the error variant for string enums should always look like the following:
    //!
    //! ```rust, ignore
    //! /// Indicates that an input string failed to parse case-insensitively into a valid variant
    //! /// of [`<EnumType>`].
    //! #[error("{t}", t = t!(
    //!     "settings.errors.invalid<FieldType>",
    //!     text = text,
    //!     variant_list = <EnumType>::VARIANT_LIST,
    //!     variant_last = <EnumType>::VARIANT_LAST,
    //! ))]
    //! Invalid<FieldType> {
    //!     text: String,
    //! },
    //! ```
    //!
    //! Where:
    //!
    //! 1. The variant is named is the type name of the string enum type prefixed with `Invalid`
    //!    and with the `Field` suffix removed, like [`InvalidTracingLevel`] for
    //!    [`TracingLevelField`].
    //! 1. The i18n key used in the `t!` macro should be `"settings.errors.<camelCasedVariant>"`,
    //!    like `"settings.errors.invalidTracingLevel"` for [`InvalidTracingLevel`].
    //! 1. The `t!` macro should always pass the text from the error variant and the list of valid
    //!    variants so the translation can correctly construct the error message.
    //!
    //! You can use the `string_enum_error` snippet to quickly insert a new error variant for an
    //! invalid string enum value.
    //!
    //! ## Collected errors
    //!
    //! These errors are used to aggregate multiple errors that occur while processing a single
    //! operation. Where possible, we want to _collect_ errors for user visibility rather than
    //! failing fast and leaving the user to fix one error at a time.
    //!
    //! For example, [`LoadMultipleErrors`] collects all errors encountered when reading a settings
    //! file, allowing the user to see all issues at once rather than failing at the first error.
    //!
    //! The shape of a collected errors variant should look like the following:
    //!
    //! ```rust, ignore
    //! #[error("{t}", t = t!(
    //!     "settings.errors.<camelCasedVariantName>",
    //!     err = DscSettingsError::collect_errors_for_message(&errors),
    //!     // hoist additional context from the variant fields as needed.
    //! ))]
    //! <VariantPrefix>MultipleErrors {
    //!     /// Collected errors encountered while <describe context>.
    //!     #[related]
    //!     errors: Vec<DscSettingsError>,
    //!     // Define additional fields as needed to provide context for the collected errors.
    //! },
    //! ```
    //!
    //! Where:
    //!
    //! 1. The variant is named coherently for the type of errors it collects and has the suffix
    //!    `MultipleErrors`.
    //! 1. The variant always defines a field named `errors` of type `Vec<DscSettingsError>` and
    //!    has the `#[related]` attribute.
    //! 1. The i18n key used in the `t!` macro should be `"settings.errors.<camelCasedVariant>"`,
    //!    like `"settings.errors.loadMultipleErrors"` for [`LoadMultipleErrors`].
    //! 1. The `t!` macro must include the `err` argument, which should be generated using
    //!    `DscSettingsError::collect_errors_for_message(errors)` to concatenate the error
    //!    messages from all collected errors into a single string.
    //! 1. If additional context is useful, the variant defines fields for that context and passes
    //!    them to the `t!` macro as additional arguments if needed.
    //!
    //! [`TracingLevelField`]: crate::settings::fields::TracingLevelField
    //! [`TracingFormatField`]: crate::settings::fields::TracingFormatField
    //! [`InvalidTracingLevel`]: super::DscSettingsError::InvalidTracingLevel
    //! [`LoadMultipleErrors`]: super::DscSettingsError::LoadMultipleErrors
}

#[non_exhaustive]
#[derive(Error, Debug, Diagnostic)]
pub enum DscSettingsError {
    /// Indicates that an invalid string representation of [`DscSettingsScope`]
    /// was provided.
    #[error("{t}", t = t!(
        "settings.errors.invalidScope",
        text = text,
        variant_list = DscSettingsScope::VARIANT_LIST,
        variant_last = DscSettingsScope::VARIANT_LAST,
    ))]
    InvalidScope {
        /// The text that failed to parse as a [`DscSettingsScope`] variant.
        text: String,
    },

    /// Indicates that an invalid string representation of [`TracingLevelField`]
    /// was provided.
    #[error("{t}", t = t!(
        "settings.errors.invalidTracingLevel",
        text = text,
        variant_list = TracingLevelField::VARIANT_LIST,
        variant_last = TracingLevelField::VARIANT_LAST,
    ))]
    InvalidTracingLevel{
        /// The text that failed to parse as a [`TracingLevelField`] variant.
        text: String,
    },

    /// Indicates that an invalid string representation of [`TracingFormatField`]
    /// was provided.
    #[error("{t}", t = t!(
        "settings.errors.invalidTracingFormat",
        text = text,
        variant_list = TracingFormatField::VARIANT_LIST,
        variant_last = TracingFormatField::VARIANT_LAST,
    ))]
    InvalidTracingFormat{
        /// The text that failed to parse as a [`TracingFormatField`] variant.
        text: String,
    },

    /// Indicates that a settings preference or policy file contained no text
    /// and couldn't be parsed into settings.
    #[error("{t}", t = t!("settings.errors.dataFileEmpty", file_path = file_path, scope = scope))]
    DataFileEmpty {
        /// The path to the settings preference or policy file that was empty.
        file_path: String,
        /// The scope for the settings file
        scope: String,
    },

    /// Indicates that a settings preference or policy file couldn't be read
    /// from the file system.
    ///
    /// This error is only raised when the file exists but reading it raises a
    /// [`std::io::Error`].
    #[error("{t}", t = t!(
        "settings.errors.dataFileReadError",
        file_path = file_path,
        scope = scope,
        err = source,
    ))]
    DataFileReadError {
        /// The path to the settings preference or policy file that couldn't be
        /// read.
        file_path: String,
        /// The scope for the settings file.
        scope: String,
        /// The underlying I/O error that occurred while trying to read the file.
        #[source]
        source: std::io::Error,
    },

    /// Indicates that the settings preference or policy file was defined with
    /// some content that couldn't be parsed as the appropriate data.
    #[error("{t}", t = t!(
        "settings.errors.dataFileUnparseable",
        file_path = file_path,
        scope = scope,
        err = source,
    ))]
    DataFileUnparseable{
        /// The path to the settings preference or policy file that couldn't be
        /// parsed.
        file_path: String,
        /// The scope for the settings file.
        scope: String,
        /// The underlying error that occurred while trying to parse the file.
        #[source]
        source: serde_json::Error,
    },

    /// Indicates that an environment variable's value couldn't be parsed as a
    /// boolean.
    ///
    /// DSC recognizes (case-insensitively):
    ///
    /// - `true` and `1` as boolean true values
    /// - `false` and `0` as boolean false values.
    ///
    /// Any other value meant to parse as a boolean from the environment
    /// variable raises this error.
    #[error("{t}", t = t!(
        "settings.errors.envVarUnparseableBoolean",
        value = value,
        expected_list = "`true`, `false`, `1`",
        expected_last = "`0`"
    ))]
    EnvVarUnparseableBoolean{
        /// The value of the environment variable that couldn't be parsed as a
        /// boolean.
        value: String,
    },

    /// Indicates that multiple errors occurred while loading settings.
    ///
    /// DSC aggregates load errors to return them together instead of failing
    /// immediately on the first error. This helps users more quickly address
    /// issues in their settings definitions.
    #[error("{t}", t = t!(
        "settings.errors.loadMultipleErrors",
        err = DscSettingsError::collect_errors_for_message(errors)
    ))]
    LoadMultipleErrors{
        /// The collection of errors raised while loading settings.
        #[related]
        errors: Vec<DscSettingsError>,
    },

    /// Indicates an error when loading an environment variable as a setting.
    #[error("{t}", t = t!(
        "settings.errors.loadEnvironmentError",
        env_var = env_var,
        err = source,
    ))]
    LoadEnvironmentError{
        /// The name of the environment variable that failed to load.
        env_var: &'static str,
        /// The underlying error that occurred while trying to load the
        /// environment variable.
        #[source]
        source: Box<DscSettingsError>,
    },

    /// Indicates that multiple errors occurred while loading settings
    /// environment variables.
    ///
    /// DSC aggregates load errors from environment variables to return them
    /// together instead of failing immediately on the first error. This helps
    /// users more quickly address issues in their environment variable settings.
    #[error("{t}", t = t!(
        "settings.errors.loadEnvironmentMultipleErrors",
        err = DscSettingsError::collect_errors_for_message(errors)
    ))]
    LoadEnvironmentMultipleErrors{
        /// The collection of errors raised while loading environment variables
        /// as settings.
        #[related]
        errors: Vec<DscSettingsError>,
    },
}

impl DscSettingsError {
    /// Collects the error messages from a list of `DscSettingsError` instances into a single string.
    ///
    /// # Arguments
    ///
    /// * `errors` - A slice of `DscSettingsError` instances to collect messages from.
    ///
    /// # Returns
    ///
    /// A single string containing the concatenated error messages, separated by commas followed by
    /// a single space.
    pub fn collect_errors_for_message(errors: &[DscSettingsError]) -> String {
        errors
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }
}