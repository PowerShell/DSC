// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines unit tests and testing stubs.
//!
//! Generally, instead of defining unit tests in the library source code, we should instead define
//! integration tests in `tests/integration`.
//!
//! Only private API tests and tests that require stubbing external systems, like the file system
//! or environment variables, should be included here.

#[macro_use]
pub(crate) mod stubs;

mod settings;
