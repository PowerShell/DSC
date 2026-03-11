// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Provides helpers for working with JSON Schemas and VS Code.

pub mod dialect;
pub mod keywords;
pub mod transforms;
pub mod vocabulary;

mod schema_extensions;
pub use schema_extensions::VSCodeSchemaExtensions;
mod validation_options_extensions;
pub use validation_options_extensions::VSCodeValidationOptionsExtensions;
