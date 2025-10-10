// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use super::Function;

#[derive(Debug, Default)]
pub struct UriComponent {}

impl Function for UriComponent {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "uriComponent".to_string(),
            description: t!("functions.uriComponent.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![vec![FunctionArgKind::String]],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let string_to_encode = args[0].as_str().unwrap();
        let result = percent_encode_uri_component(string_to_encode);
        Ok(Value::String(result))
    }
}

/// Percent-encodes a string for use as a URI component.
/// 
/// Encodes all characters except:
/// - Unreserved characters: A-Z, a-z, 0-9, -, _, ., ~
/// 
/// This follows RFC 3986 for URI component encoding.
fn percent_encode_uri_component(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 3);
    
    for byte in input.bytes() {
        match byte {
            // Unreserved characters (RFC 3986 section 2.3)
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            // Everything else gets percent-encoded
            _ => {
                result.push('%');
                result.push_str(&format!("{:02X}", byte));
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn test_uri_component_basic_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponent('hello world')]", &Context::new()).unwrap();
        assert_eq!(result, "hello%20world");
    }

    #[test]
    fn test_uri_component_special_characters() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponent('hello@example.com')]", &Context::new()).unwrap();
        assert_eq!(result, "hello%40example.com");
    }

    #[test]
    fn test_uri_component_url() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponent('https://example.com/path?query=value')]", &Context::new()).unwrap();
        assert_eq!(result, "https%3A%2F%2Fexample.com%2Fpath%3Fquery%3Dvalue");
    }

    #[test]
    fn test_uri_component_empty_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponent('')]", &Context::new()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_uri_component_unreserved_characters() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponent('ABCabc123-_.~')]", &Context::new()).unwrap();
        assert_eq!(result, "ABCabc123-_.~");
    }

    #[test]
    fn test_uri_component_reserved_characters() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponent(':/?#[]@!$&()*+,;=')]", &Context::new()).unwrap();
        assert_eq!(result, "%3A%2F%3F%23%5B%5D%40%21%24%26%28%29%2A%2B%2C%3B%3D");
    }

    #[test]
    fn test_uri_component_unicode() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponent('café')]", &Context::new()).unwrap();
        assert_eq!(result, "caf%C3%A9");
    }

    #[test]
    fn test_uri_component_query_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponent('name=John Doe&age=30')]", &Context::new()).unwrap();
        assert_eq!(result, "name%3DJohn%20Doe%26age%3D30");
    }

    #[test]
    fn test_uri_component_path_with_spaces() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponent('/path/to/my file.txt')]", &Context::new()).unwrap();
        assert_eq!(result, "%2Fpath%2Fto%2Fmy%20file.txt");
    }

    #[test]
    fn test_uri_component_nested_function() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponent(concat('hello', ' ', 'world'))]", &Context::new()).unwrap();
        assert_eq!(result, "hello%20world");
    }

    #[test]
    fn test_uri_component_email() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponent('user+tag@example.com')]", &Context::new()).unwrap();
        assert_eq!(result, "user%2Btag%40example.com");
    }

    #[test]
    fn test_uri_component_numbers_only() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponent('1234567890')]", &Context::new()).unwrap();
        assert_eq!(result, "1234567890");
    }

    #[test]
    fn test_uri_component_percent_sign() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uriComponent('100%')]", &Context::new()).unwrap();
        assert_eq!(result, "100%25");
    }
}
