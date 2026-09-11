//! Defines the fields used in DSC settings.
//! 
//! This documentation describes the structure and provides guidance for implementing new fields in
//! DSC settings.
//! 
//! Follow this guidance when defining new top-level fields in DSC settings:
//! 
//! 1. Define a private submodule for the setting field in this module. The submodule should have
//!    the same name as the field.
//! 1. Re-export all public items from the submodule in this module.
//! 
//! For example, if the field is `new_area`, you would add the following lines to this file:
//! 
//! ```ignore
//! mod new_area;
//! pub use new_area::*;
//! ```
//! 
//! Additional guidance is provided in the following sections for defining leaf fields and
//! container fields. For the purposes of this guidance, a container field is a field that has one
//! or more subfields (like [`resource_path`]), while a leaf field is a field that doesn't have any
//! subfields (like [`forbid_ignore_settings_file`]).
//! 
//! # Leaf fields
//! 
//! In the submodule for a leaf field, follow this guidance to define the types and constants
//! needed to represent the field in DSC settings:
//! 
//! 1. If the field requires a new type to represent its value, define the type in the submodule:
//! 
//!   - Use the naming convention `<FieldName>Field`, like `NewAreaField`.
//!   - Implement (or derive) [`Clone`], [`Debug`], [`PartialEq`], [`Eq`], [`Serialize`],
//!     [`Deserialize`], and [`JsonSchema`] for the type.
//! 1. Define a constant for the code default value of the field:
//! 
//!  - Use the naming convention `CODE_DEFAULT_<FIELD_NAME>`, like `CODE_DEFAULT_NEW_AREA`.
//! 1. Ensure that the appropriate field is defined in the following structs for the setting:
//! 
//!   - [`DscPolicyFileData`]
//!   - [`DscPreferenceFileData`]
//! 1. Ensure that the [`DscCodeDefaults`] struct defines the field with the appropriate value type
//!    and update the [`CODE_DEFAULT_SETTINGS`] constant by setting the field to the code
//!    default constant for the field.
//! 1. Ensure that the [`DscSettingsResolved`] struct defines the field as a
//!    [`DscSettingsResolvedField<T>`] with the appropriate value type and update the [`Default`]
//!    implementation for the struct to initialize the field with the code default constant.
//! 1. Every top-level leaf field in DSC settings must be definable as an environment variable.
//!    Follow the guidance in [`environment`] to define the appropriate field in that struct.
//! 1. If the field is definable in the command line arguments, follow the guidance in [`cli`] to
//!    define the command line argument for the field.
//! 
//! # Container fields
//!
//! In the submodule for a container field, follow this guidance to define the types and constants
//! needed to represent the field in DSC settings:
//! 
//! 1. Define a struct to represent the container setting in settings files:
//!
//!    - If the setting is identical in both the preference file and the policy file, define a
//!      single struct for the setting container using the naming convention
//!      `<ContainerFieldName>FileData`, like `NewAreaFileData`.
//!    - If the setting is different between the preference file and the policy file, define
//!      separate structs for each using the following naming conventions:
//!
//!      - `<ContainerFieldName>PreferenceFileData`, like `NewAreaPreferenceFileData`
//!      - `<ContainerFieldName>PolicyFileData`, like `NewAreaPolicyFileData
//!    - When defining both file data structs, ensure that the `*PolicyFileData` struct is _always_
//!      a superset of the `*PreferenceFileData` struct. This ensures that any field defined in the
//!      preference file can also be defined in the policy file. Policy must _always_ be able to
//!      override preferences.
//!    - Define _every_ field in file data structs as an `Option`. No field in a data file must
//!      be required. If a field is not defined in a data file, it will be `None` in the
//!      corresponding struct.
//!    - As needed, define types for leaf fields in the file data structs. Use the naming
//!      convention `<ContainerFieldName><SubFieldName>Field`, like `NewAreaFooField`.
//!    - As needed, define structs for nested container fields in the file data structs. Use the
//!      naming convention `<ContainerFieldName><FieldName>FileData`, like `NewAreaFooFileData`.
//! 1. Define a struct to represent the code defaults for the setting container:
//!
//!    - Use the naming convention `<Container>CodeDefaults`, like `NewAreaCodeDefaults`.
//!    - Define every leaf field in the code defaults struct as the appropriate value type. Don't
//!      define any fields as `Option` in the code defaults struct.
//!    - If you defined any structs for nested container fields in the file data structs, define a
//!      corresponding struct for the code defaults. Use the naming convention
//!     `<ContainerFieldName><FieldName>CodeDefaults`, like `NewAreaFooCodeDefaults`.
//! 1. Define a constant for the code defaults:
//!    - Use the naming convention `CODE_DEFAULT_<CONTAINER_FIELD_NAME>`, like
//!      `CODE_DEFAULT_NEW_AREA`.
//!    - Define the constant with the appropriate values for every field.
//! 1. Define a struct to represent the resolved settings for the setting container:
//! 
//!    - Use the naming convention `<ContainerFieldName>ResolvedSettings`, like
//!      `NewAreaResolvedSettings`.
//!    - Define every leaf field in the resolved settings struct as a
//!      [`DscSettingsResolvedField<T>`] with the appropriate value type.
//!    - If you defined any structs for nested container fields in the file data structs, define a
//!      corresponding struct for the resolved settings. Use the naming convention
//!     `<ContainerFieldName><FieldName>ResolvedSettings`, like `NewAreaFooResolvedSettings`.
//! 1. Ensure that the following traits are implemented for every type defined in this module:
//! 
//!    - Always implement (or derive) [`Clone`], [`Debug`], [`PartialEq`], [`Eq`], [`Serialize`],
//!      [`Deserialize`], and [`JsonSchema`].
//!    - Implement [`Default`] for every `*FileData` and `*ResolvedSettings` struct. You can derive
//!      the implementation for `*FileData` structs, but you must implement it manually for
//!      `*ResolvedSettings` structs. The implementation for `*ResolvedSettings` structs must
//!      initialize every field with the appropriate code default value and a scope of
//!      [`DscSettingsScope::Default`].
//! 1. Ensure that the appropriate field is defined in the following structs for the settings
//!    container:
//! 
//!    - [`DscPolicyFileData`] - define the field with the `*PolicyFileData` struct type or the
//!      `*FileData` struct type.
//!    - [`DscPreferenceFileData`] - define the field with the `*PreferenceFileData` struct type or
//!      the `*FileData` struct type.
//!    - [`DscCodeDefaults`] - define the field with the `*CodeDefaults` struct type.
//!    - [`DscSettingsResolved`] - define the field with the `*ResolvedSettings` struct type.
//! 1. If any settings for the container are definable in the environment, follow the guidance in
//!    [`environment`] to define the appropriate field in that struct.
//! 1. If any settings for the container are definable in the command line arguments, follow the
//!    guidance in [`cli`] to define the appropriate field in that struct.
//! 
//! [`DscPolicyFileData`]: crate::settings::DscPolicyFileData
//! [`DscPreferenceFileData`]: crate::settings::DscPreferenceFileData
//! [`DscCodeDefaults`]: crate::settings::DscCodeDefaults
//! [`DscSettingsResolved`]: crate::settings::DscSettingsResolved
//! [`environment`]: crate::settings::sources::environment
//! [`cli`]: crate::settings::sources::cli

mod forbid_ignore_settings_file;
pub use forbid_ignore_settings_file::*;
mod ignore_settings_file;
pub use ignore_settings_file::*;
mod resource_path;
pub use resource_path::*;
mod tracing;
pub use tracing::*;
