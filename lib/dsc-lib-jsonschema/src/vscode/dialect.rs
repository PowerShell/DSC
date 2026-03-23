// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::sync::{Arc, LazyLock};

use jsonschema::Resource;
use rust_i18n::t;
use schemars::{JsonSchema, Schema, SchemaGenerator, generate::SchemaSettings, json_schema};

use crate::vscode::{
    keywords::{
        AllowCommentsKeyword,
        AllowTrailingCommasKeyword,
        CompletionDetailKeyword,
        DefaultSnippetsKeyword,
        DeprecationMessageKeyword,
        DoNotSuggestKeyword,
        EnumDescriptionsKeyword,
        EnumDetailsKeyword,
        EnumSortTextsKeyword,
        ErrorMessageKeyword,
        MarkdownDescriptionKeyword,
        MarkdownEnumDescriptionsKeyword,
        PatternErrorMessageKeyword,
        SuggestSortTextKeyword,
        VSCodeKeywordDefinition
    },
    vocabulary::VSCodeVocabulary
};

/// Defines the meta schema for defining schemas with the VS Code vocabulary.
///
/// This meta schema is based on the JSON Schema draft 2020-12. It includes an extended set of
/// annotation keywords that VS Code recognizes and uses to enhance the JSON authoring and editing
/// experience. The vocabulary itself is defined in [`VSCodeVocabulary`].
///
/// While you don't _need_ to use this meta schema to define the annotation keywords VS Code
/// recognizes, specifying this meta schema for your documents does enable you to validate that
/// the keywords are correctly defined in your own schemas.
///
/// The meta schema struct defines associated constants and helper methods:
///
/// - [`SCHEMA_ID`] defines the canonical URI to the meta schema specified in the schema's `$id`
///   keyword.
/// - [`json_schema_bundled()`] retrieves the bundled form of the meta schema with a custom
///   [`SchemaGenerator`].
/// - [`json_schema_canonical()`] retrieves the canonical form of the meta schema with a custom
///   [`SchemaGenerator`].
/// - [`schema_resource_bundled()`] retrieves the bundled form of the meta schema with a custom
///   [`SchemaGenerator`] as a [`Resource`].
/// - [`schema_resource_canonical()`] retrieves the canonical form of the meta schema with a custom
///   [`SchemaGenerator`] as a [`Resource`].
///
/// For easier access to the schemas, consider using the following statics if you don't need to use
/// a custom generator:
///
/// - [`VSCODE_DIALECT_SCHEMA_BUNDLED`] contains the bundled form of the meta schema with the
///   schema resources for the vocabulary and keywords included in the `$defs` keyword.
/// - [`VSCODE_DIALECT_SCHEMA_CANONICAL`] contains the canonical form of the meta schema without
///   the bundled schema resources for a smaller definition.
/// - [`VSCODE_DIALECT_SCHEMA_RESOURCE_BUNDLED`] contains the bundled form of the meta schema as
///   a [`Resource`] to simplify registering the resource with a [`jsonschema::Validator`].
/// - [`VSCODE_DIALECT_SCHEMA_RESOURCE_CANONICAL`] contains the canonical form of the meta schema
///   as a [`Resource`].
///
/// [`SCHEMA_ID`]: Self::SCHEMA_ID
/// [`json_schema_bundled()`]: Self::json_schema_bundled
/// [`json_schema_canonical()`]: Self::json_schema_canonical
/// [`schema_resource_bundled()`]: Self::schema_resource_bundled
/// [`schema_resource_canonical()`]: Self::schema_resource_canonical
pub struct VSCodeDialect;

impl VSCodeDialect {
    /// Defines the canonical URI for the meta schema's `$id` keyword.
    pub const SCHEMA_ID: &str = "https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/vscode/v0/meta.json";

    /// Retrieves the bundled form of the meta schema.
    ///
    /// The bundled form presents the meta schema as a compound schema document with the VS Code
    /// vocabulary and keyword schemas included under the `$defs` keyword. Use this form of the
    /// schema when you need every schema resource included in a single document.
    ///
    /// This function requires you to pass a [`SchemaGenerator`] to retrieve the schema. The
    /// definition for the meta schema is static, but you can use custom transforms with your
    /// [`SchemaGenerator`] to modify the schema if needed. If you want to use the default
    /// representation of the bundled meta schema, use [`VSCODE_DIALECT_SCHEMA_BUNDLED`].
    ///
    /// You can also use the [`json_schema_canonical()`] method to retrieve the canonical form of
    /// the meta schema without the bundled schema resources or [`VSCODE_DIALECT_SCHEMA_CANONICAL`]
    /// to use the default representation of the canonical meta schema.
    ///
    /// [`json_schema_canonical()`]: Self::json_schema_canonical
    pub fn json_schema_bundled(generator: &mut schemars::SchemaGenerator) -> Schema {
        Self::json_schema(generator)
    }

    /// Retrieves the canonical form of the meta schema.
    ///
    /// The canonical form presents the meta schema without bundling the VS Code vocabulary or
    /// keyword schemas under the `$defs` keyword. Use this form of the schema when you can rely
    /// on retrieving the other schemas from network or other methods.
    ///
    /// This function requires you to pass a [`SchemaGenerator`] to retrieve the schema. The
    /// definition for the meta schema is static, but you can use custom transforms with your
    /// [`SchemaGenerator`] to modify the schema if needed. If you want to use the default
    /// representation of the canonical meta schema, use [`VSCODE_DIALECT_SCHEMA_CANONICAL`].
    ///
    /// You can also use the [`json_schema_bundled()`] method to retrieve the bundled
    /// form of the meta schema with the schema resources bundled under the `$defs` keyword or
    /// [`VSCODE_DIALECT_SCHEMA_BUNDLED`] to use the default representation of the canonical
    /// meta schema.
    ///
    /// [`json_schema_bundled()`]: Self::json_schema_bundled
    pub fn json_schema_canonical(generator: &mut schemars::SchemaGenerator) -> Schema {
        let mut schema = Self::json_schema(generator);
        schema.remove("$defs");
        schema
    }

    /// Retrieves the bundled form of the meta schema as a [`Resource`] so you can include
    /// it in the registered resources for a [`jsonschema::Validator`] using the [`with_resource()`]
    /// or [`with_resources()`] methods on the [`jsonschema::ValidationOptions`] builder.
    ///
    /// The bundled form presents the meta schema as a compound schema document with the VS Code
    /// vocabulary and keyword schemas included under the `$defs` keyword. Use this form of the
    /// schema when you need every schema resource included in a single document.
    ///
    /// This function requires you to pass a [`SchemaGenerator`] to retrieve the schema. The
    /// definition for the meta schema is static, but you can use custom transforms with your
    /// [`SchemaGenerator`] to modify the schema if needed. If you want to use the default
    /// representation of the bundled meta schema, use [`VSCODE_DIALECT_SCHEMA_RESOURCE_BUNDLED`].
    ///
    /// You can also use the [`schema_resource_canonical()`] method to retrieve the canonical
    /// form of the meta schema without the bundled schema resources or
    /// [`VSCODE_DIALECT_SCHEMA_RESOURCE_CANONICAL`] to use the default representation of the
    /// canonical meta schema.
    ///
    /// # Panics
    ///
    /// This method panics if the schema is malformed and can't be converted into a [`Resource`].
    ///
    /// In practice, you should never see a panic from this method because the crate's test suite
    /// checks for this failure mode.
    ///
    /// [`schema_resource_canonical()`]: Self::schema_resource_canonical
    /// [`with_resource()`]: jsonschema::ValidationOptions::with_resource
    /// [`with_resources()`]: jsonschema::ValidationOptions::with_resources
    pub fn schema_resource_bundled(generator: &mut schemars::SchemaGenerator) -> Resource {
        Resource::from_contents(Self::json_schema(generator).to_value())
    }

    /// Retrieves the bundled form of the meta schema as a [`Resource`] so you can include
    /// it in the registered resources for a [`jsonschema::Validator`] using the [`with_resource()`]
    /// or [`with_resources()`] methods on the [`jsonschema::ValidationOptions`] builder.
    ///
    /// The canonical form presents the meta schema without bundling the VS Code vocabulary or
    /// keyword schemas under the `$defs` keyword. Use this form of the schema when you can rely
    /// on retrieving the other schemas from network or other methods.
    ///
    /// This function requires you to pass a [`SchemaGenerator`] to retrieve the schema. The
    /// definition for the meta schema is static, but you can use custom transforms with your
    /// [`SchemaGenerator`] to modify the schema if needed. If you want to use the default
    /// representation of the canonical meta schema, use
    /// [`VSCODE_DIALECT_SCHEMA_RESOURCE_CANONICAL`].
    ///
    /// You can also use the [`schema_resource_bundled()`] method to retrieve the bundled form of
    /// the meta schema without the bundled schema resources or
    /// [`VSCODE_DIALECT_SCHEMA_RESOURCE_BUNDLED`] to use the default representation of the bundled
    /// meta schema.
    ///
    /// # Panics
    ///
    /// This method panics if the schema is malformed and can't be converted into a [`Resource`].
    ///
    /// In practice, you should never see a panic from this method because the crate's test suite
    /// checks for this failure mode.
    ///
    /// [`schema_resource_bundled()`]: Self::schema_resource_bundled
    /// [`with_resource()`]: jsonschema::ValidationOptions::with_resource
    /// [`with_resources()`]: jsonschema::ValidationOptions::with_resources
    pub fn schema_resource_canonical(generator: &mut schemars::SchemaGenerator) -> Resource {
        Resource::from_contents(Self::json_schema_canonical(generator).to_value())
    }
}

impl JsonSchema for VSCodeDialect {
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": Self::SCHEMA_ID,
            "$vocabulary": {
                "https://json-schema.org/draft/2020-12/vocab/core": true,
                "https://json-schema.org/draft/2020-12/vocab/applicator": true,
                "https://json-schema.org/draft/2020-12/vocab/unevaluated": true,
                "https://json-schema.org/draft/2020-12/vocab/validation": true,
                "https://json-schema.org/draft/2020-12/vocab/meta-data": true,
                "https://json-schema.org/draft/2020-12/vocab/format-annotation": true,
                "https://json-schema.org/draft/2020-12/vocab/content": true,
                VSCodeVocabulary::SPEC_URI: false,
            },
            "title": t!("vscode.dialect.title"),
            "description": t!("vscode.dialect.description"),
            "markdownDescription": t!("vscode.dialect.markdownDescription"),

            "$dynamicAnchor": "meta",
            "allOf": [
                { "$ref": "https://json-schema.org/draft/2020-12/schema" },
                { "$ref": VSCodeVocabulary::SCHEMA_ID }
            ],
            "type": ["object", "boolean"],
            "$defs": {
                VSCodeVocabulary::SCHEMA_ID: VSCodeVocabulary::json_schema_canonical(generator),
                AllowCommentsKeyword::KEYWORD_ID: AllowCommentsKeyword::json_schema(generator),
                AllowTrailingCommasKeyword::KEYWORD_ID: AllowTrailingCommasKeyword::json_schema(generator),
                CompletionDetailKeyword::KEYWORD_ID: CompletionDetailKeyword::json_schema(generator),
                DefaultSnippetsKeyword::KEYWORD_ID: DefaultSnippetsKeyword::json_schema(generator),
                DeprecationMessageKeyword::KEYWORD_ID: DeprecationMessageKeyword::json_schema(generator),
                DoNotSuggestKeyword::KEYWORD_ID: DoNotSuggestKeyword::json_schema(generator),
                EnumDescriptionsKeyword::KEYWORD_ID: EnumDescriptionsKeyword::json_schema(generator),
                EnumDetailsKeyword::KEYWORD_ID: EnumDetailsKeyword::json_schema(generator),
                EnumSortTextsKeyword::KEYWORD_ID: EnumSortTextsKeyword::json_schema(generator),
                ErrorMessageKeyword::KEYWORD_ID: ErrorMessageKeyword::json_schema(generator),
                MarkdownDescriptionKeyword::KEYWORD_ID: MarkdownDescriptionKeyword::json_schema(generator),
                MarkdownEnumDescriptionsKeyword::KEYWORD_ID: MarkdownEnumDescriptionsKeyword::json_schema(generator),
                PatternErrorMessageKeyword::KEYWORD_ID: PatternErrorMessageKeyword::json_schema(generator),
                SuggestSortTextKeyword::KEYWORD_ID: SuggestSortTextKeyword::json_schema(generator),
            }
        })
    }
    fn schema_name() -> std::borrow::Cow<'static, str> {
        Self::SCHEMA_ID.into()
    }
}

/// Contains the bundled form of the VS Code meta schema.
///
/// The bundled form presents the meta schema as a compound schema document with the VS Code
/// vocabulary and keyword schemas included under the `$defs` keyword. Use this form of the
/// schema when you need every schema resource included in a single document.
///
/// You can also use [`VSCODE_DIALECT_SCHEMA_CANONICAL`] to retrieve the canonical form of the meta 
/// schema without the bundled schema resources.
///
/// This representation of the schema is generated with the default [`SchemaSettings`] for
/// JSON Schema draft 2020-12. To retrieve the bundled schema with custom generator settings,
/// use the [`json_schema_bundled()`] method.
///
/// [`json_schema_bundled()`]: VSCodeDialect::json_schema_bundled
pub static VSCODE_DIALECT_SCHEMA_BUNDLED: LazyLock<Arc<Schema>> = LazyLock::new(|| {
    let generator = &mut SchemaGenerator::new(
        SchemaSettings::draft2020_12()
    );

    Arc::from(VSCodeDialect::json_schema_bundled(generator))
});

/// Contains the canonical form of the VS Code meta schema.
///
/// The canonical form presents the meta schema without bundling the VS Code vocabulary or
/// keyword schemas under the `$defs` keyword. Use this form of the schema when you can rely
/// on retrieving the other schemas from network or other methods.
///
/// You can also use [`VSCODE_DIALECT_SCHEMA_BUNDLED`] to retrieve the bundled form of the meta
/// schema with the schema resources bundled under the `$defs` keyword.
///
/// This representation of the schema is generated with the default [`SchemaSettings`] for
/// JSON Schema draft 2020-12. To retrieve the canonical schema with custom generator settings,
/// use the [`json_schema_canonical()`] method, which takes a [`SchemaGenerator`] as input.
///
/// [`json_schema_canonical()`]: VSCodeDialect::json_schema_canonical
pub static VSCODE_DIALECT_SCHEMA_CANONICAL: LazyLock<Arc<Schema>> = LazyLock::new(|| {
    let generator: &mut SchemaGenerator = &mut SchemaGenerator::new(
        SchemaSettings::draft2020_12()
    );

    Arc::from(VSCodeDialect::json_schema_canonical(generator))
});

/// Contains the bundled form of the VS Code meta schema as a [`Resource`] so you can include
/// it in the registered resources for a [`jsonschema::Validator`] using the [`with_resource()`]
/// or [`with_resources()`] methods on the [`jsonschema::ValidationOptions`] builder.
///
/// The bundled form presents the meta schema as a compound schema document with the VS Code
/// vocabulary and keyword schemas included under the `$defs` keyword. Use this form of the
/// schema when you need every schema resource included in a single document.
///
/// You can also use [`VSCODE_DIALECT_SCHEMA_RESOURCE_CANONICAL`] to retrieve the canonical form of
/// the meta schema without the bundled schema resources.
///
/// This representation of the schema is generated with the default [`SchemaSettings`] for
/// JSON Schema draft 2020-12. To retrieve the bundled schema with custom generator settings,
/// use the [`json_schema_bundled()`] method.
///
/// [`with_resource()`]: jsonschema::ValidationOptions::with_resource
/// [`with_resources()`]: jsonschema::ValidationOptions::with_resources
/// [`json_schema_bundled()`]: VSCodeDialect::json_schema_bundled
pub static VSCODE_DIALECT_SCHEMA_RESOURCE_BUNDLED: LazyLock<Arc<Resource>> = LazyLock::new(|| {
    let generator = &mut SchemaGenerator::new(
        SchemaSettings::draft2020_12()
    );

    Arc::from(VSCodeDialect::schema_resource_bundled(generator))
});

/// Contains the canonical form of the VS Code meta schema as a [`Resource`] so you can include
/// it in the registered resources for a [`jsonschema::Validator`] using the [`with_resource()`]
/// or [`with_resources()`] methods on the [`jsonschema::ValidationOptions`] builder.
///
/// The canonical form presents the meta schema without bundling the VS Code vocabulary or
/// keyword schemas under the `$defs` keyword. Use this form of the schema when you can rely
/// on retrieving the other schemas from network or other methods.
///
/// You can also use [`VSCODE_DIALECT_SCHEMA_RESOURCE_BUNDLED`] to retrieve the bundled form of the
/// meta schema with the schema resources bundled under the `$defs` keyword.
///
/// This representation of the schema is generated with the default [`SchemaSettings`] for
/// JSON Schema draft 2020-12. To retrieve the bundled schema with custom generator settings,
/// use the [`json_schema_canonical()`] method.
///
/// [`with_resource()`]: jsonschema::ValidationOptions::with_resource
/// [`with_resources()`]: jsonschema::ValidationOptions::with_resources
/// [`json_schema_canonical()`]: VSCodeDialect::json_schema_canonical
pub static VSCODE_DIALECT_SCHEMA_RESOURCE_CANONICAL: LazyLock<Arc<Resource>> = LazyLock::new(|| {
    let generator = &mut SchemaGenerator::new(
        SchemaSettings::draft2020_12()
    );

    Arc::from(VSCodeDialect::schema_resource_canonical(generator))
});
