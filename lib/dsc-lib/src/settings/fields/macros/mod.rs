// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines helper macros for generating fields.
//!
//! These macros are only usable within the `fields` module, and are not intended to be used
//! outside of this module. They simplify repetitive patterns for defining fields. The available
//! macros intended for direct use are:
//!
//! - [`define_boolean_field!`]: Defines a boolean settings field and the code default constant for
//!   the field.
//! - [`define_string_enum_field!`]: Defines a string enum settings field and the code default
//!   constant for the field.
//! - [`define_container_field!`]: Defines a container settings field, the resolved settings
//!   struct, the code default struct, and the code default constant for the container field.
//!
//! This module also defines [`utility`] macros that are not intended for direct use. They simplify
//! internal processing for the field definition macros.

pub(super) mod utility;

mod define_boolean_field;
pub(crate) use define_boolean_field::define_boolean_field;

mod define_string_enum_field;
pub(crate) use define_string_enum_field::define_string_enum_field;

mod define_container_field;
pub(crate) use define_container_field::define_container_field;

#[cfg(test)] mod tests;