// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Provides test stubs for standard library modules like [`std::fs`] and [`std::path`].

/// Provides stubs for [`std::env`]
#[macro_use]
pub(crate) mod env;

/// Provides stubs for [`std::fs`]
pub(crate) mod fs;

/// Provides stubs for [`std::path`]
#[macro_use]
pub(crate) mod path;
