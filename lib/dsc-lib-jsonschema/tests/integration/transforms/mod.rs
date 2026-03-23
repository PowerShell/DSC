// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Integration tests for [`dsc-lib-jsonschema::transforms`]. This module defines functions that
//! a user can add with the `#[schemars(transform = <function_name>)]` attribute to modify the
//! generated schema.

#[cfg(test)] mod canonicalize_refs_and_defs;
#[cfg(test)] mod idiomaticize_externally_tagged_enum;
#[cfg(test)] mod idiomaticize_string_enum;
#[cfg(test)] mod remove_bundled_schema_resources;
