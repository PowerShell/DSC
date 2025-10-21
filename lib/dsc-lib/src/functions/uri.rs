// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use super::Function;
use url::Url;

#[derive(Debug, Default)]
pub struct Uri {}

impl Function for Uri {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "uri".to_string(),
            description: t!("functions.uri.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String],
                vec![FunctionArgKind::String],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let base_uri = args[0].as_str().unwrap();
        let relative_uri = args[1].as_str().unwrap();

        if base_uri.is_empty() {
            return Err(DscError::Parser(t!("functions.uri.emptyBaseUri").to_string()));
        }
        
        let base = Url::parse(base_uri)
            .map_err(|_| DscError::Parser(t!("functions.uri.notAbsoluteUri").to_string()))?;
        
        if relative_uri.is_empty() {
            return Ok(Value::String(base.to_string()));
        }

        if relative_uri.starts_with("///") {
            return Err(DscError::Parser(t!("functions.uri.invalidRelativeUri").to_string()));
        }

        let result = base.join(relative_uri)
            .map_err(|e| DscError::Parser(format!("{}: {}", t!("functions.uri.invalidRelativeUri"), e)))?;
        
        Ok(Value::String(result.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn test_uri_basic_trailing_slash() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com/', 'path/file.html')]", &Context::new()).unwrap();
        assert_eq!(result, "https://example.com/path/file.html");
    }

    #[test]
    fn test_uri_trailing_and_leading_slash() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com/', '/path/file.html')]", &Context::new()).unwrap();
        assert_eq!(result, "https://example.com/path/file.html");
    }

    #[test]
    fn test_uri_no_trailing_slash_with_path() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com/api/v1', 'users')]", &Context::new()).unwrap();
        assert_eq!(result, "https://example.com/api/users");
    }

    #[test]
    fn test_uri_no_trailing_slash_with_leading_slash() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com/api/v1', '/users')]", &Context::new()).unwrap();
        // When relative starts with '/', it replaces the entire path
        assert_eq!(result, "https://example.com/users");
    }

    #[test]
    fn test_uri_no_slashes_after_scheme() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com', 'path')]", &Context::new()).unwrap();
        assert_eq!(result, "https://example.com/path");
    }

    #[test]
    fn test_uri_no_slashes_after_scheme_with_leading_slash() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com', '/path')]", &Context::new()).unwrap();
        assert_eq!(result, "https://example.com/path");
    }

    #[test]
    fn test_uri_complex_path() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://api.example.com/v2/resource/', 'item/123')]", &Context::new()).unwrap();
        assert_eq!(result, "https://api.example.com/v2/resource/item/123");
    }

    #[test]
    fn test_uri_query_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com/api/', 'search?q=test')]", &Context::new()).unwrap();
        assert_eq!(result, "https://example.com/api/search?q=test");
    }

    #[test]
    fn test_uri_empty_relative() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com/', '')]", &Context::new()).unwrap();
        assert_eq!(result, "https://example.com/");
    }

    #[test]
    fn test_uri_http_scheme() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('http://example.com/', 'page.html')]", &Context::new()).unwrap();
        assert_eq!(result, "http://example.com/page.html");
    }

    #[test]
    fn test_uri_with_port() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com:8080/', 'api')]", &Context::new()).unwrap();
        assert_eq!(result, "https://example.com:8080/api");
    }

    #[test]
    fn test_uri_nested_function() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri(concat('https://example.com', '/'), 'path')]", &Context::new()).unwrap();
        assert_eq!(result, "https://example.com/path");
    }

    #[test]
    fn test_uri_multiple_path_segments() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com/a/b/c/', 'd/e/f')]", &Context::new()).unwrap();
        assert_eq!(result, "https://example.com/a/b/c/d/e/f");
    }

    #[test]
    fn test_uri_replace_last_segment() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com/old/path', 'new')]", &Context::new()).unwrap();
        assert_eq!(result, "https://example.com/old/new");
    }

    #[test]
    fn test_uri_empty_base_uri_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('', 'path')]", &Context::new());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("baseUri"));
    }

    #[test]
    fn test_uri_triple_slash_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com/', '///foo')]", &Context::new());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid") || err.to_string().contains("invalid"));
    }

    #[test]
    fn test_uri_double_slash_protocol_relative() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com/', '//foo')]", &Context::new()).unwrap();
        assert_eq!(result, "https://foo/");
    }

    #[test]
    fn test_uri_not_absolute_no_scheme() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('example.com', 'path')]", &Context::new());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("absolute"));
    }

    #[test]
    fn test_uri_not_absolute_relative_path() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('/relative/path', 'file.txt')]", &Context::new());
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("absolute"));
    }

    #[test]
    fn test_uri_double_slash_with_path() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://example.com/', '//foo/bar')]", &Context::new()).unwrap();
        assert_eq!(result, "https://foo/bar");
    }

    #[test]
    fn test_uri_ipv6_localhost() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://[::1]/', 'path')]", &Context::new()).unwrap();
        // IPv6 uses compressed format (standard representation)
        assert_eq!(result, "https://[::1]/path");
    }

    #[test]
    fn test_uri_ipv6_address() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://[2001:db8::1]/', 'api/v1')]", &Context::new()).unwrap();
        // IPv6 uses compressed format (standard representation)
        assert_eq!(result, "https://[2001:db8::1]/api/v1");
    }

    #[test]
    fn test_uri_ipv6_with_port() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('https://[2001:db8::1]:8080/', 'api')]", &Context::new()).unwrap();
        assert_eq!(result, "https://[2001:db8::1]:8080/api");
    }

    #[test]
    fn test_uri_ipv4_address() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('http://192.168.1.1/', 'api/v1')]", &Context::new()).unwrap();
        assert_eq!(result, "http://192.168.1.1/api/v1");
    }
}
