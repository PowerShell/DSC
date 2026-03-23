// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines unit tests for [`dsc-lib-jsonschema`].
//!
//! Instead of defining tests in each of the module files for the crate, we
//! define them in this module to improve compilation times. The tests in this
//! module are for internal code. Do not define tests for public items in this
//! module. Instead, define those tests in the `tests/integration` folder,
//! which forces usage of the crate as a public API.
//!
//! When you define tests in this module, ensure that you mirror the structure
//! of the modules from the rest of the source tree.

#[cfg(test)] mod dsc_repo;
#[cfg(test)] mod schema_utility_extensions;
#[cfg(test)] mod vscode;
