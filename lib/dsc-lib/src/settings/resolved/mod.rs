// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines the types for the resolved settings.
//!
//! This module defines two types:
//!
//! - [`DscSettingsResolvedField`] is a generic struct that colocates a resolved setting value
//!   with the highest precedence scope it was defined in.
//! - [`DscSettingsResolved`] is a struct that contains all the resolved settings for DSC. Every
//!   leaf field in this struct is a [`DscSettingsResolvedField`] and every container field is
//!   a struct that contains other container fields and/or leaf fields.

#[cfg(doc)]
pub(in crate::settings) mod _implementation_guidance {
    //! Generally, only the [`DscSettingsResolved`] type should require any modification when
    //! updating settings definitions. For more information, see the
    //! [implementation guidance](super::settings::_implementation_guidance).
    //!
    //! [`DscSettingsResolved`]: super::DscSettingsResolved
}

mod field;
pub use field::*;
mod settings;
pub use settings::*;
