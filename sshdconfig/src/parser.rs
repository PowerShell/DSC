// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde_json::{Map, Value};
use tree_sitter::Parser;

use crate::error::SshdConfigError;
use crate::metadata::{MULTI_ARG_KEYWORDS, REPEATABLE_KEYWORDS};

#[derive(Debug, JsonSchema)]
pub struct SshdConfigParser {
    map: Map<String, Value>
}

impl SshdConfigParser {
    /// Create a new `SshdConfigParser` instance.
    pub fn new() -> Self {
        Self {
            map: Map::new()
        }
    }

    /// Parse `sshd_config` to map.
    ///
    /// # Arguments
    ///
    /// * `input` - The `sshd_config` text to parse, from sshd -T.
    ///
    /// # Errors
    ///
    /// This function will return an error if the parser fails to initalize or the input fails to parse.
    pub fn parse_text(&mut self, input: &str) -> Result<(), SshdConfigError> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_ssh_server_config::LANGUAGE.into())?;

        let Some(tree) = &mut parser.parse(input, None) else {
            return Err(SshdConfigError::ParserError(format!("failed to parse: {input}")));
        };
        let root_node = tree.root_node();
        if root_node.is_error() {
            return Err(SshdConfigError::ParserError(format!("failed to parse root: {input}")));
        }
        if root_node.kind() != "server_config" {
            return Err(SshdConfigError::ParserError(format!("invalid config: {input}")));
        }
        let input_bytes = input.as_bytes();
        let mut cursor = root_node.walk();
        for child_node in root_node.named_children(&mut cursor) {
            self.parse_child_node(child_node, input, input_bytes)?;
        }
        Ok(())
    }

    fn parse_child_node(&mut self, node: tree_sitter::Node, input: &str, input_bytes: &[u8]) -> Result<(), SshdConfigError> {
        if node.is_error() {
            return Err(SshdConfigError::ParserError(format!("failed to parse: {input}")));
        }
        match node.kind() {
            "keyword" => self.parse_keyword_node(node, input, input_bytes),
            "comment" | "empty_line" => Ok(()),
            _ => Err(SshdConfigError::ParserError(format!("unknown node type: {}", node.kind()))),
        }
    }

    fn parse_keyword_node(&mut self, keyword_node: tree_sitter::Node, input: &str, input_bytes: &[u8]) -> Result<(), SshdConfigError> {
        let mut cursor = keyword_node.walk();
        let mut key = None;
        let mut value = Value::Null;
        let mut is_repeatable = false;
        let mut is_vec = false;

        if let Some(keyword) = keyword_node.child_by_field_name("keyword") {
            let Ok(text) = keyword.utf8_text(input_bytes) else {
                return Err(SshdConfigError::ParserError(format!(
                    "failed to parse keyword node: {input}"
                )));
            };
            if REPEATABLE_KEYWORDS.contains(&text) {
                is_repeatable = true;
                is_vec = true;
            } else if MULTI_ARG_KEYWORDS.contains(&text) {
                is_vec = true;
            }
            key = Some(text.to_string());
        }

        for node in keyword_node.named_children(&mut cursor) {
            if node.is_error() {
                return Err(SshdConfigError::ParserError(format!("failed to parse child node: {input}")));
            }
            if node.kind() == "arguments" {
                value = parse_arguments_node(node, input, input_bytes, is_vec)?;
            }
        }
        if let Some(key) = key {
            if value.is_null() {
                return Err(SshdConfigError::ParserError(format!("missing value in child node: {input}")));
            }
            return self.update_map(&key, value, is_repeatable);
        }
        Err(SshdConfigError::ParserError(format!("missing key in child node: {input}")))
    }

    fn update_map(&mut self, key: &str, value: Value, is_repeatable: bool) -> Result<(), SshdConfigError> {
        if self.map.contains_key(key) {
            if is_repeatable {
                let existing_value = self.map.get_mut(key);
                if let Some(existing_value) = existing_value {
                    if let Value::Array(ref mut arr) = existing_value {
                        if let Value::Array(vector) = value {
                            for v in vector {
                                arr.push(v);
                            }
                        } else {
                            return Err(SshdConfigError::ParserError(
                                "value is not an array".to_string(),
                            ));
                        }
                    } else {
                        return Err(SshdConfigError::ParserError(
                            "value is not an array".to_string(),
                        ));
                    }
                } else {
                    return Err(SshdConfigError::ParserError(format!("key {key} not found")));
                }
            } else {
                return Err(SshdConfigError::ParserError(format!("key {key} is not repeatable")));
            }
        } else {
            self.map.insert(key.to_string(), value);
        }
        Ok(())
    }
}

fn parse_arguments_node(arg_node: tree_sitter::Node, input: &str, input_bytes: &[u8], is_vec: bool) -> Result<Value, SshdConfigError> {
    let mut cursor = arg_node.walk();
    let mut vec: Vec<Value> = Vec::new();
    let mut value = Value::Null;
    for node in arg_node.named_children(&mut cursor) {

        if node.is_error() {
            return Err(SshdConfigError::ParserError(format!("failed to parse child node: {input}")));
        }
        let argument: Value = match node.kind() {
            "boolean" | "string" => {
                let Ok(arg) = node.utf8_text(input_bytes) else {
                    return Err(SshdConfigError::ParserError(format!(
                        "failed to parse string node: {input}"
                    )));
                };
                Value::String(arg.to_string())
            }
            "number" => {
                let Ok(arg) = node.utf8_text(input_bytes) else {
                    return Err(SshdConfigError::ParserError(format!(
                        "failed to parse string node: {input}"
                    )));
                };
                Value::Number(arg.parse::<u64>()?.into())
            }
            "operator" => {
                // TODO: handle operator if not parsing from SSHD -T
                return Err(SshdConfigError::ParserError(format!(
                    "todo - unsuported node: {}", node.kind()
                )));
            }
            _ => return Err(SshdConfigError::ParserError(format!("unknown node: {}", node.kind())))
        };
        if is_vec {
            vec.push(argument);
        } else {
            value = argument;
        }
    }
    if is_vec {
        Ok(Value::Array(vec))
    } else{
        Ok(value)
    }
}

/// Parse `sshd_config` to map.
///
/// # Arguments
///
/// * `input` - The `sshd_config` text to parse.
///
/// # Errors
///
/// This function will return an error if the input fails to parse.
pub fn parse_text_to_map(input: &str) -> Result<Map<String,Value>, SshdConfigError> {
    let mut parser = SshdConfigParser::new();
    parser.parse_text(input)?;
    Ok(parser.map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeatable_numeric_keyword() {
        let input = r#"
          port 1234
          port 5678
        "#.trim_start();
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let expected = vec![Value::Number(1234.into()), Value::Number(5678.into())];
        assert_eq!(result.get("port").unwrap(), &Value::Array(expected));
    }

    #[test]
    fn multiarg_string_keyword() {
        let input = "hostkeyalgorithms ssh-ed25519-cert-v01@openssh.com,ecdsa-sha2-nistp256-cert-v01@openssh.com\r\n";
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let expected = vec![
            Value::String("ssh-ed25519-cert-v01@openssh.com".to_string()),
            Value::String("ecdsa-sha2-nistp256-cert-v01@openssh.com".to_string()),
        ];
        assert_eq!(result.get("hostkeyalgorithms").unwrap(), &Value::Array(expected));
    }

    #[test]
    fn bool_keyword() {
        let input = "printmotd yes\r\n";
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        assert_eq!(result.get("printmotd").unwrap(), &Value::String("yes".to_string()));
    }

    #[test]
    fn err_multiarg_repeated_keyword() {
        let input = "hostkeyalgorithms ssh-ed25519-cert-v01@openssh.com\r\n hostkeyalgorithms ecdsa-sha2-nistp256-cert-v01@openssh.com\r\n";
        let result = parse_text_to_map(input);
        assert!(result.is_err());
    }

    #[test]
    fn empty_string_is_ok() {
        let code = r#"
        "#;
        let result = parse_text_to_map(code);
        assert!(result.is_ok());
    }
}
