// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use super::Function;

#[derive(Debug, Default)]
pub struct UriComponentToString {}

impl Function for UriComponentToString {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "uriComponentToString".to_string(),
            description: t!("functions.uriComponentToString.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let uri_encoded_string = args[0].as_str().unwrap();
        let result = urlencoding::decode(uri_encoded_string)
            .map_err(|e| DscError::Parser(
                t!("functions.uriComponentToString.invalidUtf8", error = e).to_string()
            ))?;
        Ok(Value::String(result.into_owned()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn test_uri_component_to_string_basic() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponentToString('hello%20world')]", &Context::new()).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_uri_component_to_string_email() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponentToString('hello%40example.com')]", &Context::new()).unwrap();
        assert_eq!(result, "hello@example.com");
    }

    #[test]
    fn test_uri_component_to_string_url() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponentToString('https%3A%2F%2Fexample.com%2Fpath%3Fquery%3Dvalue')]", &Context::new()).unwrap();
        assert_eq!(result, "https://example.com/path?query=value");
    }

    #[test]
    fn test_uri_component_to_string_empty() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponentToString('')]", &Context::new()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_uri_component_to_string_no_encoding() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponentToString('ABCabc123-_.~')]", &Context::new()).unwrap();
        assert_eq!(result, "ABCabc123-_.~");
    }

    #[test]
    fn test_uri_component_to_string_reserved_chars() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponentToString('%3A%2F%3F%23%5B%5D%40%21%24%26%28%29%2A%2B%2C%3B%3D')]", &Context::new()).unwrap();
        assert_eq!(result, ":/?#[]@!$&()*+,;=");
    }

    #[test]
    fn test_uri_component_to_string_unicode() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponentToString('caf%C3%A9')]", &Context::new()).unwrap();
        assert_eq!(result, "café");
    }

    #[test]
    fn test_uri_component_to_string_query_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponentToString('name%3DJohn%20Doe%26age%3D30')]", &Context::new()).unwrap();
        assert_eq!(result, "name=John Doe&age=30");
    }

    #[test]
    fn test_uri_component_to_string_path() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponentToString('%2Fpath%2Fto%2Fmy%20file.txt')]", &Context::new()).unwrap();
        assert_eq!(result, "/path/to/my file.txt");
    }

    #[test]
    fn test_uri_component_to_string_roundtrip() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponentToString(uriComponent('hello world'))]", &Context::new()).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_uri_component_to_string_roundtrip_email() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponentToString(uriComponent('user+tag@example.com'))]", &Context::new()).unwrap();
        assert_eq!(result, "user+tag@example.com");
    }

    #[test]
    fn test_uri_component_to_string_percent_sign() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponentToString('100%25')]", &Context::new()).unwrap();
        assert_eq!(result, "100%");
    }

    #[test]
    fn test_uri_component_to_string_mixed() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponentToString('hello%20world%21')]", &Context::new()).unwrap();
        assert_eq!(result, "hello world!");
    }
}
