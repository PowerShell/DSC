// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Defines a container field with associated types and default code constant.
///
/// This macro provides a compact way to define container fields and the
/// required types to reduce boilerplate code and drifting implementations.
///
/// <div class="note">
///
/// To use this macro, you'll need to add a `use` statement to import the
/// [`DscRepoSchema`] trait. Recommend adding the following snippet to the
/// submodule for the field before using the macro:
///
/// ```rust, ignore
/// use crate::schemas::dsc_repo::DscRepoSchema;
/// use super::macros::*;
/// ```
///
/// </div>
///
/// <div class="warning">
///
/// Currently, the implementation doesn't support defining a container field
/// that has one or more nested container fields. For container fields with
/// nested containers, manually define the field.
///
/// </div>
///
/// This macro generates a public struct that derives the following traits:
///
/// - [`Clone`], [`Debug`], [`PartialEq`], and [`Eq`] for usability.
/// - [`Serialize`], [`Deserialize`], [`JsonSchema`] and [`DscRepoSchema`] as
///   required for every settings field.
///
/// When it generates the struct, it also:
///
/// 1. Applies the `#[serde(rename_all = "camelCase")]` attribute to the
///    struct, ensuring the container's fields use the correct case.
/// 1. Applies the `#[dsc_repo_schema]` attribute to pass through the base name
///    and folder path.
/// 1. Automatically defines the `schemars` attribute on the struct to set the
///    `title`, `description`, and `markdownDescription` keywords to the
///    localized strings. It also sets the `$schema` and `$id` keywords to the
///    default export URIs from the implementation of [`DscRepoSchema`].
/// 1. Adds every specified field from the `field_definition` argument:
///
///    - With the given field name and the type [`Option<T>`], where `T` is the
///      type specified in the `field_definition` argument.
///    - With the `schemars` attribute to set the `title`, `description`, and
///      `markdownDescription` keywords to the localized strings. It expects
///      the translation file to define entries for each field under the
///      `fields` key for the struct.
///    - With the `#[serde(skip_serializing_if = "Option::is_none")]` attribute
///      to skip serializing the field when undefined.
///
/// It also generates the required `*ResolvedSettings` and `*CodeDefaults`
/// structs as well as the `CODE_DEFAULT_*` constant. It generates the required
/// trait implementations for the additional types.
///
/// When the macro generates the struct for the code defaults, it:
///
/// 1. Derives [`Clone`], [`Debug`], [`PartialEq`], [`Eq`], [`Serialize`], and
///    [`Deserialize`].
/// 1. Applies the `#[serde(rename_all = "camelCase")]` attribute to ensure
///    consistent casing.
/// 1. Defines every specified field from the `field_definition` argument with
///    the same name and the specified type.
/// 1. Implements the [`Default`] trait to return the code default constant.
///
/// When the macro generates the code default constant, it defines the constant
/// with the specified name and uses the default values from the
/// `field_definition` argument to populate each field.
///
/// When the macro generates the resolved settings struct, it:
///
/// 1. Derives [`Clone`], [`Debug`], [`PartialEq`], [`Eq`], [`Serialize`],
///    [`Deserialize`], and [`JsonSchema`].
/// 1. Applies the `#[serde(rename_all = "camelCase")]` attribute to ensure
///    consistent casing.
/// 1. Defines every specified field from the `field_definition` argument with
///    the same name and and the [`DscSettingsResolvedField<T>`] type, where
///    `T` is the type specified in the `field_definition` argument.
/// 1. Implements the [`Default`] trait to return the resolved settings with all
///    fields set to their default values and the scope set to code defaults.
///
/// # Syntax
///
/// The following snippet shows the high-level syntax for the macro:
///
/// ```rust, ignore
/// define_container_field! {
///     base_name: <camelCaseFieldName>,
///     folder_path: <schema_file_folder_path>,
///     field_definition: {
///         /// Optional docs
///         #[optional_attribute]
///         struct <PascalCaseFieldName>FileData {
///             /// Optional docs
///             #[optional_attribute]
///             <nested_field_name>: <NestedFieldType> = <NestedFieldDefaultValue>,
///             ...
///         }
///     },
///     resolved_container_type: {
///         /// Optional docs
///         #[optional_attribute]
///         <PascalCaseFieldName>ResolvedSettings
///     },
///     code_default_type: {
///         /// Optional docs
///         #[optional_attribute]
///         <PascalCaseFieldName>CodeDefaults
///     },
///     code_default_const: {
///         /// Optional docs
///         #[optional_attribute]
///         CODE_DEFAULT<SCREAMING_SNAKE_CASE_FIELD_NAME>
///     },
/// }
/// ```
///
/// All key-value pairs are required.
///
/// ## `base_name`
///
/// Define this a a string literal containing the camel-cased name of the new
/// container field. For example, `my_new_field` would be defined as
/// `"myNewField"`.
///
/// ## `folder_path`
///
/// Define this as a string literal containing the folder path where the schema
/// file for the new container field will be located.
///
/// If the new container field is a top-level field this value should be
/// `"settings/fields"`.
///
/// If it's a nested field, this value should include the
/// ancestors in the folder path. For example:
///
/// - If the new container field is nested under `parent_field`, the folder
///   path would be be`"settings/fields/parentField"`.
/// - If the new container field is nested under `parent_field`, which is under
///   `grand_parent_field`, the folder path would be
///   `"settings/fields/parentField/childField"`.
///
/// ## `field_definition`
///
/// Inside the curly braces, specify the `struct` keyword followed by the
/// pascal-case name of the new container field with the suffix `FileData`. For
/// example, `my_new_field` would be defined as `struct MyNewFieldFileData`.
///
/// Immediately preceding the struct definition, you can optionally include
/// documentation comments and any attributes. In particular, the `serde` and
/// `schemars` attributes are supported, but so are any other generic
/// attributes.
///
/// After the struct keyword and type name, include curly braces to contain the
/// nested field definitions for the new container.
///
/// For each field:
///
/// 1. Specify the nested field name in snake_case, followed by a colon and the
///    field type.
/// 1. Assign a default value to the field using the equals sign. The default
///    value _must_ be constant function compatible.
/// 1. Optionally include documentation comments and attributes immediately
///    preceding the field definition.
///
/// ## `resolved_container_type`
///
/// Inside the curly braces, specify the resolved container type name as the
/// pascal cased field name with the suffix `ResolvedSettings`. You can
/// optionally include documentation comments and attributes immediately
/// preceding the resolved container type name.
///
/// ## `code_default_type`
///
/// Inside the curly braces, specify the code default type as the pascal cased
/// field name with the suffix `CodeDefaults`. You can optionally include
/// documentation comments and attributes immediately preceding the code
/// default type name.
///
/// ## `code_default_const`
///
/// Inside the curly braces, specify the code default constant name as the
/// screaming snake case version of the field name with the prefix
/// `CODE_DEFAULT_`. You can optionally include documentation comments and
/// attributes immediately preceding the code default constant name.
///
/// # Examples
///
/// ## Minimal
///
/// The following snippet shows how you can use the macro to define a new
/// top-level container field. It doesn't define any optional documentation
/// or attributes.
///
/// ```rust, ignore
/// define_container_field! {
///     base_name: "newArea",
///     folder_path: "settings/fields",
///     field_definition: {
///         struct NewAreaFileData {
///             foo: bool = false,
///             bar: u32  = 0,
///             baz: String = String::from("Example"),
///         }
///     },
///     resolved_container_type: {
///         NewAreaResolvedSettings
///     },
///     code_default_type: {
///         NewAreaCodeDefaults
///     },
///     code_default_const: {
///         CODE_DEFAULT_NEW_AREA
///     },
/// }
/// ```
///
/// ## With documentation and attributes
///
/// The following snippet shows how you can use the macro to define a new
/// nested container field. It includes documentation for the types and fields,
/// as well as additional attributes.
///
/// ```rust, ignore
/// define_container_field! {
///     base_name: "nestedContainer",
///     folder_path: "settings/fields/parentContainer",
///     field_definition: {
///         /// Defines settings for `parent_container.nested_container`.
///         struct ParentContainerNestedContainerFileData {
///             /// Indicates whether to enable the functionality.
///             enabled: bool = false,
///             /// Indicates the groups to process.
///             #[schemars(extend(
///                 "items" = {
///                     "type" = "string",
///                     "pattern" = "^[a-zA-Z0-9_-]+$"
///                 }
///             ))]
///             groups: Vec<String>  = Vec::new(),
///         }
///     },
///     resolved_container_type: {
///         /// Resolved settings for `parent_container.nested_container`.
///         ParentContainerNestedContainerResolvedSettings
///     },
///     code_default_type: {
///         /// Code default structure for `parent_container.nested_container`.
///         ParentContainerNestedContainerCodeDefaults
///     },
///     code_default_const: {
///         /// Defines the default values for `parent_container.nested_container`.
///         CODE_DEFAULT_PARENT_CONTAINER_NESTED_CONTAINER
///     },
/// }
/// ```
///
/// [`Debug`]: std::fmt::Debug
/// [`Serialize`]: serde::Serialize
/// [`Deserialize`]: serde::Deserialize
/// [`JsonSchema`]: schemars::JsonSchema
/// [`DscRepoSchema`]: crate::schemas::dsc_repo::DscRepoSchema
/// [`DscSettingsResolvedField<T>`]: crate::settings::DscSettingsResolvedField
macro_rules! define_container_field {
    (
        base_name: $base_name:tt,
        folder_path: $folder_path:tt,
        field_definition: {
            $(#[$meta_type:meta])*
            struct $type:ident {
                $(
                    $(#[$meta_field:meta])*
                    $field_name:ident: $field_type:ty = $field_default:expr,
                )*
                $(,)?
            }
        },
        resolved_container_type: {
            $(#[$meta_resolved:meta])*
            $resolved_type:ident
        },
        code_default_type: {
            $(#[$meta_default_const:meta])*
            $code_default_type:ident
        },
        code_default_const: {
            $(#[$meta_const:meta])*
            $default_const_name:ident
        }
        $(,)?
    ) => {
        #[derive(
            Clone, Debug, Default, PartialEq, Eq,
            serde::Serialize, serde::Deserialize,
            schemars::JsonSchema, DscRepoSchema
        )]
        #[serde(rename_all = "camelCase")]
        #[dsc_repo_schema(base_name = $base_name, folder_path = $folder_path)]
        #[schemars(
            title = crate::schemas::dsc_repo::schema_i18n!("title"),
            description = crate::schemas::dsc_repo::schema_i18n!("description"),
            extend(
                "$schema" = Self::default_export_meta_schema_uri(),
                "$id" = Self::default_export_schema_id_uri(),
                "markdownDescription" = crate::schemas::dsc_repo::schema_i18n!("markdownDescription"),
            )
        )]
        $(#[$meta_type])*
        pub struct $type {
            $(
                #[schemars(
                    title = crate::schemas::dsc_repo::schema_i18n!(
                        &format!("fields.{}.title", stringify!($field_name))
                    ),
                    description = crate::schemas::dsc_repo::schema_i18n!(
                        &format!("fields.{}.description", stringify!($field_name))
                    ),
                    extend(
                        "markdownDescription" = crate::schemas::dsc_repo::schema_i18n!(
                            &format!("fields.{}.markdownDescription", stringify!($field_name))
                        ),
                    )
                )]
                #[serde(skip_serializing_if = "Option::is_none")]
                $(#[$meta_field])*
                pub $field_name: Option<$field_type>,
            )*
        }

        $(#[$meta_default_const])*
        #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct $code_default_type {
            $(
                pub $field_name: $field_type,
            )*
        }

        impl Default for $code_default_type {
            fn default() -> Self {
                $default_const_name
            }
        }

        $(#[$meta_const])*
        pub const $default_const_name: $code_default_type = $code_default_type {
            $(
                $field_name: $field_default,
            )*
        };

        $(#[$meta_resolved])*
        #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
        #[serde(rename_all = "camelCase")]
        #[schemars(inline)]
        pub struct $resolved_type {
            $(
                pub $field_name: crate::settings::DscSettingsResolvedField<$field_type>,
            )*
        }

        impl Default for $resolved_type {
            fn default() -> Self {
                let scope = crate::settings::DscSettingsScope::Default;
                Self {
                    $(
                        $field_name: crate::settings::DscSettingsResolvedField::new(
                            $default_const_name.$field_name.clone(),
                            scope
                        ),
                    )*
                }
            }
        }
    };
}

pub(crate) use define_container_field;
