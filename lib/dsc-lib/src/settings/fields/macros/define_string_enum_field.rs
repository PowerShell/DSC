// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Defines a string enum settings field and optionally the default code
/// constant for the field.
///
/// This macro generates a public enum that derives the following traits:
///
/// - [`Clone`], [`Debug`], [`PartialEq`], and [`Eq`] for usability.
/// - [`Serialize`], [`Deserialize`], [`JsonSchema`] and [`DscRepoSchema`] as
///   required for every settings field.
///
/// The macro implements the following for the public API:
///
/// - `ALL` - constant array containing every variant of the enum.
/// - `parse(text: &str) -> Result<EnumType, DscSettingsError>` - function to
///   parse a string into the enum case insensitively.
///
/// The macro implements the following for the private (crate public) API:
///
/// - `VARIANT_LIST` - constant string containing the display string for all but
///   the last variant wrapped in backticks and separated by commas, like
///   `` "`foo`, `bar`, `baz`" ``.
/// - `VARIANT_LAST` - constant string containing the display string for the
///   last variant wrapped in backticks.
///
/// Those constants are used for surfacing useful error messages to users.
///
/// The macro also implements the following traits for ergonomic usage:
///
/// - [`FromStr`] and [`TryFrom<String>`] for parsing from strings, calling the
///   public API `parse()` function instead of duplicating logic.
/// - [`Display`] and [`From<EnumType> for String`] for converting to strings.
///
/// It inserts the appropriate attribute for [`DscRepoSchema`] to set the base
/// name and folder path. For [`JsonSchema`], it inserts a schemars attribute
/// that ensures:
///
/// 1. The schema includes `$id` and `$schema`.
/// 1. The schema includes the `title`, `description`, and `markdownDescription`
///    docs keywords.
/// 1. To provide better hover help in editors like VS Code, it adds the
///    `markdownEnumDescriptions` keyword to provide per-variant help text.
///
/// For the purposes of serialization and deserialization, the enum is treated
/// as a string and:
///
/// 1. Treats the camelCase variant name as the canonical value for
///    serialization.
/// 1. Parses the string case-insensitively for deserialization.
///
/// If you specify the code default constant, it will be generated along with
/// the struct.
///
/// <div class="warning">
///
/// To use this macro, you _must_ ensure that [`DscRepoSchema`] is imported in
/// the current module. Due to limitations in declarative macros and
/// referencing the trait by path, this macro **does not** insert the `use`
/// statement for [`DscRepoSchema`] - otherwise, multiple uses of this macro in
/// the same module would raise a compilation error on duplicate `use`
/// statements.
///
/// Note that until you define the error variant for [`DscSettingsError`]
/// specified in `parse_error_variant`, the macro will raise a compile error
/// indicating that the variant doesn't exist.
///
/// </div>
///
/// # Syntax
///
/// The macro uses the following syntax:
///
/// ```ignore
/// define_string_enum_field! {
///     base_name: "<fieldName>",
///     folder_path: "settings/fields[/<optional_subfolder_path>]",
///     parse_error_variant: <ParseErrorVariant>,
///     field_definition: {
///         /// Optional documentation for the field and any additional
///         /// attributes
///         enum <FieldName>Field {
///             <Variant1>,
///             <Variant2>,
///             ...
///         }
///     }
/// }
/// ```
///
/// # Examples
///
/// ## Top-level field
///
/// The following snippet shows how you would use the macro to define a string
/// enum field for the top-level setting field `top_level`.
///
/// ```ignore
/// define_string_enum_field! {
///     base_name: "topLevel",
///     folder_path: "settings/fields",
///     parse_error_variant: InvalidTopLevel,
///     field_definition: {
///         enum TopLevelField {
///             Foo { display: "foo", lowercase: "foo" },
///             Bar { display: "bar", lowercase: "bar" },
///             Baz { display: "baz", lowercase: "baz" },
///             VeryLongVariant {
///                 display: "veryLongVariant",
///                 lowercase: "verylongvariant"
///             },
///         }
///     },
///     code_default_const: {
///         CODE_DEFAULT_TOP_LEVEL = Foo;
///     }
/// }
/// ```
///
/// ## Nested field
///
/// The following snippet shows how you would use the macro to define the field
/// `nested` as a string enum nested inside the top-level container field
/// `top_level`.
///
/// ```ignore
/// define_string_enum_field! {
///     base_name: "nested",
///     folder_path: "settings/fields/topLevel",
///     parse_error_variant: InvalidNested,
///     field_definition: {
///         enum NestedField {
///             Foo { display: "foo", lowercase: "foo" },
///             Bar { display: "bar", lowercase: "bar" },
///             Baz { display: "baz", lowercase: "baz" },
///         }
///     }
/// }
/// ```
///
/// [`Debug`]: std::fmt::Debug
/// [`Serialize`]: serde::Serialize
/// [`Deserialize`]: serde::Deserialize
/// [`JsonSchema`]: schemars::JsonSchema
/// [`DscRepoSchema`]: crate::schemas::dsc_repo::DscRepoSchema
/// [`FromStr`]: std::str::FromStr
/// [`TryFrom<String>`]: std::convert::TryFrom
/// [`Display`]: std::fmt::Display
/// [`From<EnumType> for String`]: std::convert::From
/// [`DscSettingsError`]: crate::settings::DscSettingsError
macro_rules! define_string_enum_field {
    (
        base_name: $base_name:literal,
        folder_path:$folder_path:literal,
        parse_error_variant: $parse_error_variant:ident,
        field_definition: {
            $(#[$meta_type:meta])*
            enum $type:ident{
                $(
                    $(#[$meta_variant:meta])*
                    $variant_name:ident {
                        display: $variant_display:literal,
                        lowercase: $variant_lowercase:literal $( , )?
                    },
                )+
            }
        }$(,
        code_default_const: {
            $(#[$meta_const:meta])*
            $default_const_name:ident = $default_const_variant:ident;
        }$(,)?
        )?
    ) => {
        #[derive(
            Clone,
            Debug,
            PartialEq,
            Eq,
            serde::Serialize,
            serde::Deserialize,
            schemars::JsonSchema,
            DscRepoSchema
        )]
        #[serde(rename_all = "camelCase", try_from = "String", into = "String")]
        #[dsc_repo_schema(base_name = $base_name, folder_path = $folder_path)]
        #[schemars(
            !try_from, !into,
            transform = crate::schemas::transforms::idiomaticize_string_enum,
            title = crate::schemas::dsc_repo::schema_i18n!("title"),
            description = crate::schemas::dsc_repo::schema_i18n!("description"),
            extend(
                "$schema" = Self::default_export_meta_schema_uri(),
                "$id" = Self::default_export_schema_id_uri(),
                "markdownDescription" = crate::schemas::dsc_repo::schema_i18n!("markdownDescription")
            )
        )]
        $(#[$meta_type])*
        pub enum $type {
            $(
                $(#[$meta_variant])*
                #[schemars(
                    title = crate::schemas::dsc_repo::schema_i18n!("variants", $variant_display, "title"),
                    description = crate::schemas::dsc_repo::schema_i18n!("variants", $variant_display, "description"),
                    extend(
                        "markdownDescription" = crate::schemas::dsc_repo::schema_i18n!("variants", $variant_display, "markdownDescription")
                    )
                )]
                $variant_name,
            )+
        }

        // Public API
        impl $type {
            #[doc = concat!("An array containing every variant of [`", stringify!($type), "`].")]
            pub const ALL: [$type; $crate::settings::fields::macros::utility::count_idents!($($variant_name),+)] = [
                $(
                    Self::$variant_name,
                )+
            ];

            #[doc = concat!("Parse a string into an instance of [`", stringify!($type), "`].")]
            ///
            /// This function parses the input string case-insensitively. If the downcased input
            /// string matches the downcased display for a variant, the function returns that
            /// variant.
            ///
            /// # Arguments
            ///
            /// - `s` - The string to parse into an instance.
            ///
            /// # Errors
            ///
            /// If the string doesn't match any variant, this function emits the
            #[doc = concat!("[`", stringify!($parse_error_variant), "`] error.")]
            ///
            /// # Returns
            ///
            #[doc = concat!("This function returns an instance of [`", stringify!($type), "`]")]
            /// if the input text matches a variant and otherwise returns a parse error.
            ///
            #[doc = concat!(
                "[`", stringify!($parse_error_variant), "`]: ",
                "crate::settings::DscSettingsError::", stringify!($parse_error_variant),
            )]
            pub fn parse(s: &str) -> Result<Self, crate::settings::DscSettingsError> {
                match s.to_lowercase().as_str() {
                    $(
                        $variant_lowercase => Ok(Self::$variant_name),
                    )+
                    _ => Err(crate::settings::DscSettingsError::$parse_error_variant{text: s.to_string()}),
                }
            }
        }

        // Private API
        impl $type {
            $crate::settings::fields::macros::utility::define_variant_consts!($($variant_display),+);
        }

        impl core::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let format_str = match self {
                    $(
                        Self::$variant_name => $variant_display,
                    )+
                };
                write!(f, "{}", format_str)
            }
        }

        impl std::str::FromStr for $type {
            type Err = crate::settings::DscSettingsError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::parse(s)
            }
        }

        impl TryFrom<String> for $type {
            type Error = crate::settings::DscSettingsError;

            fn try_from(value: String) -> Result<Self, <Self as TryFrom<String>>::Error> {
                Self::parse(&value)
            }
        }

        impl From<$type> for String {
            fn from(value: $type) -> Self {
                value.to_string()
            }
        }

        impl TryFrom<&str> for $type {
            type Error = crate::settings::DscSettingsError;

            fn try_from(value: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
                Self::parse(value)
            }
        }

        $(
            $(#[$meta_const])*
            pub const $default_const_name: $type = $type::$default_const_variant;
        )?
    };
}
pub(crate) use define_string_enum_field;
