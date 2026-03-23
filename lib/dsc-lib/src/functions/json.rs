// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct Json {}

impl Function for Json {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "json".to_string(),
            description: t!("functions.json.description").to_string(),
            category: vec![FunctionCategory::Object],
            min_args: 1,
            max_args: 1,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Object, FunctionArgKind::Array, FunctionArgKind::String, FunctionArgKind::Number, FunctionArgKind::Boolean],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        let json_str = args[0].as_str().unwrap();

        match serde_json::from_str(json_str) {
            Ok(value) => Ok(value),
            Err(e) => Err(DscError::Parser(format!("{}: {}", t!("functions.json.invalidJson"), e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;
    use serde_json::json;

    #[test]
    fn json_parse_object() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(r#"[json('{"name":"John","age":30}')]"#, &Context::new()).unwrap();
        
        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("name").and_then(|v| v.as_str()), Some("John"));
        assert_eq!(obj.get("age").and_then(|v| v.as_i64()), Some(30));
    }

    #[test]
    fn json_parse_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(r#"[json('[1,2,3]')]"#, &Context::new()).unwrap();
        
        assert_eq!(result, json!([1, 2, 3]));
    }

    #[test]
    fn json_parse_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(r#"[json('"hello"')]"#, &Context::new()).unwrap();
        
        assert_eq!(result, json!("hello"));
    }

    #[test]
    fn json_parse_number() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(r#"[json('42')]"#, &Context::new()).unwrap();
        
        assert_eq!(result, json!(42));
    }

    #[test]
    fn json_parse_boolean() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(r#"[json('true')]"#, &Context::new()).unwrap();
        
        assert_eq!(result, json!(true));
    }

    #[test]
    fn json_parse_null() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(r#"[json('null')]"#, &Context::new()).unwrap();
        
        assert_eq!(result, json!(null));
    }

    #[test]
    fn json_parse_nested_object() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(r#"[json('{"user":{"name":"Jane","roles":["admin","user"]}}')]"#, &Context::new()).unwrap();
        
        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        let user = obj.get("user").unwrap().as_object().unwrap();
        assert_eq!(user.get("name").and_then(|v| v.as_str()), Some("Jane"));
        
        let roles = user.get("roles").unwrap().as_array().unwrap();
        assert_eq!(roles.len(), 2);
        assert_eq!(roles[0].as_str(), Some("admin"));
    }

    #[test]
    fn json_parse_with_whitespace() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(r#"[json('  { "key" : "value" }  ')]"#, &Context::new()).unwrap();
        
        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("key").and_then(|v| v.as_str()), Some("value"));
    }

    #[test]
    fn json_invalid_string_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(r#"[json('not valid json')]"#, &Context::new());
        
        assert!(result.is_err());
    }

    #[test]
    fn json_unclosed_brace_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(r#"[json('{"key":"value"')]"#, &Context::new());
        
        assert!(result.is_err());
    }

    #[test]
    fn json_empty_string_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(r#"[json('')]"#, &Context::new());
        
        assert!(result.is_err());
    }

    #[test]
    fn json_not_string_error() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(r#"[json(123)]"#, &Context::new());
        
        assert!(result.is_err());
    }
}
