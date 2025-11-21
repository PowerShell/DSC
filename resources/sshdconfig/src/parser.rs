// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use schemars::JsonSchema;
use serde_json::{Map, Value};
use tracing::debug;
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
            return Err(SshdConfigError::ParserError(t!("parser.failedToParse", input = input).to_string()));
        };
        let root_node = tree.root_node();
        if root_node.is_error() {
            return Err(SshdConfigError::ParserError(t!("parser.failedToParseRoot", input = input).to_string()));
        }
        if root_node.kind() != "server_config" {
            return Err(SshdConfigError::ParserError(t!("parser.invalidConfig", input = input).to_string()));
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
            return Err(SshdConfigError::ParserError(t!("parser.failedToParse", input = input).to_string()));
        }
        match node.kind() {
            "keyword" => self.parse_keyword_node(node, input, input_bytes),
            "comment" | "empty_line" | "match" => Ok(()), // TODO: do not ignore match nodes when parsing
            _ => Err(SshdConfigError::ParserError(t!("parser.unknownNodeType", node = node.kind()).to_string())),
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
                return Err(SshdConfigError::ParserError(
                    t!("parser.failedToParseChildNode", input = input).to_string()
                ));
            };
            debug!("{}", t!("parser.keywordDebug", text = text).to_string());
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
                return Err(SshdConfigError::ParserError(t!("parser.failedToParseChildNode", input = input).to_string()));
            }
            if node.kind() == "arguments" {
                value = parse_arguments_node(node, input, input_bytes, is_vec)?;
                debug!("{}: {:?}", t!("parser.valueDebug").to_string(), value);
            }
        }
        if let Some(key) = key {
            if value.is_null() {
                return Err(SshdConfigError::ParserError(t!("parser.missingValueInChildNode", input = input).to_string()));
            }
            return self.update_map(&key, value, is_repeatable);
        }
        Err(SshdConfigError::ParserError(t!("parser.missingKeyInChildNode", input = input).to_string()))
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
                                t!("parser.failedToParseAsArray").to_string()
                            ));
                        }
                    } else {
                        return Err(SshdConfigError::ParserError(
                            t!("parser.failedToParseAsArray").to_string()
                        ));
                    }
                } else {
                    return Err(SshdConfigError::ParserError(t!("parser.keyNotFound", key = key).to_string()));
                }
            } else {
                return Err(SshdConfigError::ParserError(t!("parser.keyNotRepeatable", key = key).to_string()));
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

    // if there is more than one argument, but a vector is not expected for the keyword, throw an error
    let children: Vec<_> = arg_node.named_children(&mut cursor).collect();
    if children.len() > 1 && !is_vec {
        return Err(SshdConfigError::ParserError(t!("parser.invalidMultiArgNode", input = input).to_string()));
    }

    for node in children {
        if node.is_error() {
            return Err(SshdConfigError::ParserError(t!("parser.failedToParseChildNode", input = input).to_string()));
        }
        let argument: Value = match node.kind() {
            "boolean" | "string" | "quotedString" => {
                let Ok(arg) = node.utf8_text(input_bytes) else {
                    return Err(SshdConfigError::ParserError(
                        t!("parser.failedToParseNode", input = input).to_string()
                    ));
                };
                Value::String(arg.trim().to_string())
            }
            "number" => {
                let Ok(arg) = node.utf8_text(input_bytes) else {
                    return Err(SshdConfigError::ParserError(
                        t!("parser.failedToParseNode", input = input).to_string()
                    ));
                };
                Value::Number(arg.parse::<u64>()?.into())
            }
            "operator" => {
                // TODO: handle operator if not parsing from SSHD -T
                return Err(SshdConfigError::ParserError(
                    t!("parser.invalidValue").to_string()
                ));
            },
            _ => return Err(SshdConfigError::ParserError(t!("parser.unknownNode", kind = node.kind()).to_string()))
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
    let lowercased_map = parser.map.into_iter()
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect();
    Ok(lowercased_map)
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
    fn multiarg_string_with_spaces_keyword() {
        let input = "allowgroups administrators \"openssh users\"\r\n";
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let expected = vec![
            Value::String("administrators".to_string()),
            Value::String("\"openssh users\"".to_string()),
        ];
        assert_eq!(result.get("allowgroups").unwrap(), &Value::Array(expected));
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
