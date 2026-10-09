// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Defines a boolean settings field and optionally the default code constant
/// for the field.
///
/// This macro generates a public struct wrapping an inner [`bool`] that
/// derives the following traits:
///
/// - [`Clone`], [`Copy`], [`Debug`], [`PartialEq`], and [`Eq`] for usability.
/// - [`Serialize`], [`Deserialize`], [`JsonSchema`] and [`DscRepoSchema`] as
///   required for every settings field.
///
/// It inserts the appropriate attribute for [`DscRepoSchema`] to set the base
/// name and folder path. For [`JsonSchema`], it inserts a schemars attribute
/// that ensures:
///
/// 1. The schema includes `$id` and `$schema`.
/// 1. The schema includes the `title`, `description`, and `markdownDescription`
///    docs keywords.
/// 1. To provide better hover help in editors like VS Code, it adds the `enum`
///    keyword with the boolean values and the `markdownEnumDescriptions`
///    keyword to describe how the setting behaves when set to `true` or
///    `false`.
///
/// It also implements the [`AsRef`], [`Deref`], and [`From`] traits for
/// ergonomic usage, since those can't be derived.
///
/// If you specify the code default constant, it will be generated along with
/// the struct.
///
/// <div class="warning">
///
/// To use this macro, you _must_ ensure that [`DscRepoSchema`] is imported in
/// the current module. Due to limitations in declarative macros and
/// referencing the trait by path, this macro **does not** insert the `use`
/// statement for [`DscRepoSchema`] - otherwise, multiple uses of this macro
/// in the same module would raise a compilation error on duplicate `use`
/// statements.
///
/// </div>
///
/// # Syntax
///
/// The macro uses the following syntax:
///
/// ```ignore
/// define_boolean_field! {
///     base_name: "<fieldName>",
///     folder_path: "settings/fields[/<optional_subfolder_path>]",
///     field_definition: {
///         /// Optional documentation for the field and any additional
///         /// attributes.
///         struct <FieldName>Field;
///     },
///     code_default_const: {
///         /// Optional documentation for the code default constant.
///         CODE_DEFAULT_<FIELD_NAME> = <default_value>;
///     },
/// }
/// ```
///
/// Every argument pair for the macro consists of a key and a value, separated
/// by a colon. The following list describes each argument:
///
/// - `base_name`: Defines the base name for the repository schema of the
///   settings field. This value should always be a string representing the
///   camel-cased field name.
///
/// - `folder_path`: Defines the folder path for the settings field in the
///   repository schema. For top-level fields this should always be
///   `"settings/fields"`. For nested fields, include the optional subfolder
///   path with the container field names for each segment in the nested
///   structure.
///
///   For example, consider `ExampleFooBarField`, which is nested under
///   `ExampleFooField`, which itself is nested under `ExampleFileData`. The
///   folder path for `ExampleFooBarField` would be
///   `"settings/fields/example/foo"`.
///
/// - `field_definition`: Defines the field as a unit struct to get the type
///   name for the struct that represents the field. The type name should
///   always be the PascalCase name of the field with the suffix `Field`, like
///   `TopLevelField` for `top_level`.
///
///   Before the unit struct definition, you can add triple slash documentation
///   and additional attributes.
///
/// - `code_default_const`: Defines the code default constant for the field to
///   get the constant name and default value. The name should follow the
///   convention `CODE_DEFAULT_<FIELD_NAME>`. The default value must be `true`
///   or `false`.
///
///   Before the constant definition, you can add triple slash documentation
///   and additional attributes.
///
/// # Examples
///
/// ## Top-level field
///
/// For any invocation you _must_ define the the values for the `base_name` and
/// `folder_path` [`DscRepoSchema`] attribute and the field definition as a
/// unit struct with the type name for the field.
///
/// This defines the field struct and derives/implements the required traits.
///
/// ```ignore
/// define_boolean_field! {
///     base_name: "topLevelExample",
///     folder_path: "settings/fields",
///     field_definition: {
///         /// Documentation explaining how the setting affects DSC.
///         struct TopLevelExample;
///     },
///     code_default_const: {
///         /// Defines the code default for the `top_level_example` field.
///         CODE_DEFAULT_TOP_LEVEL_EXAMPLE = true;
///     }
/// }
/// ```
///
/// ## Nested field
///
/// This example shows how you can define a nested boolean field for a
/// container field. It omits the documentation lines for brevity.
///
/// For this example, the top-level container field is named `top_level`.
///
/// ```ignore
/// define_boolean_field! {
///     base_name: "nestedField",
///     folder_path: "settings/fields/topLevel",
///     field_definition: {
///         struct NestedField;
///     },
///     code_default_const: {
///         CODE_DEFAULT_NESTED_FIELD = true;
///     }
/// }
/// ```
///
/// [`Debug`]: std::fmt::Debug
/// [`Serialize`]: serde::Serialize
/// [`Deserialize`]: serde::Deserialize
/// [`JsonSchema`]: schemars::JsonSchema
/// [`DscRepoSchema`]: crate::schemas::dsc_repo::DscRepoSchema
/// [`AsRef`]: std::convert::AsRef
/// [`Deref`]: std::ops::Deref
/// [`From`]: std::convert::From
macro_rules! define_boolean_field {
    (
        base_name: $base_name:literal,
        folder_path: $folder_path:literal,
        field_definition: {
            $(#[$meta_type:meta])*
            struct $type:ident;
        }$(,
        code_default_const: {
            $(#[$meta_const:meta])*
            $default_const_name:ident = $default_const_value:literal;
        }$(,)?)?
    ) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            PartialEq,
            Eq,
            serde::Serialize,
            serde::Deserialize,
            schemars::JsonSchema,
            DscRepoSchema
        )]
        #[dsc_repo_schema(base_name = $base_name, folder_path = $folder_path)]
        #[schemars(
            title = crate::schemas::dsc_repo::schema_i18n!("title"),
            description = crate::schemas::dsc_repo::schema_i18n!("description"),
            extend(
                "$schema" = Self::default_export_schema_id_uri(),
                "$id" = Self::default_export_schema_id_uri(),
                "markdownDescription" = crate::schemas::dsc_repo::schema_i18n!("markdownDescription"),
                "enum" = [true, false],
                "markdownEnumDescriptions" = [
                    crate::schemas::dsc_repo::schema_i18n!("markdownEnumDescriptions.true"),
                    crate::schemas::dsc_repo::schema_i18n!("markdownEnumDescriptions.false")
                ]
            )
        )]
        $(#[$meta_type])*
        pub struct $type(bool);

        impl $type {
            #[doc = concat!(
                "Creates a new instance of [`",
                stringify!($type),
                "`] with the given boolean value."
            )]
            pub fn new(value: bool) -> Self {
                $type(value)
            }
        }

        // Enable accessing the inner bool value as reference
        impl AsRef<bool> for $type {
            fn as_ref(&self) -> &bool {
                &self.0
            }
        }
        // Enable dereferencing to the inner bool value
        impl std::ops::Deref for $type {
            type Target = bool;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        // Enable writing as string
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.as_ref())
            }
        }
        // Enable conversion to/from bool values
        impl From<bool> for $type {
            fn from(value: bool) -> Self {
                $type(value)
            }
        }
        impl From<$type> for bool {
            fn from(value: $type) -> Self {
                value.0
            }
        }
        // Enable comparing directly with bool values
        impl PartialEq<bool> for $type {
            fn eq(&self, other: &bool) -> bool {
                self.0 == *other
            }
        }
        impl PartialEq<$type> for bool {
            fn eq(&self, other: &$type) -> bool {
                *self == other.0
            }
        }

        $(
            $(#[$meta_const])*
            pub const $default_const_name: $type = $type($default_const_value);
        )?
    };
}

pub(crate) use define_boolean_field;