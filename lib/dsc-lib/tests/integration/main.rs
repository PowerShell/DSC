// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines integration tests for [`dsc-lib`].
//!
//! Instead of defining tests in each of the module files for the crate, we define them here as
//! integration tests to improve compilation times.
//! 
//! The tests in this module are for public code. The tests should validate expected behaviors at
//! the public API level. Don't add tests to this module for inner code behaviors.
//! 
//! We organize the tests in the `tests/integration` folder instead of directly in `tests` to
//! minimize compilation times. If we defined the tests one level higher in the `tests` folder,
//! Rust would generate numerous binaries to execute our tests.

#[cfg(test)] mod schemas;
