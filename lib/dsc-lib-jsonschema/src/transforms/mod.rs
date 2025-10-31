// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines various functions that implement the [`Transform`] trait for [`schemars`], enabling you
//! modify generated JSON Schemas.
//!
//! [`Transform`]: schemars::transform

mod idiomaticize_externally_tagged_enum;
pub use idiomaticize_externally_tagged_enum::idiomaticize_externally_tagged_enum;
mod idiomaticize_string_enum;
pub use idiomaticize_string_enum::idiomaticize_string_enum;
