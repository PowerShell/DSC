// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use super::Function;

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

        let result = combine_uri(base_uri, relative_uri)?;
        Ok(Value::String(result))
    }
}

fn combine_uri(base_uri: &str, relative_uri: &str) -> Result<String, DscError> {
    if base_uri.is_empty() {
        return Err(DscError::Parser(t!("functions.uri.emptyBaseUri").to_string()));
    }
    
    if !base_uri.contains("://") && !base_uri.starts_with("//") {
        return Err(DscError::Parser(t!("functions.uri.notAbsoluteUri").to_string()));
    }
    
    if relative_uri.is_empty() {
        return Ok(base_uri.to_string());
    }

    if relative_uri.starts_with("///") {
        return Err(DscError::Parser(t!("functions.uri.invalidRelativeUri").to_string()));
    }

    if let Some(uri_without_slashes) = relative_uri.strip_prefix("//") {
        // Protocol-relative URI: extract scheme from base
        let scheme = if let Some(scheme_end) = base_uri.find("://") {
            &base_uri[..scheme_end]
        } else {
            "https"
        };
        
        // Check if the protocol-relative URI has a path
        if uri_without_slashes.contains('/') {
            // Has a path, use as-is
            return Ok(format!("{scheme}:{relative_uri}"));
        }
        // No path specified, add trailing slash for root
        return Ok(format!("{scheme}:{relative_uri}/"));
    }

    let relative_starts_with_slash = relative_uri.starts_with('/');

    // Extract scheme and host from base URI
    let scheme_end = if base_uri.starts_with("http://") || base_uri.starts_with("https://") {
        base_uri.find("://").map_or(0, |pos| pos + 3)
    } else if base_uri.starts_with("//") {
        2
    } else {
        0
    };

    // If relative URI starts with '/', it replaces the entire path of the base URI
    // Keep only scheme://host and append the relative URI
    if relative_starts_with_slash {
        let after_scheme = &base_uri[scheme_end..];
        if let Some(first_slash_pos) = after_scheme.find('/') {
            // Base has a path, extract scheme://host only
            let scheme_and_host = &base_uri[..scheme_end + first_slash_pos];
            return Ok(format!("{scheme_and_host}{relative_uri}"));
        }
        // Base has no path (e.g., "https://example.com"), just append
        return Ok(format!("{base_uri}{relative_uri}"));
    }

    // Relative URI doesn't start with '/'
    // Standard behavior: resolve relative to base
    let base_ends_with_slash = base_uri.ends_with('/');

    if base_ends_with_slash {
        // Base ends with '/', just append
        return Ok(format!("{base_uri}{relative_uri}"));
    }

    // Base doesn't end with '/', need to remove last segment
    let after_scheme = &base_uri[scheme_end..];
    
    if let Some(last_slash_pos) = after_scheme.rfind('/') {
        // Base has a path with segments, remove last segment
        let base_without_last_segment = &base_uri[..=(scheme_end + last_slash_pos)];
        Ok(format!("{base_without_last_segment}{relative_uri}"))
    } else {
        // No path after scheme (e.g., "https://example.com")
        // Add a '/' between host and relative URI
        Ok(format!("{base_uri}/{relative_uri}"))
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
        // .NET Uri behavior: adds a slash between host and relative path
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
    fn test_uri_relative_protocol() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[uri('//example.com/', 'path')]", &Context::new()).unwrap();
        assert_eq!(result, "//example.com/path");
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
        assert!(err.to_string().contains("Invalid") || err.to_string().contains("hostname"));
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
}
