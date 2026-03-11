// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use rust_i18n::t;
use schemars::JsonSchema;
use serde_json::{Map, Value};
use tracing::debug;
use tree_sitter::Parser;

use crate::error::SshdConfigError;
use crate::repeat_keyword::{KeywordInfo, ValueSeparator};

/// Unescape backslashes in strings. sshd -T outputs paths with escaped backslashes
/// (e.g., "c:\\openssh\\bin\\sftp.exe") but we want to normalize them to the original
/// form (e.g., "c:\openssh\bin\sftp.exe") for comparison and storage.
#[cfg(windows)]
fn unescape_backslashes(s: &str) -> String {
    s.replace("\\\\", "\\")
}

#[cfg(not(windows))]
fn unescape_backslashes(s: &str) -> String {
    s.to_string()
}

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
            "comment" => Ok(()),
            "keyword" => {
                Self::parse_and_insert_keyword(node, input, input_bytes, Some(&mut self.map))?;
                Ok(())
            }
            "match" => self.parse_match_node(node, input, input_bytes),
            _ => Err(SshdConfigError::ParserError(t!("parser.unknownNodeType", node = node.kind()).to_string())),
        }
    }

    fn parse_match_node(&mut self, match_node: tree_sitter::Node, input: &str, input_bytes: &[u8]) -> Result<(), SshdConfigError> {
        let mut criteria_map = Map::new();
        let mut cursor = match_node.walk();
        let mut match_object = Map::new();

        for child_node in match_node.named_children(&mut cursor) {
            if child_node.is_error() {
                return Err(SshdConfigError::ParserError(t!("parser.failedToParseNode", input = input).to_string()));
            }

            match child_node.kind() {
                "comment" => {}
                "criteria" => {
                    Self::parse_match_criteria(child_node, input, input_bytes, &mut criteria_map)?;
                }
                "keyword" => {
                    Self::parse_and_insert_keyword(child_node, input, input_bytes, Some(&mut match_object))?;
                }
                _ => {
                    return Err(SshdConfigError::ParserError(t!("parser.unknownNodeType", node = child_node.kind()).to_string()));
                }
            }
        }

        // Add the match object to the main map
        if criteria_map.is_empty() {
            return Err(SshdConfigError::ParserError(t!("parser.missingCriteriaInMatch", input = input).to_string()));
        }
        match_object.insert("criteria".to_string(), Value::Object(criteria_map));
        Self::insert_into_map(&mut self.map, "match", Value::Object(match_object), true)?;
        Ok(())
    }

    /// Parse a single match criteria node and insert into the provided criteria map.
    /// Example criteria node: "user alice,bob" or "address *.*.0.1"
    /// Inserts the criterion as a key with an array value into the `criteria_map`.
    fn parse_match_criteria(criteria_node: tree_sitter::Node, input: &str, input_bytes: &[u8], criteria_map: &mut Map<String, Value>) -> Result<(), SshdConfigError> {
        if let Some(key_node) = criteria_node.child_by_field_name("keyword") {
            let Ok(key_text) = key_node.utf8_text(input_bytes) else {
                return Err(SshdConfigError::ParserError(t!("parser.failedToParseNode", input = input).to_string()));
            };
            let key = key_text.to_string();

            let values: Value;
            if let Some(value_node) = criteria_node.child_by_field_name("argument") {
                // Match criteria are always treated as arrays (comma-separated), so we override
                // the keyword_info to force multi-arg behavior
                let mut keyword_info = KeywordInfo::from_keyword(&key);
                // Force comma separator and multi-arg for match criteria
                keyword_info.separator = ValueSeparator::Comma;
                values = parse_arguments_node_as_array(value_node, input, input_bytes, &keyword_info)?;
            }
            else {
                return Err(SshdConfigError::ParserError(t!("parser.missingValueInCriteria", input = input).to_string()));
            }

            criteria_map.insert(key.to_lowercase(), values);
            Ok(())
        } else {
            Err(SshdConfigError::ParserError(t!("parser.missingKeyInCriteria", input = input).to_string()))
        }
    }

    /// Parse a keyword node and optionally insert it into a map.
    /// If `target_map` is provided, the keyword will be inserted into that map with repeatability handling.
    /// If `target_map` is None, returns the key-value pair without inserting.
    fn parse_and_insert_keyword(
        keyword_node: tree_sitter::Node,
        input: &str,
        input_bytes: &[u8],
        target_map: Option<&mut Map<String, Value>>
    ) -> Result<(String, Value), SshdConfigError> {
        let mut cursor = keyword_node.walk();
        let mut key = None;
        let mut value = Value::Null;
        let mut keyword_info: Option<KeywordInfo> = None;
        let mut operator: Option<String> = None;

        if let Some(keyword) = keyword_node.child_by_field_name("keyword") {
            let Ok(text) = keyword.utf8_text(input_bytes) else {
                return Err(SshdConfigError::ParserError(t!("parser.failedToParseNode", input = input).to_string()));
            };
            let info = KeywordInfo::from_keyword(text);
            keyword_info = Some(info);
            key = Some(text.to_string());
        }

        // Check for operator field
        if let Some(operator_node) = keyword_node.child_by_field_name("operator") {
            let Ok(op_text) = operator_node.utf8_text(input_bytes) else {
                return Err(
                    SshdConfigError::ParserError(t!("parser.failedToParseNode", input = input).to_string())
                );
            };
            operator = Some(op_text.to_string());
        }

        // Validate that structured keywords cannot use operators
        if let Some(ref info) = keyword_info {
            if !info.allows_operator() && operator.is_some() {
                return Err(SshdConfigError::ParserError(
                    t!("parser.structuredKeywordCannotUseOperator", keyword = &info.name).to_string()
                ));
            }
        }

        for node in keyword_node.named_children(&mut cursor) {
            if node.is_error() {
                return Err(SshdConfigError::ParserError(t!("parser.failedToParseNode", input = input).to_string()));
            }
            if node.kind() == "arguments" {
                if let Some(ref info) = keyword_info {
                    value = parse_arguments_node(node, input, input_bytes, info)?;
                    debug!("{}: {:?}", t!("parser.valueDebug").to_string(), value);
                }
            }
        }

        // If operator is present, wrap value in a nested map
        if let Some(op) = operator {
            let mut operator_map = Map::new();
            operator_map.insert("value".to_string(), value);
            operator_map.insert("operator".to_string(), Value::String(op));
            value = Value::Object(operator_map);
        }

        if let Some(key) = key {
            if value.is_null() {
                return Err(SshdConfigError::ParserError(t!("parser.missingValueInChildNode", input = input).to_string()));
            }

            // If target_map is provided, insert the value with repeatability handling
            if let Some(map) = target_map {
                let is_repeatable = keyword_info.as_ref().is_some_and(|i| i.is_repeatable);
                Self::insert_into_map(map, &key, value.clone(), is_repeatable)?;
            }

            return Ok((key, value));
        }
        Err(SshdConfigError::ParserError(t!("parser.missingKeyInChildNode", input = input).to_string()))
    }

    /// Insert a key-value pair into a map with repeatability handling.
    /// If the key is repeatable and already exists, append to the array.
    /// If the key is not repeatable and already exists, return an error.
    fn insert_into_map(map: &mut Map<String, Value>, key: &str, value: Value, is_repeatable: bool) -> Result<(), SshdConfigError> {
        if map.contains_key(key) {
            if is_repeatable {
                let existing_value = map.get_mut(key);
                if let Some(existing_value) = existing_value {
                    if let Value::Array(ref mut arr) = existing_value {
                        if let Value::Array(vector) = value {
                            for v in vector {
                                arr.push(v);
                            }
                        } else {
                            arr.push(value);
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
        } else if is_repeatable {
            // Initialize repeatable keywords as arrays
            if value.is_array() {
                map.insert(key.to_string(), value);
            } else {
                map.insert(key.to_string(), Value::Array(vec![value]));
            }
        } else {
            map.insert(key.to_string(), value);
        }
        Ok(())
    }
}

fn parse_arguments_node(arg_node: tree_sitter::Node, input: &str, input_bytes: &[u8], keyword_info: &KeywordInfo) -> Result<Value, SshdConfigError> {
    let mut cursor = arg_node.walk();
    let mut vec: Vec<Value> = Vec::new();
    let is_vec = keyword_info.is_multi_arg();

    // if there is more than one argument, but a vector is not expected for the keyword, throw an error
    let children: Vec<_> = arg_node.named_children(&mut cursor).collect();
    if children.len() > 1 && !is_vec {
        return Err(SshdConfigError::ParserError(t!("parser.invalidMultiArgNode", input = input).to_string()));
    }

    for node in &children {
        if node.is_error() {
            return Err(SshdConfigError::ParserError(t!("parser.failedToParseNode", input = input).to_string()));
        }
        let arg = node.utf8_text(input_bytes)?;
        match node.kind() {
            "boolean" => {
                let arg_str = arg.trim();
                vec.push(Value::Bool(arg_str.eq_ignore_ascii_case("yes")));
            }
            "string" => {
                let arg_str = arg.trim();
                // Unescape backslashes (sshd -T escapes them on Windows)
                let unescaped = unescape_backslashes(arg_str);
                vec.push(Value::String(unescaped));
            },
            "number" => {
                vec.push(Value::Number(arg.parse::<u64>()?.into()));
            },
            _ => return Err(SshdConfigError::ParserError(t!("parser.unknownNode", kind = node.kind()).to_string()))
        }
    }

    // Handle structured keywords (e.g., subsystem)
    if keyword_info.requires_structured_format() {
        if vec.len() < 2 {
            return Err(SshdConfigError::ParserError(
                t!("parser.structuredKeywordRequiresMinArgs", keyword = &keyword_info.name).to_string()
            ));
        }

        // Extract name (first element) and value (remaining elements joined with space)
        let name = vec[0].clone();
        let value_parts: Vec<String> = vec[1..].iter()
            .filter_map(|v| {
                match v {
                    Value::String(s) => Some(s.clone()),
                    Value::Number(n) => Some(n.to_string()),
                    Value::Bool(b) => Some(if *b { "yes".to_string() } else { "no".to_string() }),
                    _ => None
                }
            })
            .collect();
        let value_str = value_parts.join(" ");

        let mut structured = Map::new();
        structured.insert("name".to_string(), name);
        structured.insert("value".to_string(), Value::String(value_str));
        return Ok(Value::Object(structured));
    }

    // Always return array if is_vec is true (for MULTI_ARG_KEYWORDS, and REPEATABLE_KEYWORDS)
    if is_vec {
        Ok(Value::Array(vec))
    } else if !vec.is_empty() {
        Ok(vec[0].clone())
    } else { /* shouldn't happen */
        Err(SshdConfigError::ParserError(t!("parser.noArgumentsFound", input = input).to_string()))
    }
}

/// Parse arguments node for match criteria, always returning an array.
/// Match criteria are always comma-separated and should be arrays.
fn parse_arguments_node_as_array(arg_node: tree_sitter::Node, input: &str, input_bytes: &[u8], _keyword_info: &KeywordInfo) -> Result<Value, SshdConfigError> {
    let mut cursor = arg_node.walk();
    let mut vec: Vec<Value> = Vec::new();

    let children: Vec<_> = arg_node.named_children(&mut cursor).collect();

    for node in &children {
        if node.is_error() {
            return Err(SshdConfigError::ParserError(t!("parser.failedToParseNode", input = input).to_string()));
        }
        let arg = node.utf8_text(input_bytes)?;
        match node.kind() {
            "boolean" => {
                let arg_str = arg.trim();
                vec.push(Value::Bool(arg_str.eq_ignore_ascii_case("yes")));
            }
            "string" => {
                let arg_str = arg.trim();
                // Unescape backslashes (sshd -T escapes them on Windows)
                let unescaped = unescape_backslashes(arg_str);
                vec.push(Value::String(unescaped));
            },
            "number" => {
                vec.push(Value::Number(arg.parse::<u64>()?.into()));
            },
            _ => return Err(SshdConfigError::ParserError(t!("parser.unknownNode", kind = node.kind()).to_string()))
        }
    }

    // Always return array for match criteria
    Ok(Value::Array(vec))
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
    fn subsystem_keyword() {
        let input = r#"
          subsystem sftp /usr/bin/sftp
          subsystem pwsh c:/progra~1/powershell/7/pwsh.exe -sshs -NoLogo -NoProfile
        "#.trim_start();
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let subsystems = result.get("subsystem").unwrap().as_array().unwrap();
        assert_eq!(subsystems.len(), 2);

        // Check first subsystem
        let sftp = subsystems[0].as_object().unwrap();
        assert_eq!(sftp.get("name").unwrap(), &Value::String("sftp".to_string()));
        assert_eq!(sftp.get("value").unwrap(), &Value::String("/usr/bin/sftp".to_string()));

        // Check second subsystem with multiple arguments
        let pwsh = subsystems[1].as_object().unwrap();
        assert_eq!(pwsh.get("name").unwrap(), &Value::String("pwsh".to_string()));
        assert_eq!(pwsh.get("value").unwrap(), &Value::String("c:/progra~1/powershell/7/pwsh.exe -sshs -NoLogo -NoProfile".to_string()));
    }

    #[test]
    fn subsystem_single_entry() {
        let input = "subsystem sftp /usr/lib/openssh/sftp-server\n";
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let subsystems = result.get("subsystem").unwrap().as_array().unwrap();
        assert_eq!(subsystems.len(), 1);

        let sftp = subsystems[0].as_object().unwrap();
        assert_eq!(sftp.get("name").unwrap(), &Value::String("sftp".to_string()));
        assert_eq!(sftp.get("value").unwrap(), &Value::String("/usr/lib/openssh/sftp-server".to_string()));
    }

    #[test]
    fn subsystem_name_only_should_error() {
        let input = "subsystem sftp\n";
        let result = parse_text_to_map(input);
        assert!(result.is_err());
    }

    #[test]
    fn subsystem_with_operator_should_error() {
        let input = "subsystem +sftp /usr/bin/sftp\n";
        let result = parse_text_to_map(input);
        assert!(result.is_err());
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
            Value::String("openssh users".to_string()),
        ];
        assert_eq!(result.get("allowgroups").unwrap(), &Value::Array(expected));
    }

    #[test]
    fn bool_keyword() {
        let input = "printmotd yes\r\n";
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        assert_eq!(result.get("printmotd").unwrap(), &Value::Bool(true));
    }

    #[test]
    fn multiarg_string_with_spaces_no_quotes_keyword() {
        let input = "allowgroups administrators developers\n";
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let allowgroups = result.get("allowgroups").unwrap().as_array().unwrap();
        assert_eq!(allowgroups.len(), 2);
        assert_eq!(allowgroups[0], Value::String("administrators".to_string()));
        assert_eq!(allowgroups[1], Value::String("developers".to_string()));
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

    #[test]
    fn keyword_with_operator_variations() {
        let input = r#"
ciphers +aes256-ctr
macs -hmac-md5
kexalgorithms ^ecdh-sha2-nistp256
"#;
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();

        let ciphers = result.get("ciphers").unwrap().as_object().unwrap();
        assert_eq!(ciphers.get("operator").unwrap(), &Value::String("+".to_string()));
        assert!(ciphers.get("value").unwrap().is_array());

        let macs = result.get("macs").unwrap().as_object().unwrap();
        assert_eq!(macs.get("operator").unwrap(), &Value::String("-".to_string()));
        assert!(macs.get("value").unwrap().is_array());

        let kex = result.get("kexalgorithms").unwrap().as_object().unwrap();
        assert_eq!(kex.get("operator").unwrap(), &Value::String("^".to_string()));
        assert!(kex.get("value").unwrap().is_array());
    }

    #[test]
    fn keyword_with_operator_multiple_values() {
        let input = r#"
ciphers +aes256-ctr,aes128-ctr
"#;
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let ciphers = result.get("ciphers").unwrap().as_object().unwrap();
        let value_array = ciphers.get("value").unwrap().as_array().unwrap();
        assert_eq!(value_array.len(), 2);
        assert_eq!(value_array[0], Value::String("aes256-ctr".to_string()));
        assert_eq!(value_array[1], Value::String("aes128-ctr".to_string()));
        assert_eq!(ciphers.get("operator").unwrap(), &Value::String("+".to_string()));
    }

    #[test]
    fn single_match_block() {
        let input = r#"
port 22
match user bob
    gssapiauthentication yes
    allowtcpforwarding yes
"#;
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let match_array = result.get("match").unwrap().as_array().unwrap();
        assert_eq!(match_array.len(), 1);
        let match_obj = match_array[0].as_object().unwrap();
        let criteria = match_obj.get("criteria").unwrap().as_object().unwrap();
        let user_array = criteria.get("user").unwrap().as_array().unwrap();
        assert_eq!(user_array[0], Value::String("bob".to_string()));
        assert_eq!(match_obj.get("gssapiauthentication").unwrap(), &Value::Bool(true));
        assert_eq!(match_obj.get("allowtcpforwarding").unwrap(), &Value::Bool(true));
    }

    #[test]
    fn multiple_match_blocks() {
        let input = r#"
match user alice
    passwordauthentication yes
match group administrators
    permitrootlogin yes
"#;
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let match_array = result.get("match").unwrap().as_array().unwrap();
        assert_eq!(match_array.len(), 2);
        let match_obj1 = match_array[0].as_object().unwrap();
        let criteria1 = match_obj1.get("criteria").unwrap().as_object().unwrap();
        let user_array1 = criteria1.get("user").unwrap().as_array().unwrap();
        assert_eq!(user_array1[0], Value::String("alice".to_string()));
        assert_eq!(match_obj1.get("passwordauthentication").unwrap(), &Value::Bool(true));
        let match_obj2 = match_array[1].as_object().unwrap();
        let criteria2 = match_obj2.get("criteria").unwrap().as_object().unwrap();
        let group_array2 = criteria2.get("group").unwrap().as_array().unwrap();
        assert_eq!(group_array2[0], Value::String("administrators".to_string()));
        assert_eq!(match_obj2.get("permitrootlogin").unwrap(), &Value::Bool(true));
    }

    #[test]
    fn match_with_comma_separated_criteria() {
        let input = r#"
match user alice,bob
    passwordauthentication yes
"#;
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let match_array = result.get("match").unwrap().as_array().unwrap();
        let match_obj = match_array[0].as_object().unwrap();
        let criteria = match_obj.get("criteria").unwrap().as_object().unwrap();
        let user_array = criteria.get("user").unwrap().as_array().unwrap();
        assert_eq!(user_array.len(), 2);
        assert_eq!(user_array[0], Value::String("alice".to_string()));
        assert_eq!(user_array[1], Value::String("bob".to_string()));
    }

    #[test]
    fn match_with_multiarg_keyword() {
        let input = r#"
match user testuser
    passwordauthentication yes
    allowgroups administrators developers
"#;
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let match_array = result.get("match").unwrap().as_array().unwrap();
        let match_obj = match_array[0].as_object().unwrap();
        let allowgroups = match_obj.get("allowgroups").unwrap().as_array().unwrap();
        assert_eq!(allowgroups.len(), 2);
        assert_eq!(allowgroups[0], Value::String("administrators".to_string()));
        assert_eq!(allowgroups[1], Value::String("developers".to_string()));
    }

    #[test]
    fn match_with_repeated_multiarg_keyword() {
        let input = r#"
match user testuser
    allowgroups administrators developers
    allowgroups guests users
"#;
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();

        let match_array = result.get("match").unwrap().as_array().unwrap();
        let match_obj = match_array[0].as_object().unwrap();

        // allowgroups is both MULTI_ARG and REPEATABLE
        // Multiple occurrences should append all values to a flat array
        let allowgroups = match_obj.get("allowgroups").unwrap().as_array().unwrap();
        assert_eq!(allowgroups.len(), 4);
        assert_eq!(allowgroups[0], Value::String("administrators".to_string()));
        assert_eq!(allowgroups[1], Value::String("developers".to_string()));
        assert_eq!(allowgroups[2], Value::String("guests".to_string()));
        assert_eq!(allowgroups[3], Value::String("users".to_string()));
    }

    #[test]
    fn match_with_repeated_single_value_keyword() {
        let input = r#"
match user testuser
    port 2222
    port 3333
"#;
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();

        let match_array = result.get("match").unwrap().as_array().unwrap();
        let match_obj = match_array[0].as_object().unwrap();

        // port is REPEATABLE - values should be in a flat array
        let ports = match_obj.get("port").unwrap().as_array().unwrap();
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0], Value::Number(2222.into()));
        assert_eq!(ports[1], Value::Number(3333.into()));
    }

    #[test]
    fn match_with_comments() {
        let input = r#"
match user developer
    # Enable password authentication for developers - comment ignored
    passwordauthentication yes
"#;
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let match_array = result.get("match").unwrap().as_array().unwrap();
        let match_obj = match_array[0].as_object().unwrap();
        assert_eq!(match_obj.get("passwordauthentication").unwrap(), &Value::Bool(true));
        assert_eq!(match_obj.len(), 2);
    }

    #[test]
    fn match_with_multiple_criteria_types() {
        let input = r#"
match user alice,bob address 1.2.3.4/56
    passwordauthentication yes
    allowtcpforwarding no
"#;
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let match_array = result.get("match").unwrap().as_array().unwrap();
        assert_eq!(match_array.len(), 1);
        let match_obj = match_array[0].as_object().unwrap();

        let criteria = match_obj.get("criteria").unwrap().as_object().unwrap();

        let user_array = criteria.get("user").unwrap().as_array().unwrap();
        assert_eq!(user_array.len(), 2);
        assert_eq!(user_array[0], Value::String("alice".to_string()));
        assert_eq!(user_array[1], Value::String("bob".to_string()));

        let address_array = criteria.get("address").unwrap().as_array().unwrap();
        assert_eq!(address_array.len(), 1);
        assert_eq!(address_array[0], Value::String("1.2.3.4/56".to_string()));

        assert_eq!(match_obj.get("passwordauthentication").unwrap(), &Value::Bool(true));
        assert_eq!(match_obj.get("allowtcpforwarding").unwrap(), &Value::Bool(false));
    }

    #[test]
    fn match_with_operator_argument() {
        let input = r#"
match group administrators
    pubkeyacceptedalgorithms +ssh-rsa
"#;
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let match_array = result.get("match").unwrap().as_array().unwrap();
        let match_obj = match_array[0].as_object().unwrap();
        let pubkey = match_obj.get("pubkeyacceptedalgorithms").unwrap().as_object().unwrap();
        let value_array = pubkey.get("value").unwrap().as_array().unwrap();
        assert_eq!(pubkey.get("operator").unwrap(), &Value::String("+".to_string()));
        assert_eq!(value_array.len(), 1);
        assert_eq!(value_array[0], Value::String("ssh-rsa".to_string()));
    }

    #[test]
    fn subsystem_in_match_block() {
        let input = r#"
match user alice
    subsystem sftp /usr/bin/sftp
"#;
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let match_array = result.get("match").unwrap().as_array().unwrap();
        let match_obj = match_array[0].as_object().unwrap();

        let subsystems = match_obj.get("subsystem").unwrap().as_array().unwrap();
        assert_eq!(subsystems.len(), 1);

        let sftp = subsystems[0].as_object().unwrap();
        assert_eq!(sftp.get("name").unwrap(), &Value::String("sftp".to_string()));
        assert_eq!(sftp.get("value").unwrap(), &Value::String("/usr/bin/sftp".to_string()));
    }

    #[test]
    fn subsystem_multiple_in_match_block() {
        let input = r#"
match user developer
    subsystem sftp /usr/bin/sftp
    subsystem pwsh /usr/bin/pwsh -sshs
"#;
        let result: Map<String, Value> = parse_text_to_map(input).unwrap();
        let match_array = result.get("match").unwrap().as_array().unwrap();
        let match_obj = match_array[0].as_object().unwrap();

        let subsystems = match_obj.get("subsystem").unwrap().as_array().unwrap();
        assert_eq!(subsystems.len(), 2);

        let sftp = subsystems[0].as_object().unwrap();
        assert_eq!(sftp.get("name").unwrap(), &Value::String("sftp".to_string()));
        assert_eq!(sftp.get("value").unwrap(), &Value::String("/usr/bin/sftp".to_string()));

        let pwsh = subsystems[1].as_object().unwrap();
        assert_eq!(pwsh.get("name").unwrap(), &Value::String("pwsh".to_string()));
        assert_eq!(pwsh.get("value").unwrap(), &Value::String("/usr/bin/pwsh -sshs".to_string()));
    }

    #[test]
    fn subsystem_roundtrip() {
        use crate::formatter::write_config_map_to_text;

        let input = r#"subsystem sftp /usr/bin/sftp
subsystem pwsh c:/progra~1/powershell/7/pwsh.exe -sshs -NoLogo
"#;
        let parsed = parse_text_to_map(input).unwrap();
        let formatted = write_config_map_to_text(&parsed).unwrap();

        // Parse again to verify round-trip
        let reparsed = parse_text_to_map(&formatted).unwrap();

        let subsystems1 = parsed.get("subsystem").unwrap().as_array().unwrap();
        let subsystems2 = reparsed.get("subsystem").unwrap().as_array().unwrap();

        assert_eq!(subsystems1.len(), subsystems2.len());
        assert_eq!(subsystems1[0], subsystems2[0]);
        assert_eq!(subsystems1[1], subsystems2[1]);
    }
}

