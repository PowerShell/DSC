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

#[macro_use]
pub(crate) mod macros {
    //! Defines macros for simplifying integration tests.
    //!
    //! This module includes helper macros to reduce boiler plate and ensure consistent
    //! implementation across integration tests. While each of the macros can be used independently,
    //! the most ergonomic option is to use [`test_dsc_repo_schema!`] to handle the creation of the
    //! test module for verifying schema correctness and documentation.

    mod define_schema_statics;
    pub(crate) use define_schema_statics::define_schema_statics;
    mod test_schema_docs;
    pub(crate) use test_schema_docs::test_schema_docs;
    mod test_meta_schema;
    pub(crate) use test_meta_schema::test_meta_schema;
    mod test_schema_validation;
    pub(crate) use test_schema_validation::test_schema_validation;
    mod test_dsc_repo_schema;
    pub(crate) use test_dsc_repo_schema::test_dsc_repo_schema;
}
#[macro_use]
pub(crate) mod schemas;
mod command_resource;
mod types;
mod settings;
