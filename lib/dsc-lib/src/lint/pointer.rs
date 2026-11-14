// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::fmt::Display;

use serde_json::Value;

/// A JSON pointer (RFC 6901) that identifies a location in a document.
///
/// The empty pointer identifies the document root.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JsonPointer(String);

impl JsonPointer {
    /// The pointer to the document root.
    #[must_use]
    pub fn root() -> Self {
        Self(String::new())
    }

    /// Create a pointer from an existing pointer string, such as one produced by a JSON Schema
    /// validator.
    ///
    /// # Arguments
    ///
    /// * `pointer` - The pointer text, which must be empty or start with `/`.
    #[must_use]
    pub fn from_pointer(pointer: &str) -> Self {
        Self(pointer.to_string())
    }

    /// Return a pointer to the member `key` of the object at this pointer.
    ///
    /// # Arguments
    ///
    /// * `key` - The member name; `~` and `/` are escaped as the RFC requires.
    #[must_use]
    pub fn push(&self, key: &str) -> Self {
        let escaped = key.replace('~', "~0").replace('/', "~1");
        Self(format!("{}/{escaped}", self.0))
    }

    /// Return a pointer to the element `index` of the array at this pointer.
    ///
    /// # Arguments
    ///
    /// * `index` - The zero-based element index.
    #[must_use]
    pub fn push_index(&self, index: usize) -> Self {
        Self(format!("{}/{index}", self.0))
    }

    /// Return a pointer that appends another pointer to this one.
    ///
    /// # Arguments
    ///
    /// * `other` - The pointer text to append, which must be empty or start with `/`.
    #[must_use]
    pub fn join(&self, other: &str) -> Self {
        Self(format!("{}{other}", self.0))
    }

    /// The pointer text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether this pointer identifies `other` or a location below it.
    ///
    /// # Arguments
    ///
    /// * `other` - The pointer to compare against.
    #[must_use]
    pub fn starts_with(&self, other: &JsonPointer) -> bool {
        self.0 == other.0 || (self.0.starts_with(&other.0) && self.0[other.0.len()..].starts_with('/'))
    }

    /// Resolve the pointer against a document.
    ///
    /// # Arguments
    ///
    /// * `document` - The document to look in.
    ///
    /// # Returns
    ///
    /// The value at this pointer, or `None` when the document has no such location.
    #[must_use]
    pub fn resolve<'a>(&self, document: &'a Value) -> Option<&'a Value> {
        document.pointer(&self.0)
    }
}

impl Display for JsonPointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Call `visit` for every string value in a document.
///
/// # Arguments
///
/// * `value` - The value to walk.
/// * `pointer` - The pointer to `value`.
/// * `visit` - Called with the pointer and text of each string found.
pub fn for_each_string(value: &Value, pointer: &JsonPointer, visit: &mut dyn FnMut(&JsonPointer, &str)) {
    match value {
        Value::String(text) => visit(pointer, text),
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                for_each_string(item, &pointer.push_index(index), visit);
            }
        },
        Value::Object(members) => {
            for (key, member) in members {
                for_each_string(member, &pointer.push(key), visit);
            }
        },
        _ => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn push_escapes_special_characters() {
        let pointer = JsonPointer::root().push("a/b").push("c~d");
        assert_eq!(pointer.as_str(), "/a~1b/c~0d");
    }

    #[test]
    fn push_index_and_join() {
        let pointer = JsonPointer::root().push("resources").push_index(2).join("/properties/path");
        assert_eq!(pointer.to_string(), "/resources/2/properties/path");
    }

    #[test]
    fn starts_with_matches_self_and_descendants_only() {
        let base = JsonPointer::root().push("resources").push_index(1);
        assert!(base.starts_with(&base));
        assert!(base.push("name").starts_with(&base));
        assert!(base.starts_with(&JsonPointer::root()));
        assert!(!JsonPointer::root().push("resources").push_index(10).starts_with(&base));
    }

    #[test]
    fn resolve_finds_values() {
        let document = json!({"resources": [{"name": "first"}]});
        let pointer = JsonPointer::root().push("resources").push_index(0).push("name");
        assert_eq!(pointer.resolve(&document), Some(&json!("first")));
        assert_eq!(JsonPointer::root().push("missing").resolve(&document), None);
    }

    #[test]
    fn for_each_string_visits_nested_strings() {
        let document = json!({"a": ["x", {"b": "y"}], "c": 1});
        let mut found = Vec::new();
        for_each_string(&document, &JsonPointer::root(), &mut |pointer, text| found.push((pointer.to_string(), text.to_string())));
        assert_eq!(found, vec![("/a/0".to_string(), "x".to_string()), ("/a/1/b".to_string(), "y".to_string())]);
    }
}
