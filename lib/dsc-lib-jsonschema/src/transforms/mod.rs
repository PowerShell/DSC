// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines various functions that implement the [`Transform`] trait for [`schemars`], enabling you
//! modify generated JSON Schemas.
//!
//! [`Transform`]: schemars::transform

mod canonicalize_refs_and_defs;
pub use canonicalize_refs_and_defs::canonicalize_refs_and_defs;
mod idiomaticize_externally_tagged_enum;
pub use idiomaticize_externally_tagged_enum::idiomaticize_externally_tagged_enum;
mod idiomaticize_string_enum;
pub use idiomaticize_string_enum::idiomaticize_string_enum;
mod remove_bundled_schema_resources;
pub use remove_bundled_schema_resources::remove_bundled_schema_resources;
