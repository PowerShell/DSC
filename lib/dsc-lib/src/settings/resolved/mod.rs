//! Defines the types for the resolved settings.
//! 
//! This module defines two types:
//! 
//! - [`DscSettingsResolvedField`] is a generic struct that colocates a resolved setting value with
//!   the highest precedence scope it was defined in.
//! - [`DscSettingsResolved`] is a struct that contains all the resolved settings for DSC. Every
//!   leaf field in this struct is a [`DscSettingsResolvedField`] and every container field is
//!   a struct that contains other container fields and/or leaf fields.
//! 
//! Generally, only the [`DscSettingsResolved`] type should require any modification when updating
//! settings definitions.

mod field;
pub use field::*;
mod settings;
pub use settings::*;
