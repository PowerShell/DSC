// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Unit tests for DSC Settings
//!
//! This module contains unit tests for the DSC Settings functionality, including tests for
//! loading settings from various sources such as environment variables, policy files, and
//! preference files.
//!
//! These unit tests are defined to take advantage of mocking the filesystem and environment
//! variables to enable isolated testing for the behavior of the public API for [`DscSettings`],
//! [`PreferenceFileData`], [`PolicyFileData`], and [`EnvironmentData`].
//!
//! [`DscSettings`]: crate::settings::DscSettings
//! [`PreferenceFileData`]: crate::settings::sources::PreferenceFileData
//! [`PolicyFileData`]: crate::settings::sources::PolicyFileData
//! [`EnvironmentData`]: crate::settings::sources::EnvironmentData
//!
//! The majority of tests for the [`crate::settings`] module are located in the integration test
//! suite.

/// Testing for the [`DscSettings`] struct.
///
/// [`DscSettings`]: crate::settings::DscSettings
mod dsc_settings;
/// Testing for the various sources that DSC settings uses.
mod sources {
    /// Testing for [`EnvironmentData`], which loads settings from environment variables.
    ///
    /// [`EnvironmentData`]: crate::settings::sources::EnvironmentData
    mod environment;
    /// Testing for [`PolicyFileData`], which loads settings from the machine policy file.
    ///
    /// [`PolicyFileData`]: crate::settings::sources::PolicyFileData
    mod policy_file;
    /// Testing for [`PreferenceFileData`], which loads settings from the user preference file.
    ///
    /// [`PreferenceFileData`]: crate::settings::sources::PreferenceFileData
    mod preference_file;
}