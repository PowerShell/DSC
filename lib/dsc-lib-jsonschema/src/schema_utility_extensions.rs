// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Provides helper functions for retrieving data from and munging [`schemars::Schema`] instances.
//!
//! The `get_keyword_as_*` functions simplify retrieving the value of a keyword for a given type.
//! If the schema defines the keyword with the expected type, those functions return a reference to
//! that value as the correct type. If the keyword doesn't exist or has the wrong value type, the
//! functions return [`None`].
//!
//! The rest of the utility methods work with specific keywords, like `$id` and `$defs`.

use core::{clone::Clone, convert::TryInto, iter::Iterator, option::Option::None};
use std::{collections::HashSet, string::String};
use std::vec::Vec;

use schemars::Schema;
use serde_json::{Map, Number, Value};
use url::{Position, Url};

/// Provides utility extension methods for [`schemars::Schema`].
pub trait SchemaUtilityExtensions {
    //********************** get_keyword_as_* functions **********************//
    /// Checks a JSON Schema for a given keyword and returns a reference to the value of that
    /// keyword, if it exists, as a [`Vec`].
    ///
    /// If the keyword doesn't exist or isn't an array, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is an array, the function returns the array.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "enum": ["foo", "bar", "baz"]
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_array("enum"),
    ///     json!(["foo", "bar", "baz"]).as_array()
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "enum": "foo, bar, baz"
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_array("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_array("enum"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_array(&self, key: &str) -> Option<&Vec<Value>>;
    /// Checks a JSON Schema for a given keyword and mutably borrows the value of that  keyword,
    /// if it exists, as a [`Vec`].
    ///
    /// If the keyword doesn't exist or isn't an array, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is an array, the function returns the array.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref mut array_json = json!(["foo", "bar", "baz"]);
    /// let ref mut schema = json_schema!({
    ///     "enum": array_json
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_array_mut("enum"),
    ///     array_json.as_array_mut()
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref mut schema = json_schema!({
    ///     "enum": "foo, bar, baz"
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_array_mut("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_array_mut("enum"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_array_mut(&mut self, key: &str) -> Option<&mut Vec<Value>>;
    /// Checks a JSON Schema for a given keyword and returns the value of that  keyword, if it
    /// exists, as a [`bool`].
    ///
    /// If the keyword doesn't exist or isn't a boolean, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is a boolean, the function returns the boolean.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "readOnly": true
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_bool("readOnly"),
    ///     Some(true)
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "readOnly": "invalid"
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_bool("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_bool("readOnly"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_bool(&self, key: &str) -> Option<bool>;
    /// Checks a JSON Schema for a given keyword and returns the value of that  keyword, if it
    /// exists, as a [`f64`].
    ///
    /// If the keyword doesn't exist or isn't a float, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is a float, the function returns the float.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "x-float-value": 1.23
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_f64("x-float-value"),
    ///     Some(1.23)
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "x-float-value": "invalid"
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_f64("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_f64("x-float-value"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_f64(&self, key: &str) -> Option<f64>;
    /// Checks a JSON Schema for a given keyword and returns the value of that  keyword, if it
    /// exists, as a [`i64`].
    ///
    /// If the keyword doesn't exist or isn't an integer, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is an integer, the function returns the integer.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "minLength": 123
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_i64("minLength"),
    ///     Some(123)
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "minLength": "invalid"
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_i64("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_i64("minLength"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_i64(&self, key: &str) -> Option<i64>;
    /// Checks a JSON Schema for a given keyword and returns the value of that  keyword, if it
    /// exists, as `()`.
    ///
    /// If the keyword doesn't exist or isn't an object, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is an integer, the function returns the integer.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "x-null-value": null
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_null("x-null-value"),
    ///     Some(())
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "x-null-value": "invalid"
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_null("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_null("x-null-value"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_null(&self, key: &str) -> Option<()>;
    /// Checks a JSON Schema for a given keyword and returns the value of that  keyword, if it
    /// exists, as a [`Map`].
    ///
    /// If the keyword doesn't exist or isn't an object, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is an object, the function returns the object.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "properties": {
    ///         "foo": { "title": "Foo property"}
    ///     }
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_object("properties"),
    ///     json!({"foo": { "title": "Foo property"}}).as_object()
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "properties": "invalid"
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_object("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_object("properties"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_object(&self, key: &str) -> Option<&Map<String, Value>>;
    /// Checks a JSON Schema for a given keyword and mutably borrows the value of that  keyword,
    /// if it exists, as a [`Map`].
    ///
    /// If the keyword doesn't exist or isn't an object, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is an object, the function returns the object.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref mut object_json = json!({
    ///     "foo": {
    ///         "title": "Foo property"
    ///     }
    /// });
    /// let ref mut schema = json_schema!({
    ///     "properties": object_json
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_object_mut("properties"),
    ///     object_json.as_object_mut()
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref mut schema = json_schema!({
    ///     "properties": "invalid"
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_object_mut("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_object_mut("enum"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_object_mut(&mut self, key: &str) -> Option<&mut Map<String, Value>>;
    /// Checks a JSON schema for a given keyword and borrows the value of that keyword, if it
    /// exists, as a [`Number`].
    ///
    /// If the keyword doesn't exist or isn't an object, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is a number, the function returns the number.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "minLength": 1
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_number("minLength"),
    ///     json!(1).as_number()
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "minLength": "invalid"
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_number("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_number("minLength"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_number(&self, key: &str) -> Option<&Number>;
    /// Checks a JSON schema for a given keyword and borrows the value of that keyword, if it
    /// exists, as a [`str`].
    ///
    /// If the keyword doesn't exist or isn't a string, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is a string, the function returns the string.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "title": "Schema title"
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_str("title"),
    ///     Some("Schema title")
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "title": true
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_str("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_str("title"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_str(&self, key: &str) -> Option<&str>;
    /// Checks a JSON schema for a given keyword and returns the value of that keyword, if it
    /// exists, as a [`String`].
    ///
    /// If the keyword doesn't exist or isn't a string, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is a string, the function returns the string.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "title": "Schema title"
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_string("title"),
    ///     Some("Schema title".to_string())
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "title": true
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_string("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_string("title"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_string(&self, key: &str) -> Option<String>;
    /// Checks a JSON Schema for a given keyword and returns the value of that  keyword, if it
    /// exists, as a [`Schema`].
    ///
    /// If the keyword doesn't exist or isn't a subschema, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is a subschema, the function returns the subschema.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "type": "array",
    ///     "items": {
    ///         "type": "string"
    ///     }
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_subschema("items"),
    ///     Some(&json_schema!({"type": "string"}))
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "items": "invalid"
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_subschema("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_subschema("items"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_subschema(&self, key: &str) -> Option<&Schema>;
    /// Checks a JSON Schema for a given keyword and mutably borrows the value of that keyword,
    /// if it exists, as a [`Schema`].
    ///
    /// If the keyword doesn't exist or isn't a subschema, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is a subschema, the function returns the subschema.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref mut subschema = json_schema!({
    ///     "type": "string"
    /// });
    /// let ref mut schema = json_schema!({
    ///     "type": "array",
    ///     "items": subschema
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_subschema_mut("items"),
    ///     Some(subschema)
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref mut schema = json_schema!({
    ///     "items": "invalid"
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_subschema_mut("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_subschema_mut("items"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_subschema_mut(&mut self, key: &str) -> Option<&mut Schema>;
    /// Checks a JSON schema for a given keyword and returns the value of that keyword, if it
    /// exists, as a [`u64`].
    ///
    /// If the keyword doesn't exist or isn't a [`u64`], this function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the given keyword exists and is a [`u64`], the function returns the [`u64`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "minLength": 5
    /// });
    /// assert_eq!(
    ///     schema.get_keyword_as_u64("minLength"),
    ///     Some(5 as u64)
    /// );
    /// ```
    ///
    /// When the given keyword doesn't exist or has the wrong data type, the function returns
    /// [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "minLength": "invalid"
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_u64("not_exist"),
    ///     None
    /// );
    ///
    /// assert_eq!(
    ///     schema.get_keyword_as_u64("minLength"),
    ///     None
    /// )
    /// ```
    fn get_keyword_as_u64(&self, key: &str) -> Option<u64>;

    //************************ $id keyword functions *************************//
    /// Retrieves the `$id` values for any entries in the `$defs` keyword.
    /// 
    /// A compound schema resource document, also called a "bundled schema", includes referenced
    /// schema resources in the `$defs` keyword. A schema resource always defines the `$id` keyword.
    /// 
    /// This method retrieves those IDs and returns a hashset containing the unique values.
    /// 
    /// Optionally, you can recursively search for schema resource IDs to handle cases where the
    /// a bundled schema resource may itself have bundled resources.
    /// 
    /// If the schema doesn't have any bundled schema resources, this method returns an empty
    /// hashset.
    /// 
    /// # Examples
    /// 
    /// This example demonstrates the difference between invoking the method for the top-level
    /// only and recursively returning nested bundled schema resources.
    /// 
    /// ```rust
    /// use std::collections::HashSet;
    /// 
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example.json",
    ///     "$defs": {
    ///         "https://contoso.com/schemas/example/string.json": {
    ///             "$id": "https://contoso.com/schemas/example/string.json",
    ///             "type": "string"
    ///         },
    ///         "https://contoso.com/schemas/example/object.json": {
    ///             "$id": "https://contoso.com/schemas/example/object.json",
    ///             "type": "object",
    ///             "unevaluatedProperties": {
    ///                 "$ref": "https://contoso.com/schemas/example/unevaluatedProperties.json"
    ///             },
    ///             "$defs": {
    ///                 "https://contoso.com/schemas/example/unevaluatedProperties.json": {
    ///                     "$id": "https://contoso.com/schemas/example/unevaluatedProperties.json"
    ///                 }
    ///             }
    ///         }
    ///     }
    /// });
    /// 
    /// let non_recursive_result: HashSet<&str> = [
    ///     "https://contoso.com/schemas/example/string.json",
    ///     "https://contoso.com/schemas/example/object.json"
    /// ].into();
    /// assert_eq!(
    ///     schema.get_bundled_schema_resource_ids(false),
    ///     non_recursive_result
    /// );
    /// 
    /// let recursive_result: HashSet<&str> = [
    ///     "https://contoso.com/schemas/example/string.json",
    ///     "https://contoso.com/schemas/example/object.json",
    ///     "https://contoso.com/schemas/example/unevaluatedProperties.json"
    /// ].into();
    /// assert_eq!(
    ///     schema.get_bundled_schema_resource_ids(true),
    ///     recursive_result
    /// );
    /// ```
    fn get_bundled_schema_resource_ids(&self, recurse: bool) -> HashSet<&str>;
    /// Retrieves the value of the `$id` keyword as a [`String`].
    ///
    /// If the schema doesn't have the `$id` keyword, this function returns [`None`].
    ///
    /// # Examples
    ///
    ///  When the schema defines the `$id` as a string, the function returns the value.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example.json"
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_id(),
    ///     Some("https://contoso.com/schemas/example.json")
    /// )
    /// ```
    fn get_id(&self) -> Option<&str>;
    /// Retrieves the value of the `$id` keyword as a [`Url`].
    ///
    /// If the schema doesn't have the `$id` keyword, or the value isn't an absolute URL, this
    /// function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the schema defines `$id` as a string representing an absolute URL, the function returns
    /// that URL object.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use url::Url;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example.json"
    /// });
    /// let id_url = Url::parse("https://contoso.com/schemas/example.json").unwrap();
    ///
    /// assert_eq!(
    ///     schema.get_id_as_url(),
    ///     Some(id_url)
    /// )
    /// ```
    fn get_id_as_url(&self) -> Option<Url>;
    /// Indicates whether the [`Schema`] defines the `$id` keyword.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref schema = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example.json"
    /// });
    ///
    /// assert_eq!(
    ///     schema.has_id_keyword(),
    ///     true
    /// )
    /// ```
    fn has_id_keyword(&self) -> bool;
    /// Defines the `$id` keyword for the [`Schema`], returning the old value if `$id` was already
    /// defined.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref mut schema = json_schema!({
    ///     "title": "Example schema"
    /// });
    ///
    /// // Because the schema didn't already define `$id`, there's no prior value to return.
    /// assert_eq!(
    ///     schema.set_id("https://contoso.com/schemas/initial.json"),
    ///     None
    /// );
    /// // When the ID is set a second time, the prior value is returned.
    /// assert_eq!(
    ///     schema.set_id("https://contoso.com/schemas/final.json"),
    ///     Some("https://contoso.com/schemas/initial.json".to_string())
    /// );
    /// ```
    fn set_id(&mut self, id_uri: &str) -> Option<String>;

    //*********************** $defs keyword functions ************************//
    /// Retrieves the `$defs` keyword and returns the object if it exists.
    ///
    /// If the keyword isn't defined or isn't an object, the function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the schema defines `$defs` as an object, the function returns a reference to that
    /// object.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref defs_json = json!({
    ///     "foo": {
    ///         "title": "Foo subschema"
    ///     }
    /// });
    /// let ref schema = json_schema!({
    ///     "$defs": defs_json
    /// });
    /// assert_eq!(
    ///     schema.get_defs(),
    ///     defs_json.as_object()
    /// );
    /// ```
    fn get_defs(&self) -> Option<&Map<String, Value>>;
    /// Retrieves the `$defs` keyword and mutably borrows the object if it exists.
    ///
    /// If the keyword isn't defined or isn't an object, the function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the schema defines `$defs` as an object, the function mutably borrows that
    /// object.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref mut defs_json = json!({
    ///     "foo": {
    ///         "title": "Foo subschema"
    ///     }
    /// });
    /// let ref mut schema = json_schema!({
    ///     "$defs": defs_json
    /// });
    /// assert_eq!(
    ///     schema.get_defs_mut(),
    ///     defs_json.as_object_mut()
    /// );
    /// ```
    fn get_defs_mut(&mut self) -> Option<&mut Map<String, Value>>;
    /// Looks up a reference in the `$defs` keyword by `$id` and returns the subschema entry as a
    /// [`Schema`] if it exists.
    ///
    /// The value for the `id` _must_ be the absolute URL of the target subschema's `$id` keyword.
    /// If the target subschema doesn't define the `$id` keyword, this function can't resolve the
    /// lookup.
    ///
    /// For a more flexible lookup, use the [`get_defs_subschema_from_reference()`] function
    /// instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref definition = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example/foo.json",
    ///     "title": "Foo property"
    /// });
    /// let ref schema = json_schema!({
    ///     "$defs": {
    ///         "foo": definition
    ///     }
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_id("https://contoso.com/schemas/example/foo.json"),
    ///     Some(definition)
    /// );
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_id("/schemas/example/foo.json"),
    ///     None
    /// );
    /// ```
    ///
    /// [`get_defs_subschema_from_reference()`]: SchemaUtilityExtensions::get_defs_subschema_from_reference
    fn get_defs_subschema_from_id(&self, id: &str) -> Option<&Schema>;
    /// Looks up a reference in the `$defs` keyword by `$id` and mutably borrows the subschema
    /// entry as a [`Schema`] if it exists.
    ///
    /// The value for the `id` _must_ be the absolute URL of the target subschema's `$id` keyword.
    /// If the target subschema doesn't define the `$id` keyword, this function can't resolve the
    /// lookup.
    ///
    /// For a more flexible lookup, use the [`get_defs_subschema_from_reference_mut()`] function
    /// instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref mut definition = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example/foo.json",
    ///     "title": "Foo property"
    /// });
    /// let ref mut schema = json_schema!({
    ///     "$defs": {
    ///         "foo": definition
    ///     }
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_id_mut("https://contoso.com/schemas/example/foo.json"),
    ///     Some(definition)
    /// );
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_id_mut("/schemas/example/foo.json"),
    ///     None
    /// );
    /// ```
    ///
    /// [`get_defs_subschema_from_reference_mut()`]: SchemaUtilityExtensions::get_defs_subschema_from_reference_mut
    fn get_defs_subschema_from_id_mut(&mut self, id: &str) -> Option<&mut Schema>;
    /// Looks up a reference in the `$defs` keyword and returns the subschema entry as a [`Schema`]
    /// if it exists.
    ///
    /// The reference can be any of the following:
    ///
    /// - A URI identifier fragment, like `#/$defs/foo`
    /// - An absolute URL for the referenced schema, like `https://contoso.com/schemas/example.json`
    /// - A site-relative URL for the referenced schema, like `/schemas/example.json`. The function
    ///   can only resolve site-relative URLs when the schema itself defines `$id` with an absolute
    ///   URL, because it uses the current schema's `$id` as the base URL.
    ///
    /// If the reference can't be resolved or resolves to a non-object value, this function returns
    /// [`None`].
    ///
    /// # Examples
    ///
    /// You can retrieve a definition with a fragment point, the absolute URL of the target schema's
    /// `$id` keyword, or the relative URL of the target schema's `$id` keyword.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref definition = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example/foo.json",
    ///     "title": "Foo property"
    /// });
    /// let ref schema = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example.json",
    ///     "$defs": {
    ///         "foo": definition
    ///     }
    /// });
    /// // Lookup with pointer:
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_reference("#/$defs/foo"),
    ///     Some(definition)
    /// );
    /// // Lookup with absolute URL:
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_reference("https://contoso.com/schemas/example/foo.json"),
    ///     Some(definition)
    /// );
    /// // Lookup with site-relative URL:
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_reference("/schemas/example/foo.json"),
    ///     Some(definition)
    /// );
    /// ```
    ///
    /// If the [`Schema`] _doesn't_ define the `$id` keyword as an absolute URL, lookups for
    /// site-relative references fail to resolve and return [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref definition = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example/foo.json",
    ///     "title": "Foo property"
    /// });
    /// let ref schema = json_schema!({
    ///     "$defs": {
    ///         "foo": definition
    ///     }
    /// });
    /// // Lookup with pointer:
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_reference("#/$defs/foo"),
    ///     Some(definition)
    /// );
    /// // Lookup with absolute URL:
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_reference("https://contoso.com/schemas/example/foo.json"),
    ///     Some(definition)
    /// );
    /// // Lookup with site-relative URL:
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_reference("/schemas/example/foo.json"),
    ///     None
    /// );
    /// ```
    fn get_defs_subschema_from_reference(&self, reference: &str) -> Option<&Schema>;
    /// Looks up a reference in the `$defs` keyword and mutably borrows the subschema entry as a
    /// [`Schema`] if it exists.
    ///
    /// The reference can be any of the following:
    ///
    /// - An absolute URL for the referenced schema, like `https://contoso.com/schemas/example.json`
    /// - A site-relative URL for the referenced schema, like `/schemas/example.json`. The function
    ///   can only resolve site-relative URLs when the schema itself defines `$id` with an absolute
    ///   URL, because it uses the current schema's `$id` as the base URL.
    ///
    /// If the reference can't be resolved or resolves to a non-object value, this function returns
    /// [`None`].
    ///
    /// Due to a bug in [`schemars::Schema::pointer_mut()`], this function can't correctly resolve
    /// references from URI fragment identifiers like `#/$defs/foo`, unlike
    /// [`get_defs_subschema_from_reference()`]. Until the [fixing PR] is merged and included in a
    /// [`schemars`] release, this function can only resolve absolute and relative URLs matching
    /// the target definitions subschema's `$id` keyword. For more information on the bug, see
    /// see [schemars#478].
    ///
    /// # Examples
    ///
    /// You can retrieve a definition with the absolute URL of the target schema's `$id` keyword or
    /// the relative URL of the target schema's `$id` keyword.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref mut definition = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example/foo.json",
    ///     "title": "Foo property"
    /// });
    /// let ref mut schema = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example.json",
    ///     "$defs": {
    ///         "foo": definition
    ///     }
    /// });
    /// // Lookup with absolute URL:
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_reference_mut("https://contoso.com/schemas/example/foo.json").unwrap(),
    ///     definition
    /// );
    /// // Lookup with site-relative URL:
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_reference_mut("/schemas/example/foo.json").unwrap(),
    ///     definition
    /// );
    /// ```
    ///
    /// If the [`Schema`] _doesn't_ define the `$id` keyword as an absolute URL, lookups for
    /// site-relative references fail to resolve and return [`None`].
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref mut definition = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example/foo.json",
    ///     "title": "Foo property"
    /// });
    /// let ref mut schema = json_schema!({
    ///     "$defs": {
    ///         "foo": definition
    ///     }
    /// });
    /// // Lookup with absolute URL:
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_reference_mut("https://contoso.com/schemas/example/foo.json"),
    ///     Some(definition)
    /// );
    /// // Lookup with site-relative URL:
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_reference_mut("/schemas/example/foo.json"),
    ///     None
    /// );
    /// ```
    ///
    /// [`get_defs_subschema_from_reference()`]: SchemaUtilityExtensions::get_defs_subschema_from_reference
    /// [schemars#478]: https://github.com/GREsau/schemars/issues/478
    /// [fixing PR]: https://github.com/GREsau/schemars/pull/479
    fn get_defs_subschema_from_reference_mut(&mut self, reference: &str) -> Option<&mut Schema>;
    /// Inserts a subschema entry into the `$defs` keyword for the [`Schema`]. If an entry for the
    /// given key already exists, this function returns the old value as a map.
    ///
    /// If the [`Schema`] doesn't define the `$defs` keyword, this function inserts it as an object
    /// containing the given key-value pair for the definition.
    ///
    /// # Examples
    ///
    /// When the given definition key exists, the function returns that value as an object after
    /// replacing it in the `$defs` object.
    ///
    /// ```
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let original_definition = json_schema!({
    ///     "title": "Foo property"
    /// }).clone();
    /// let mut new_definition = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example/foo.json",
    /// }).clone();
    /// let ref mut schema = json_schema!({
    ///     "$defs": {
    ///         "foo": original_definition
    ///     }
    /// });
    /// assert_eq!(
    ///     schema.insert_defs_subschema("foo", &new_definition.as_object().unwrap()),
    ///     original_definition.as_object().cloned()
    /// );
    /// assert_eq!(
    ///     schema.get_defs_subschema_from_reference_mut("https://contoso.com/schemas/example/foo.json"),
    ///     Some(&mut new_definition)
    /// )
    /// ```
    fn insert_defs_subschema(&mut self, definition_key: &str, definition_value: &Map<String, Value>) -> Option<Map<String, Value>>;
    /// Looks up a subschema in the `$defs` keyword by reference and, if it exists, renames the
    /// _key_ for the definition.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let definition = json!({
    ///     "$id": "https://contoso.com/schemas/example/foo.json",
    ///     "title": "Foo property"
    /// });
    /// let ref mut schema = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example.json",
    ///     "$defs": {
    ///         "foo": definition.clone()
    ///     }
    /// });
    /// // Lookup the definition by site-relative URL and replace with full ID
    /// schema.rename_defs_subschema_for_reference(
    ///     "/schemas/example/foo.json",
    ///     "https://contoso.com/schemas/example/foo.json"
    /// );
    /// // Prior key no longer resolveable
    /// assert_eq!(
    ///     schema.get_defs_mut().unwrap().get("foo"),
    ///     None
    /// );
    /// // New key contains expected value
    /// assert_eq!(
    ///     schema.get_defs_mut().unwrap()
    ///           .get("https://contoso.com/schemas/example/foo.json")
    ///           .unwrap()
    ///           .as_object(),
    ///     definition.as_object()
    /// )
    /// ```
    fn rename_defs_subschema_for_reference(&mut self, reference: &str, new_name: &str);

    //********************* properties keyword functions *********************//
    /// Retrieves the `properties` keyword and returns the object if it exists.
    ///
    /// If the keyword isn't defined or isn't an object, the function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the schema defines `properties` as an object, the function returns a reference to that
    /// object.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref properties_json = json!({
    ///     "foo": {
    ///         "title": "Foo property"
    ///     }
    /// });
    /// let ref schema = json_schema!({
    ///     "properties": properties_json
    /// });
    /// assert_eq!(
    ///     schema.get_properties(),
    ///     properties_json.as_object()
    /// );
    /// ```
    fn get_properties(&self) -> Option<&Map<String, Value>>;
    /// Retrieves the `properties` keyword and mutably borrows the object if it exists.
    ///
    /// If the keyword isn't defined or isn't an object, the function returns [`None`].
    ///
    /// # Examples
    ///
    /// When the schema defines `properties` as an object, the function mutably borrows that
    /// object.
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref mut properties_json = json!({
    ///     "foo": {
    ///         "title": "Foo subschema"
    ///     }
    /// });
    /// let ref mut schema = json_schema!({
    ///     "properties": properties_json
    /// });
    /// assert_eq!(
    ///     schema.get_properties_mut(),
    ///     properties_json.as_object_mut()
    /// );
    /// ```
    fn get_properties_mut(&mut self) -> Option<&mut Map<String, Value>>;
    /// Looks up a property in the `properties` keyword by name and returns the subschema entry as
    /// a [`Schema`] if it exists.
    ///
    /// If the named property doesn't exist or isn't a valid subschema, this function returns [`None`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use serde_json::json;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref property = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example/foo.json",
    ///     "title": "Foo property"
    /// });
    /// let ref schema = json_schema!({
    ///     "properties": {
    ///         "foo": property
    ///     }
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_property_subschema("foo"),
    ///     Some(property)
    /// );
    /// ```
    fn get_property_subschema(&self, property_name: &str) -> Option<&Schema>;
    /// Looks up a property in the `properties` keyword by name and mutably borrows the subschema
    /// entry as a [`Schema`] if it exists.
    ///
    /// If the named property doesn't exist or isn't a [`Schema`], this function returns [`None`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let ref mut property = json_schema!({
    ///     "$id": "https://contoso.com/schemas/example/foo.json",
    ///     "title": "Foo property"
    /// });
    /// let ref mut schema = json_schema!({
    ///     "properties": {
    ///         "foo": property
    ///     }
    /// });
    ///
    /// assert_eq!(
    ///     schema.get_property_subschema_mut("foo"),
    ///     Some(property)
    /// );
    /// ```
    fn get_property_subschema_mut(&mut self, property_name: &str) -> Option<&mut Schema>;

    //************************ $ref keyword functions ************************//
    /// Retrieves the value for every `$ref` keyword from the [`Schema`] as a [`HashSet`] of
    /// unique values.
    /// 
    /// 
    /// This method recurses through a given schema for the `$ref` keyword and inserts the value
    /// into a hashset to return. If the schema doesn't define any references, this method returns
    /// an empty hashset.
    /// 
    /// # Examples
    /// 
    /// This example shows how the method returns a unique set of references by recursively
    /// searching the schema for the `$ref` keyword.
    /// 
    /// ```rust
    /// use std::collections::HashSet;
    /// 
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let schema = &json_schema!({
    ///     "id": "https://contoso.com/schemas/example/object.json",
    ///     "type": "object",
    ///     "properties": {
    ///         "name": { "$ref": "/schemas/example/properties/name.json" },
    ///         "age": { "$ref": "/schemas/example/properties/age.json" },
    ///     },
    /// });
    /// 
    /// let references: HashSet<&str> = [
    ///     "/schemas/example/properties/name.json",
    ///     "/schemas/example/properties/age.json"
    /// ].into();
    /// 
    /// assert_eq!(
    ///     schema.get_references(),
    ///     references
    /// )
    /// ```
    fn get_references(&self) -> HashSet<&str>;
    /// Searches the schema for instances of the `$ref` keyword defined as a
    /// given value and replaces each instance with a new value.
    /// 
    /// This method simplifies replacing references programmatically, especially
    /// for converting references to use the canonical ID of a bundled schema
    /// resource.
    /// 
    /// # Examples
    /// 
    /// This example replaces the reference to `#/$defs/name.json` with the
    /// canonical ID URI for the referenced schema resource, which is bundled
    /// in the schema.
    /// 
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let schema = &mut json_schema!({
    ///     "id": "https://contoso.com/schemas/example/object.json",
    ///     "type": "object",
    ///     "properties": {
    ///         "name": { "$ref": "#/$defs/name" },
    ///     },
    ///     "$defs": {
    ///         "name": {
    ///             "$id": "https://contoso.com/schemas/example/properties/name.json",
    ///             "type": "string"
    ///         }
    ///     }
    /// });
    /// 
    /// schema.replace_references(
    ///     "#/$defs/name",
    ///     "https://contoso.com/schemas/example/properties/name.json"
    /// );
    /// 
    /// assert_eq!(
    ///     schema.get_references().into_iter().nth(0).unwrap(),
    ///     "https://contoso.com/schemas/example/properties/name.json"
    /// );
    /// ```
    fn replace_references(&mut self, find_value: &str, new_value: &str);
    /// Checks whether a given reference maps to a bundled schema resource.
    /// 
    /// This method takes the value of a `$ref` keyword and searches for a matching entry in the
    /// `$defs` keyword. The method returns `true` if the reference resolves to an entry in
    /// `$defs` and otherwise false.
    /// 
    /// The reference can be any of the following:
    ///
    /// - A URI identifier fragment, like `#/$defs/foo`
    /// - An absolute URL for the referenced schema, like `https://contoso.com/schemas/example.json`
    /// - A site-relative URL for the referenced schema, like `/schemas/example.json`. The function
    ///   can only resolve site-relative URLs when the schema itself defines `$id` with an absolute
    ///   URL, because it uses the current schema's `$id` as the base URL.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use schemars::json_schema;
    /// use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    ///
    /// let schema = &json_schema!({
    ///     "$id": "https://contoso.com/schemas/example/object.json",
    ///     "$defs": {
    ///         "name": {
    ///             "$id": "https://contoso.com/schemas/example/properties/name.json",
    ///             "type": "string"
    ///         }
    ///     }
    /// });
    /// 
    /// // Resolving reference as pointer
    /// assert_eq!(schema.reference_is_for_bundled_resource("#/$defs/name"), true);
    /// // Resolving reference as site-relative URI
    /// assert_eq!(
    ///     schema.reference_is_for_bundled_resource("/schemas/example/properties/name.json"),
    ///     true
    /// );
    /// // Resolving reference as absolute URI
    /// assert_eq!(
    ///     schema.reference_is_for_bundled_resource(
    ///         "https://contoso.com/schemas/example/properties/name.json"
    ///     ),
    ///     true
    /// );
    /// // Returns false for unresolvable definition
    /// assert_eq!(schema.reference_is_for_bundled_resource("#/$defs/invalid"), false);
    /// ```
    fn reference_is_for_bundled_resource(&self, reference: &str) -> bool;
}

impl SchemaUtilityExtensions for Schema {
    fn get_keyword_as_array(&self, key: &str) -> Option<&Vec<Value>> {
        self.get(key)
            .and_then(Value::as_array)
    }
    fn get_keyword_as_array_mut(&mut self, key: &str) -> Option<&mut Vec<Value>> {
        self.get_mut(key)
            .and_then(Value::as_array_mut)
    }
    fn get_keyword_as_bool(&self, key: &str) -> Option<bool> {
        self.get(key)
            .and_then(Value::as_bool)
    }
    fn get_keyword_as_f64(&self, key: &str) -> Option<f64> {
        self.get(key)
            .and_then(Value::as_f64)
    }
    fn get_keyword_as_i64(&self, key: &str) -> Option<i64> {
        self.get(key)
            .and_then(Value::as_i64)
    }
    fn get_keyword_as_null(&self, key: &str) -> Option<()> {
        self.get(key)
            .and_then(Value::as_null)
    }
    fn get_keyword_as_number(&self, key: &str) -> Option<&Number> {
        self.get(key)
            .and_then(Value::as_number)
    }
    fn get_keyword_as_object(&self, key: &str) -> Option<&Map<String, Value>> {
        self.get(key)
            .and_then(Value::as_object)
    }
    fn get_keyword_as_object_mut(&mut self, key: &str) -> Option<&mut Map<String, Value>> {
        self.get_mut(key)
            .and_then(Value::as_object_mut)
    }
    fn get_keyword_as_str(&self, key: &str) -> Option<&str> {
        self.get(key)
            .and_then(Value::as_str)
    }
    fn get_keyword_as_string(&self, key: &str) -> Option<String> {
        self.get(key)
            .and_then(Value::as_str)
            .map(std::string::ToString::to_string)
    }
    fn get_keyword_as_subschema(&self, key: &str) -> Option<&Schema> {
        self.get(key)
            .and_then(|v| <&Value as TryInto<&Schema>>::try_into(v).ok())
    }
    fn get_keyword_as_subschema_mut(&mut self, key: &str) -> Option<&mut Schema> {
        self.get_mut(key)
            .and_then(|v| <&mut Value as TryInto<&mut Schema>>::try_into(v).ok())
    }
    fn get_keyword_as_u64(&self, key: &str) -> Option<u64> {
        self.get(key)
            .and_then(Value::as_u64)
    }
    fn get_defs(&self) -> Option<&Map<String, Value>> {
        self.get_keyword_as_object("$defs")
    }
    fn get_defs_mut(&mut self) -> Option<&mut Map<String, Value>> {
        self.get_keyword_as_object_mut("$defs")
    }
    fn get_defs_subschema_from_id(&self, id: &str) -> Option<&Schema> {
        let defs = self.get_defs()?;

        for def in defs.values() {
            if let Some(definition) = def.as_object() {
                let def_id = definition.get("$id").and_then(Value::as_str);

                if def_id == Some(id) {
                    return <&Value as TryInto<&Schema>>::try_into(def).ok()
                }
            }
        }

        None
    }
    fn get_defs_subschema_from_id_mut(&mut self, id: &str) -> Option<&mut Schema> {
        let defs = self.get_defs_mut()?;

        for def in defs.values_mut() {
            if let Some(definition) = def.as_object_mut() {
                let def_id = definition.get("$id").and_then(Value::as_str);

                if def_id == Some(id) {
                    return <&mut Value as TryInto<&mut Schema>>::try_into(def).ok()
                }
            }
        }

        None
    }
    fn get_defs_subschema_from_reference(&self, reference: &str) -> Option<&Schema> {
        // If the reference is a normative pointer to $defs, short-circuit.
        if reference.to_string().starts_with("#/$defs/") {
            return self.pointer(reference).and_then(|v| {
                <&Value as TryInto<&Schema>>::try_into(v).ok()
            });
        }

        let id = reference.to_string();
        // if the reference is a full URL, look up subschema by $id
        if id.starts_with("https://") {
            return self.get_defs_subschema_from_id(reference);
        }
        // if the reference is a relative URL, try to compose ID from current schema $id
        if let Some(schema_id) = self.get_id_as_url() {
            let url_prefix = schema_id[..Position::BeforePath].to_string();
            let id = format!("{url_prefix}{id}");
            return self.get_defs_subschema_from_id(&id)
        }

        None
    }
    fn get_defs_subschema_from_reference_mut(&mut self, reference: &str) -> Option<&mut Schema> {
        // If the reference is a normative pointer to $defs, short-circuit.
        if reference.to_string().starts_with("#/$defs/") {
            return self.pointer_mut(reference).and_then(|v| {
                <&mut Value as TryInto<&mut Schema>>::try_into(v).ok()
            });
        }

        let id = reference.to_string();
        // if the reference is a full URL, look up subschema by $id
        if id.starts_with("https://") {
            return self.get_defs_subschema_from_id_mut(reference);
        }
        // if the reference is a relative URL, try to compose ID from current schema $id
        if let Some(schema_id) = self.get_id_as_url() {
            let url_prefix = schema_id[..Position::BeforePath].to_string();
            let id = format!("{url_prefix}{id}");
            return self.get_defs_subschema_from_id_mut(&id)
        }

        None
    }
    fn insert_defs_subschema(
        &mut self,
        definition_key: &str,
        definition_value: &Map<String, Value>
    ) -> Option<Map<String, Value>> {
        if let Some(defs) = self.get_defs_mut() {
            let old_value = defs.clone()
                .get(definition_key)
                .and_then(Value::as_object)
                .cloned();

            defs.insert(definition_key.to_string(), Value::Object(definition_value.clone()))
                .and(old_value)
        } else {
            let defs: &mut Map<String, Value> = &mut Map::new();
            defs.insert(definition_key.to_string(), Value::Object(definition_value.clone()));
            self.insert("$defs".to_string(), Value::Object(defs.clone()));

            None
        }
    }
    fn rename_defs_subschema_for_reference(&mut self, reference: &str, new_name: &str) {
        let lookup_self = self.clone();
        // Lookup the reference. If unresolved, return immediately.
        let Some(value) = lookup_self.get_defs_subschema_from_reference(reference).and_then(Schema::as_object) else {
            return;
        };
        // If defs can't be retrieved mutably, return immediately.
        let Some(defs) = self.get_defs_mut() else {
            return;
        };
        // Replace the existing key in the map by looking for the key-value pair with the same
        // value and rename it.
        let new_key = &new_name.to_string();
        *defs = defs.iter_mut().map(|(k, v)| {
            if *v == Value::Object(value.clone()) {
                (new_key.clone(), v.clone())
            } else {
                (k.clone(), v.clone())
            }
        }).collect();
    }
    fn get_bundled_schema_resource_ids(&self, recurse: bool) -> HashSet<&str> {
        let mut schema_resource_ids: HashSet<&str> = HashSet::new();

        let Some(defs) = self.get_defs() else {
            return schema_resource_ids
        };

        for (_key, value) in defs {
            let Ok(def) = <&Value as TryInto<&Schema>>::try_into(value) else {
                continue;
            };
            if let Some(id) = def.get_id() {
                schema_resource_ids.insert(id);
            }
            if recurse {
                let recursive = def.get_bundled_schema_resource_ids(recurse);

                schema_resource_ids = schema_resource_ids
                    .union(&recursive)
                    .copied()
                    .collect();
            }
        }

        schema_resource_ids
    }
    fn get_id(&self) -> Option<&str> {
        self.get_keyword_as_str("$id")
    }
    fn get_id_as_url(&self) -> Option<Url> {
        // By default `Url::parse` fails for non-absolute URLs.
        match self.get_id() {
            None => None,
            Some(id_str) => Url::parse(id_str).ok()
        }
    }
    fn has_id_keyword(&self) -> bool {
        self.get_id().is_some()
    }
    fn set_id(&mut self, id_uri: &str) -> Option<String> {
        // Unfortunately, we need to clone the Schema to immutably retrieve the ID to return it.
        // Attempting to return it from the `insert().and_then()` fails to compile for temporary
        // value.
        let old_id = self.clone()
            .get_mut("$id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        self.insert("$id".to_string(), Value::String(id_uri.to_string()))
            .and(old_id)
    }
    fn get_properties(&self) -> Option<&Map<String, Value>> {
        self.get_keyword_as_object("properties")
    }
    fn get_properties_mut(&mut self) -> Option<&mut Map<String, Value>> {
        self.get_keyword_as_object_mut("properties")
    }
    fn get_property_subschema(&self, property_name: &str) -> Option<&Schema> {
        self.get_properties()
            .and_then(|properties| properties.get(property_name))
            .and_then(|v| <&Value as TryInto<&Schema>>::try_into(v).ok())
    }
    fn get_property_subschema_mut(&mut self, property_name: &str) -> Option<&mut Schema> {
        self.get_properties_mut()
            .and_then(|properties| properties.get_mut(property_name))
            .and_then(|v| <&mut Value as TryInto<&mut Schema>>::try_into(v).ok())
    }
    fn get_references(&self) -> HashSet<&str> {
        let mut references: HashSet<&str> = HashSet::new();
        // First, check the top-level for a reference
        if let Some(reference) = self.get_keyword_as_str("$ref") {
            references.insert(reference);
        }

        // Next, recursively check references in subschemas
        for (key, value) in self.as_object().into_iter().flatten() {
            // Recursing implementation borrowed from schemars::transform::transform_subschemas - needed
            // to recursively transform the schema while passing parameters other than the schema
            // itself, which is the only supported option for a function implementing the
            // Transform trait.
            match key.as_str() {
                "not"
                | "if"
                | "then"
                | "else"
                | "contains"
                | "additionalProperties"
                | "propertyNames"
                | "additionalItems" => {
                    if let Ok(subschema) = <&Value as TryInto<&Schema>>::try_into(value) {
                        references.extend(subschema.get_references());
                    }
                }
                "allOf" | "anyOf" | "oneOf" | "prefixItems" => {
                    if let Some(array) = value.as_array() {
                        for value in array {
                            if let Ok(subschema) = <&Value as TryInto<&Schema>>::try_into(value) {
                                references.extend(subschema.get_references());
                            }
                        }
                    }
                }
                "items" => {
                    if let Some(array) = value.as_array() {
                        for value in array {
                            if let Ok(subschema) = <&Value as TryInto<&Schema>>::try_into(value) {
                                references.extend(subschema.get_references());
                            }
                        }
                    } else if let Ok(subschema) = <&Value as TryInto<&Schema>>::try_into(value) {
                        references.extend(subschema.get_references());
                    }
                }
                "properties" | "patternProperties" | "$defs" | "definitions" => {
                    if let Some(obj) = value.as_object() {
                        for value in obj.values() {
                            if let Ok(subschema) = <&Value as TryInto<&Schema>>::try_into(value) {
                                references.extend(subschema.get_references());
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        references
    }
    fn replace_references(&mut self, find_value: &str, new_value: &str) {
        if self.get_keyword_as_str("$ref").is_some_and(|r| r == find_value) {
            self.insert("$ref".to_string(), Value::String(new_value.to_string()));
        }

        for (key, value) in self.as_object_mut().into_iter().flatten() {
            // Recursing implementation borrowed from schemars::transform::transform_subschemas - needed
            // to recursively transform the schema while passing parameters other than the schema
            // itself, which is the only supported option for a function implementing the
            // Transform trait.
            match key.as_str() {
                "not"
                | "if"
                | "then"
                | "else"
                | "contains"
                | "additionalProperties"
                | "propertyNames"
                | "additionalItems" => {
                    if let Ok(subschema) = <&mut Value as TryInto<&mut Schema>>::try_into(value) {
                        subschema.replace_references(find_value, new_value);
                    }
                }
                "allOf" | "anyOf" | "oneOf" | "prefixItems" => {
                    if let Some(array) = value.as_array_mut() {
                        for value in array {
                            if let Ok(subschema) = <&mut Value as TryInto<&mut Schema>>::try_into(value) {
                                subschema.replace_references(find_value, new_value);
                            }
                        }
                    }
                }
                "items" => {
                    if let Some(array) = value.as_array_mut() {
                        for value in array {
                            if let Ok(subschema) = <&mut Value as TryInto<&mut Schema>>::try_into(value) {
                                subschema.replace_references(find_value, new_value);
                            }
                        }
                    } else if let Ok(subschema) = <&mut Value as TryInto<&mut Schema>>::try_into(value) {
                        subschema.replace_references(find_value, new_value);
                    }
                }
                "properties" | "patternProperties" | "$defs" | "definitions" => {
                    if let Some(obj) = value.as_object_mut() {
                        for value in obj.values_mut() {
                            if let Ok(subschema) = <&mut Value as TryInto<&mut Schema>>::try_into(value) {
                                subschema.replace_references(find_value, new_value);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    fn reference_is_for_bundled_resource(&self, reference: &str) -> bool {
        self.get_defs_subschema_from_reference(reference).is_some()
    }
}
