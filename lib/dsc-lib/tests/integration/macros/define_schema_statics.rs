// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Defines static variables for JSON schemas to use in tests.
///
/// This macro always generates the `ROOT_SCHEMA` and `VALIDATOR` static
/// variables, which are used by the other helper macros to validate the JSON
/// schema for a defined type.
///
/// <div class="info">
///
/// This macro is best used through the [`test_dsc_repo_schema!`] macro. Only
/// use this macro directly when the higher level macro is unsuitable.
///
/// </div>
///
/// ## Syntax
///
/// The first argument to this macro must be one of the following:
///
/// - A type path, e.g., `my_crate::MyType`
/// - `dsc_repo_schema_for: <type path>` - generate the root schema with the
///   [`DscRepoSchema`] trait's `generate_exportable_schema` method as the
///   latest recognized schema version and the VS Code schema form to retain
///   the VS Code schema documentation keywords.
///
///   For the validator, it generates the schema in the bundled form, which
///   strips the VS Code keywords and canonicalizes the schema while ensuring
///   that references to the bundled schema resources are usable for validation.
/// - `json_schema_for: <type path>` - generate the root schema with the
///   [`schemars::schema_for!`] macro. This should be used rarely since most
///   types should implement the [`DscRepoSchema`] trait.
///
/// The macro supports the following arguments after the first argument, which
/// must always be included to define the `ROOT_SCHEMA` and `VALIDATOR` statics:
///
/// - `properties: { <STATIC_NAME>: "<propertyName>", ... }`
///
///   Use the `properties` argument to define one or more additional statics
///   for the schema properties, where each static corresponds to a property
///   name in the JSON schema.
///
///   This is primarily useful for ensuring that the property subschema has the
///   expected documentation keywords.
///
/// # Examples
///
/// ## With type only
///
/// When you use this macro with a single type as the only argument, it will
/// generate the `ROOT_SCHEMA` and `VALIDATOR` static variables for that type
/// as a DSC repository schema.
///
/// ```rust, ignore
/// define_schema_statics!(my_crate::MyType);
/// ```
///
/// <details><summary><Expanded macro usage</summary>
///
/// ```rust, ignore
/// pub(super) static ROOT_SCHEMA: std::sync::LazyLock<schemars::Schema> =
///     std::sync::LazyLock::new(|| {
///         use dsc_lib::schemas::dsc_repo::DscRepoSchema;
///         <dsc_lib::settings::sources::PreferenceFileData>::generate_exportable_schema(
///             dsc_lib::schemas::dsc_repo::RecognizedSchemaVersion::latest(),
///             dsc_lib::schemas::dsc_repo::SchemaForm::VSCode,
///         )
///     });
/// pub(super) static VALIDATOR: std::sync::LazyLock<jsonschema::Validator> =
///     std::sync::LazyLock::new(|| {
///         use dsc_lib::schemas::dsc_repo::DscRepoSchema;
///         let ref schema =
///             <dsc_lib::settings::sources::PreferenceFileData>::generate_exportable_schema(
///                 dsc_lib::schemas::dsc_repo::RecognizedSchemaVersion::latest(),
///                 dsc_lib::schemas::dsc_repo::SchemaForm::Bundled,
///             );
///         match jsonschema::Validator::new(schema.as_value()) {
///             Ok(validator) => validator,
///             Err(err) => panic!(
///                 "Failed to create JSON schema validator:\n---\nERROR: {:#?}\n---\nSCHEMA: {}\n---",
///                 err,
///                 serde_json::to_string_pretty(schema).unwrap(),
///             )
///         }
///     });
/// ```
///
/// </details>
///
/// ## With type as DSC repository schema
///
/// When you use this macro with the `dsc_repo_schema_for` argument, it will
/// generate the `ROOT_SCHEMA` and `VALIDATOR` static variables for that type
/// as a DSC repository schema.
///
/// ```rust, ignore
/// define_schema_statics!{dsc_repo_schema_for: my_crate::MyType}
/// ```
///
/// <details><summary><Expanded macro usage</summary>
///
/// ```rust, ignore
/// pub(super) static ROOT_SCHEMA: std::sync::LazyLock<schemars::Schema> =
///     std::sync::LazyLock::new(|| {
///         use dsc_lib::schemas::dsc_repo::DscRepoSchema;
///         <dsc_lib::settings::sources::PreferenceFileData>::generate_exportable_schema(
///             dsc_lib::schemas::dsc_repo::RecognizedSchemaVersion::latest(),
///             dsc_lib::schemas::dsc_repo::SchemaForm::VSCode,
///         )
///     });
/// pub(super) static VALIDATOR: std::sync::LazyLock<jsonschema::Validator> =
///     std::sync::LazyLock::new(|| {
///         use dsc_lib::schemas::dsc_repo::DscRepoSchema;
///         let ref schema =
///             <dsc_lib::settings::sources::PreferenceFileData>::generate_exportable_schema(
///                 dsc_lib::schemas::dsc_repo::RecognizedSchemaVersion::latest(),
///                 dsc_lib::schemas::dsc_repo::SchemaForm::Bundled,
///             );
///         match jsonschema::Validator::new(schema.as_value()) {
///             Ok(validator) => validator,
///             Err(err) => panic!(
///                 "Failed to create JSON schema validator:\n---\nERROR: {:#?}\n---\nSCHEMA: {}\n---",
///                 err,
///                 serde_json::to_string_pretty(schema).unwrap(),
///             )
///         }
///     });
/// ```
///
/// </details>
///
/// ## With type as general JSON schema
///
/// When you use this macro with the `json_schema_for` argument, it will
/// generate the `ROOT_SCHEMA` and `VALIDATOR` static variables for that type
/// as a general JSON schema, using [`schemars::schema_for!`].
///
/// ```rust, ignore
/// define_schema_statics!(json_schema_for: my_crate::MyType);
/// ```
///
/// <details><summary><Expanded macro usage</summary>
///
/// ```rust, ignore
/// static ROOT_SCHEMA: std::sync::LazyLock<schemars::Schema> =
///     std::sync::LazyLock::new(|| {
///         schemars::schema_for!(my_crate::MyType)
///     });
/// static VALIDATOR: std::sync::LazyLock<jsonschema::Validator> =
///     std::sync::LazyLock::new(|| {
///         jsonschema::Validator::new((&*ROOT_SCHEMA).as_value()).unwrap()
///     });
/// ```
///
/// </details>
///
/// ## With additional property schemas
///
/// When you use this macro with the `properties` argument, it will
/// generate static variables for each specified property schema.
///
/// ```rust, ignore
/// define_schema_statics!{
///     my_crate::MyType,
///     properties: {
///         MY_PROPERTY_SCHEMA => "myProperty",
///         OTHER_PROPERTY_SCHEMA => "otherProperty",
///     },
/// }
/// ```
///
/// <details><summary><Expanded macro usage</summary>
///
/// ```rust, ignore
/// // ROOT_SCHEMA and VALIDATOR are elided for brevity
///
/// static MY_PROPERTY_SCHEMA: std::sync::LazyLock<schemars::Schema> = std::sync::LazyLock::new(|| {
///     std::sync::LazyLock::new(|| {
///         use dsc_lib::schemas::schema_utility_extensions::SchemaUtilityExtensions;
///         ROOT_SCHEMA
///             .clone()
///             .get_property_subschema("myProperty")
///             .expect(&format!(
///                 "properties.{} should be defined",
///                 "myProperty"
///             ))
///             .clone()
///     });
/// static OTHER_PROPERTY_SCHEMA: std::sync::LazyLock<schemars::Schema> = std::sync::LazyLock::new(|| {
///     std::sync::LazyLock::new(|| {
///         use dsc_lib::schemas::schema_utility_extensions::SchemaUtilityExtensions;
///         ROOT_SCHEMA
///             .clone()
///             .get_property_subschema("otherProperty")
///             .expect(&format!(
///                 "properties.{} should be defined",
///                 "otherProperty"
///             ))
///             .clone()
///     });
/// ```
///
/// </details>
///
/// [`test_dsc_repo_schema!`]: crate::macros::test_dsc_repo_schema
/// [`DscRepoSchema`]: dsc_lib::schemas::dsc_repo::DscRepoSchema
macro_rules! define_schema_statics {
    ($typ:ty $(, $($rest:tt)*)?) => {
        $crate::macros::define_schema_statics!(
            dsc_repo_schema_for: $typ
            $(, $($rest)*)?
        );
    };
    (json_schema_for: $type:ty $(, $($rest:tt)*)?) => {
        static ROOT_SCHEMA: std::sync::LazyLock<schemars::Schema> = std::sync::LazyLock::new(|| {
            schemars::schema_for!($type)
        });
        static VALIDATOR: std::sync::LazyLock<jsonschema::Validator> = std::sync::LazyLock::new(|| {
            jsonschema::Validator::new((&*ROOT_SCHEMA).as_value()).unwrap()
        });

        $($crate::macros::define_schema_statics!($($rest)*);)?
    };
    (dsc_repo_schema_for: $type:ty $(, $($rest:tt)*)?) => {
        static ROOT_SCHEMA: std::sync::LazyLock<schemars::Schema> = std::sync::LazyLock::new(|| {
            use dsc_lib::schemas::dsc_repo::DscRepoSchema;
            <$type>::generate_exportable_schema(
                dsc_lib::schemas::dsc_repo::RecognizedSchemaVersion::latest(),
                dsc_lib::schemas::dsc_repo::SchemaForm::VSCode
            )
        });

        static VALIDATOR: std::sync::LazyLock<jsonschema::Validator> = std::sync::LazyLock::new(|| {
            use dsc_lib::schemas::dsc_repo::DscRepoSchema;
            let ref schema = <$type>::generate_exportable_schema(
                dsc_lib::schemas::dsc_repo::RecognizedSchemaVersion::latest(),
                dsc_lib::schemas::dsc_repo::SchemaForm::Bundled
            );
            match jsonschema::Validator::new(schema.as_value()) {
                Ok(validator) => validator,
                Err(err) => panic!(
                    "Failed to create JSON schema validator:\n---\nERROR: {:#?}\n---\nSCHEMA: {}\n---",
                    err,
                    serde_json::to_string_pretty(schema).unwrap(),
                ),
            }
        });

        $($crate::macros::define_schema_statics!($($rest)*);)?
    };
    (properties: {
        $($static_ident:ident => $property_name:literal),+ $(,)?
    } $(, $($rest:tt)*)?) => {
        $(
            static $static_ident: std::sync::LazyLock<schemars::Schema> = std::sync::LazyLock::new(|| {
                use dsc_lib::schemas::schema_utility_extensions::SchemaUtilityExtensions;
                ROOT_SCHEMA
                    .clone()
                    .get_property_subschema($property_name)
                    .expect(&format!("properties.{} should be defined", $property_name))
                    .clone()
            });
        )+
        $( $crate::macros::define_schema_statics!($($rest)*);)?
    };
}

pub(crate) use define_schema_statics;