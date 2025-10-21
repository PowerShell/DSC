// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Helper library for working with DSC and JSON Schemas.

use rust_i18n::i18n;

#[macro_use]
pub mod macros;

pub mod schema_utility_extensions;
pub mod transforms;
pub mod vscode;

#[cfg(test)]
mod tests;

// Enable localization for emitted strings
i18n!("locales", fallback = "en-us");
