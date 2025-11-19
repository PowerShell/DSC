// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use jsonschema::{Keyword, Resource, ValidationError, paths::Location};
use schemars::{JsonSchema, Schema, SchemaGenerator, generate::SchemaSettings, json_schema};
use serde_json::{Map, Value};

/// Defines a VS Code keyword for the [`jsonschema`] crate.
///
/// This trait requires the target type to implement both the [`JsonSchema`] and [`Keyword`] traits,
/// as this crate publishes both a vocabulary and meta schema for using the VS Code keywords.
pub trait VSCodeKeywordDefinition : JsonSchema + Keyword {
    /// Defines the property name for the keyword, like `markdownDescription`.
    const KEYWORD_NAME: &str;

    /// Defines the canonical `$id` URI for the keyword.
    const KEYWORD_ID: &str;

    /// Defines the meta schema used to validate the keyword's own schema definition.
    const META_SCHEMA: &str = "https://json-schema.org/draft/2020-12/schema";

    /// Defines the factory function [`jsonschema`] requires for registering a custom keyword.
    ///
    /// For more information, see [`Keyword`].
    /// 
    /// # Errors
    /// 
    /// The function returns a [`ValidationError`] when the JSON value is invalid for a given
    /// keyword. In practice, none of the VS Code keywords ever return a validation error because
    /// they are all annotation keywords.
    #[allow(clippy::result_large_err)]
    fn keyword_factory<'a>(
        _parent: &'a Map<String, Value>,
        value: &'a Value,
        path: Location,
    ) -> Result<Box<dyn Keyword>, ValidationError<'a>>;

    /// Returns the default representation of the JSON Schema for the keyword.
    ///
    /// The [`JsonSchema`] trait requires the [`json_schema()`] function to accept a mutable
    /// reference to a [`SchemaGenerator`] for transforming and otherwise modifying the schema.
    ///
    /// The VS Code keyword schemas are statically defined with the [`json_schema!()`] macro and
    /// don't use the generator. This convenience method passes a dummy default generator to the
    /// [`json_schema()`] function so you can retrieve the schema without always needing to
    /// instantiate a generator you're not using. In other cases, where you _do_ want to apply
    /// transforms to every generated schema, you can still access the [`json_schema()`] trait
    /// function directly.
    ///
    /// [`json_schema()`]: JsonSchema::json_schema
    /// [`json_schema!()`]: schemars::json_schema!
    #[must_use]
    fn default_schema() -> Schema {
        let generator = &mut SchemaGenerator::new(
            SchemaSettings::draft2020_12()
        );

        Self::json_schema(generator)
    }

    /// Returns a [`Schema`] object using the `$ref` keyword to point to the
    /// VS Code keyword's canonical `$id` URI.
    /// 
    /// This convenience method enables you to retrieve the reference subschema
    /// without needing to construct it manually.
    #[must_use]
    fn schema_reference() -> Schema {
        json_schema!({
            "$ref": Self::KEYWORD_ID
        })
    }

    /// Returns the default schema for the keyword as a [`Resource`] to register with a
    /// [`jsonschema::Validator`].
    ///
    /// # Panics
    ///
    /// This method panics if the schema is malformed and can't be converted into a [`Resource`].
    ///
    /// In practice, you should never see a panic from this method because the crate's test suite
    /// checks for this failure mode.
    #[must_use]
    fn default_schema_resource() -> Resource {
        Resource::from_contents(Self::default_schema().to_value())
    }
}