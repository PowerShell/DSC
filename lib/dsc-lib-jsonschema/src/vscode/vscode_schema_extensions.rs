// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Provides helper functions for working with VS Code's extended JSON Schema keywords with
//! [`schemars::Schema`] instances.
//!
//! The `get_keyword_as_*` functions simplify retrieving the value of a keyword for a given type.
//! If the schema defines the keyword with the expected type, those functions return a reference to
//! that value as the correct type. If the keyword doesn't exist or has the wrong value type, the
//! functions return [`None`].
//!
//! The rest of the utility methods work with specific keywords, like `$id` and `$defs`.

use core::{clone::Clone, iter::Iterator, option::Option::None};
use std::string::String;

use schemars::Schema;
use serde_json::{Map, Number, Value};
use url::{Position, Url};

type Array = Vec<Value>;
type Object = Map<String, Value>;

pub trait VSCodeSchemaExtensions {
    fn has_markdown_description(&self) -> bool;
}

impl VSCodeSchemaExtensions for Schema {
    fn has_markdown_description(&self) -> bool {
        self.get("markdownDescription").is_some()
    }
}