// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib_registry::config::RegistryValueData;
use roxmltree::{Document, Node};
use rust_i18n::t;
use serde::Serialize;
use serde_json::{Map, Value, json};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use windows::Win32::Globalization::GetUserDefaultLocaleName;

const ADAPTER_TYPE: &str = "Microsoft.Adapter/GroupPolicyTemplate";
const LOCALE_NAME_MAX_LENGTH: usize = 85;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("{0}")]
    Input(String),
    #[error("{0}")]
    Resource(String),
}

impl AdapterError {
    pub fn is_input_error(&self) -> bool {
        matches!(self, Self::Input(_))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyValue {
    Data(dsc_lib_registry::config::RegistryValueData),
    Delete,
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub class: PolicyClass,
    pub key: String,
    pub value_name: Option<String>,
    pub enabled: Option<PolicyValue>,
    pub disabled: Option<PolicyValue>,
    pub elements: Vec<PolicyElement>,
    pub enabled_list: Vec<RegistrySetting>,
    pub disabled_list: Vec<RegistrySetting>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyClass {
    Both,
    Machine,
    User,
}

#[derive(Debug, Clone)]
pub struct RegistrySetting {
    pub key: Option<String>,
    pub value_name: String,
    pub value: PolicyValue,
}

#[derive(Debug, Clone)]
pub struct PolicyElement {
    pub id: String,
    pub key: Option<String>,
    pub value_name: Option<String>,
    pub kind: ElementKind,
}

#[derive(Debug, Clone)]
pub enum ElementKind {
    Boolean {
        true_value: PolicyValue,
        false_value: PolicyValue,
    },
    Decimal {
        minimum: Option<u64>,
        maximum: Option<u64>,
        store_as_text: bool,
    },
    Enum(Vec<EnumItem>),
    List,
    MultiText,
    Text {
        expandable: bool,
    },
}

#[derive(Debug, Clone)]
pub struct EnumItem {
    pub title: String,
    pub value: PolicyValue,
}

#[derive(Debug, Clone)]
pub struct CategoryResource {
    pub type_name: String,
    pub display_name: String,
    pub description: String,
    pub policies: Vec<Policy>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ListedResource {
    #[serde(rename = "type")]
    type_name: String,
    kind: &'static str,
    version: &'static str,
    capabilities: [&'static str; 2],
    path: PathBuf,
    directory: PathBuf,
    implemented_as: &'static str,
    author: &'static str,
    properties: Vec<String>,
    require_adapter: &'static str,
    description: String,
    schema: Map<String, Value>,
}

pub fn list_resources() -> Result<Vec<String>, AdapterError> {
    let policy_definitions = policy_definitions_path()?;
    let locale = user_locale();
    let mut result = Vec::new();
    let entries = fs::read_dir(&policy_definitions).map_err(|error| {
        AdapterError::Resource(
            t!(
                "admx.readDirectory",
                path = policy_definitions.display(),
                error = error
            )
            .to_string(),
        )
    })?;

    for entry in entries {
        let path = entry
            .map_err(|error| {
                AdapterError::Resource(
                    t!(
                        "admx.readDirectory",
                        path = policy_definitions.display(),
                        error = error
                    )
                    .to_string(),
                )
            })?
            .path();
        if !path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("admx"))
        {
            continue;
        }

        match parse_template(&path, &locale) {
            Ok(resources) => {
                for resource in resources {
                    let listed = create_listed_resource(&resource, &path);
                    result.push(serde_json::to_string(&listed).map_err(|error| {
                        AdapterError::Resource(
                            t!("admx.serializeResource", error = error).to_string(),
                        )
                    })?);
                }
            }
            Err(error) => {
                eprintln!(
                    "{}",
                    json!({
                        "warn": t!(
                            "admx.skipTemplate",
                            path = path.display(),
                            error = error
                        )
                    })
                );
            }
        }
    }
    Ok(result)
}

pub fn load_resource(path: &Path, resource_type: &str) -> Result<CategoryResource, AdapterError> {
    parse_template(path, &user_locale())?
        .into_iter()
        .find(|resource| resource.type_name.eq_ignore_ascii_case(resource_type))
        .ok_or_else(|| {
            AdapterError::Input(
                t!(
                    "admx.resourceNotFound",
                    resource = resource_type,
                    path = path.display()
                )
                .to_string(),
            )
        })
}

fn parse_template(path: &Path, locale: &str) -> Result<Vec<CategoryResource>, AdapterError> {
    let admx_content = read_xml(path).map_err(|error| {
        AdapterError::Resource(
            t!("admx.readFile", path = path.display(), error = error).to_string(),
        )
    })?;
    let document = Document::parse(&admx_content).map_err(|error| {
        AdapterError::Resource(
            t!("admx.parseFile", path = path.display(), error = error).to_string(),
        )
    })?;
    let strings = load_strings(path, locale)?;

    let categories: HashMap<String, (String, Option<String>)> = document
        .descendants()
        .filter(|node| node.has_tag_name("category"))
        .filter_map(|category| {
            let name = category.attribute("name")?.to_string();
            let display_name = resolve_reference(category.attribute("displayName")?, &strings);
            let parent = child(category, "parentCategory")
                .and_then(|node| node.attribute("ref"))
                .map(reference_name);
            Some((name, (display_name, parent)))
        })
        .collect();

    let mut policies_by_category: HashMap<String, Vec<Policy>> = HashMap::new();
    for policy_node in document
        .descendants()
        .filter(|node| node.has_tag_name("policy"))
    {
        let Some(category) = child(policy_node, "parentCategory")
            .and_then(|node| node.attribute("ref"))
            .map(reference_name)
        else {
            continue;
        };
        let policy = parse_policy(policy_node, &strings)?;
        policies_by_category
            .entry(category)
            .or_default()
            .push(policy);
    }

    let template_description = load_adml_description(path, locale).unwrap_or_default();
    let mut resources = Vec::new();
    for (category_name, policies) in policies_by_category {
        let (display_name, parent) = categories
            .get(&category_name)
            .cloned()
            .unwrap_or_else(|| (category_name.clone(), None));
        let parent_name = parent.as_deref().unwrap_or(&category_name);
        resources.push(CategoryResource {
            type_name: resource_type_name(parent_name, &category_name),
            display_name,
            description: template_description.clone(),
            policies,
        });
    }
    resources.sort_by(|left, right| left.type_name.cmp(&right.type_name));
    Ok(resources)
}

fn parse_policy(
    node: Node<'_, '_>,
    strings: &HashMap<String, String>,
) -> Result<Policy, AdapterError> {
    let required_attribute = |name: &str| {
        node.attribute(name).ok_or_else(|| {
            AdapterError::Resource(
                t!(
                    "admx.missingPolicyAttribute",
                    policy = node.attribute("name").unwrap_or_default(),
                    attribute = name
                )
                .to_string(),
            )
        })
    };

    let value_name = node.attribute("valueName").map(ToString::to_string);
    let enabled = child(node, "enabledValue")
        .map(parse_policy_value)
        .transpose()?
        .or_else(|| {
            value_name
                .as_ref()
                .map(|_| PolicyValue::Data(RegistryValueData::DWord(1)))
        });
    let disabled = child(node, "disabledValue")
        .map(parse_policy_value)
        .transpose()?
        .or_else(|| value_name.as_ref().map(|_| PolicyValue::Delete));
    let elements = child(node, "elements")
        .map(|elements| {
            elements
                .children()
                .filter(Node::is_element)
                .map(|element| parse_element(element, strings))
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?
        .unwrap_or_default();
    let class = match required_attribute("class")? {
        "Both" => PolicyClass::Both,
        "Machine" => PolicyClass::Machine,
        "User" => PolicyClass::User,
        value => {
            return Err(AdapterError::Resource(
                t!("admx.invalidPolicyClass", class = value).to_string(),
            ));
        }
    };

    Ok(Policy {
        name: required_attribute("name")?.to_string(),
        display_name: resolve_reference(required_attribute("displayName")?, strings),
        description: node
            .attribute("explainText")
            .map(|value| resolve_reference(value, strings)),
        class,
        key: required_attribute("key")?.to_string(),
        value_name,
        enabled,
        disabled,
        elements,
        enabled_list: child(node, "enabledList")
            .map(parse_value_list)
            .transpose()?
            .unwrap_or_default(),
        disabled_list: child(node, "disabledList")
            .map(parse_value_list)
            .transpose()?
            .unwrap_or_default(),
    })
}

fn parse_value_list(node: Node<'_, '_>) -> Result<Vec<RegistrySetting>, AdapterError> {
    node.children()
        .filter(|child| child.has_tag_name("item"))
        .map(|item| {
            let value = child(item, "value")
                .ok_or_else(|| AdapterError::Resource(t!("admx.emptyPolicyValue").to_string()))
                .and_then(parse_policy_value)?;
            Ok(RegistrySetting {
                key: item.attribute("key").map(ToString::to_string),
                value_name: item
                    .attribute("valueName")
                    .ok_or_else(|| AdapterError::Resource(t!("admx.missingValueName").to_string()))?
                    .to_string(),
                value,
            })
        })
        .collect()
}

fn parse_element(
    node: Node<'_, '_>,
    strings: &HashMap<String, String>,
) -> Result<PolicyElement, AdapterError> {
    let id = node
        .attribute("id")
        .ok_or_else(|| AdapterError::Resource(t!("admx.missingElementId").to_string()))?
        .to_string();
    let value_name = node.attribute("valueName").map(ToString::to_string);
    let kind = match node.tag_name().name() {
        "boolean" => ElementKind::Boolean {
            true_value: child(node, "trueValue")
                .map(parse_policy_value)
                .transpose()?
                .unwrap_or(PolicyValue::Data(RegistryValueData::DWord(1))),
            false_value: child(node, "falseValue")
                .map(parse_policy_value)
                .transpose()?
                .unwrap_or(PolicyValue::Data(RegistryValueData::DWord(0))),
        },
        "decimal" => ElementKind::Decimal {
            minimum: node.attribute("minValue").map(parse_u64).transpose()?,
            maximum: node.attribute("maxValue").map(parse_u64).transpose()?,
            store_as_text: node.attribute("storeAsText") == Some("true"),
        },
        "enum" => ElementKind::Enum(
            node.children()
                .filter(|child| child.has_tag_name("item"))
                .map(|item| {
                    let value_container = child(item, "value").ok_or_else(|| {
                        AdapterError::Resource(t!("admx.emptyPolicyValue").to_string())
                    })?;
                    let value = parse_policy_value(value_container)?;
                    Ok(EnumItem {
                        title: resolve_reference(
                            item.attribute("displayName").unwrap_or_default(),
                            strings,
                        ),
                        value,
                    })
                })
                .collect::<Result<Vec<_>, AdapterError>>()?,
        ),
        "list" => ElementKind::List,
        "multiText" => ElementKind::MultiText,
        "text" => ElementKind::Text {
            expandable: node.attribute("expandable") == Some("true"),
        },
        unsupported => {
            return Err(AdapterError::Resource(
                t!("admx.unsupportedElementType", element_type = unsupported).to_string(),
            ));
        }
    };
    Ok(PolicyElement {
        id,
        key: node.attribute("key").map(ToString::to_string),
        value_name,
        kind,
    })
}

fn parse_policy_value(container: Node<'_, '_>) -> Result<PolicyValue, AdapterError> {
    use dsc_lib_registry::config::RegistryValueData;

    let Some(value) = container.children().find(Node::is_element) else {
        return Err(AdapterError::Resource(
            t!("admx.emptyPolicyValue").to_string(),
        ));
    };
    let text = value.text().unwrap_or_default().trim();
    let data = match value.tag_name().name() {
        "delete" => return Ok(PolicyValue::Delete),
        "decimal" => RegistryValueData::DWord(parse_u32(value.attribute("value").unwrap_or(text))?),
        "longDecimal" => {
            RegistryValueData::QWord(parse_u64(value.attribute("value").unwrap_or(text))?)
        }
        "string" => RegistryValueData::String(text.to_string()),
        "expandableString" => RegistryValueData::ExpandString(text.to_string()),
        "multiString" => RegistryValueData::MultiString(
            value
                .children()
                .filter(|child| child.has_tag_name("string"))
                .filter_map(|child| child.text())
                .map(ToString::to_string)
                .collect(),
        ),
        "binary" => RegistryValueData::Binary(parse_binary(text)?),
        unsupported => {
            return Err(AdapterError::Resource(
                t!("admx.unsupportedValueType", value_type = unsupported).to_string(),
            ));
        }
    };
    Ok(PolicyValue::Data(data))
}

fn create_listed_resource(resource: &CategoryResource, path: &Path) -> ListedResource {
    let mut properties = Map::new();
    properties.insert(
        "scope".to_string(),
        json!({
            "type": "string",
            "title": t!("schema.scopeTitle"),
            "description": t!("schema.scopeDescription"),
            "enum": ["allUsers", "currentUser"],
            "default": "currentUser"
        }),
    );
    for policy in &resource.policies {
        let boolean_schema = policy
            .enabled
            .as_ref()
            .zip(policy.disabled.as_ref())
            .map(|_| {
                json!({
                    "type": "boolean",
                    "title": policy.display_name,
                    "description": policy.description
                })
            });
        let object_schema = if policy.elements.is_empty() {
            None
        } else {
            let mut element_properties = Map::new();
            if boolean_schema.is_some() {
                element_properties.insert(
                    "enabled".to_string(),
                    json!({
                        "type": "boolean",
                        "title": t!("schema.enabledTitle")
                    }),
                );
            }
            for element in &policy.elements {
                element_properties.insert(element.id.clone(), element_schema(element));
            }
            Some(json!({
                "type": "object",
                "title": policy.display_name,
                "description": policy.description,
                "additionalProperties": false,
                "properties": element_properties
            }))
        };
        let property = match (boolean_schema, object_schema) {
            (Some(boolean), Some(object)) => json!({ "oneOf": [boolean, object] }),
            (Some(boolean), None) => boolean,
            (None, Some(object)) => object,
            (None, None) => json!({
                "type": "boolean",
                "title": policy.display_name,
                "description": policy.description
            }),
        };
        properties.insert(policy.name.clone(), property);
    }

    let embedded = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": resource.display_name,
        "description": resource.description,
        "type": "object",
        "additionalProperties": false,
        "properties": properties
    });
    let mut schema = Map::new();
    schema.insert("embedded".to_string(), embedded);

    ListedResource {
        type_name: resource.type_name.clone(),
        kind: "resource",
        version: "0.1.0",
        capabilities: ["get", "set"],
        path: path.to_path_buf(),
        directory: path.parent().unwrap_or_else(|| Path::new("")).to_path_buf(),
        implemented_as: "adapter",
        author: "Microsoft",
        properties: std::iter::once("scope".to_string())
            .chain(resource.policies.iter().map(|policy| policy.name.clone()))
            .collect(),
        require_adapter: ADAPTER_TYPE,
        description: resource.description.clone(),
        schema,
    }
}

fn element_schema(element: &PolicyElement) -> Value {
    match &element.kind {
        ElementKind::Boolean { .. } => json!({
            "type": "boolean",
            "title": element.id
        }),
        ElementKind::Decimal {
            minimum, maximum, ..
        } => {
            let mut schema = Map::new();
            schema.insert("type".to_string(), Value::String("integer".to_string()));
            schema.insert("title".to_string(), Value::String(element.id.clone()));
            if let Some(minimum) = minimum {
                schema.insert("minimum".to_string(), Value::from(*minimum));
            }
            if let Some(maximum) = maximum {
                schema.insert("maximum".to_string(), Value::from(*maximum));
            }
            Value::Object(schema)
        }
        ElementKind::Enum(items) => json!({
            "title": element.id,
            "oneOf": items.iter().map(|item| json!({
                "const": policy_value_to_json(&item.value),
                "title": item.title
            })).collect::<Vec<_>>()
        }),
        ElementKind::List => json!({
            "type": "object",
            "title": element.id,
            "additionalProperties": { "type": "string" }
        }),
        ElementKind::MultiText => json!({
            "type": "array",
            "title": element.id,
            "items": { "type": "string" }
        }),
        ElementKind::Text { .. } => json!({
            "type": "string",
            "title": element.id
        }),
    }
}

pub fn registry_value_to_json(value: &RegistryValueData) -> Value {
    match value {
        RegistryValueData::String(value) | RegistryValueData::ExpandString(value) => {
            Value::String(value.clone())
        }
        RegistryValueData::DWord(value) => Value::from(*value),
        RegistryValueData::QWord(value) => Value::from(*value),
        RegistryValueData::Binary(value) => {
            Value::Array(value.iter().copied().map(Value::from).collect())
        }
        RegistryValueData::MultiString(value) => {
            Value::Array(value.iter().cloned().map(Value::String).collect())
        }
        RegistryValueData::None => Value::Null,
    }
}

pub fn policy_value_to_json(value: &PolicyValue) -> Value {
    match value {
        PolicyValue::Data(data) => registry_value_to_json(data),
        PolicyValue::Delete => Value::Null,
    }
}

fn load_strings(path: &Path, locale: &str) -> Result<HashMap<String, String>, AdapterError> {
    let adml_path = adml_path(path, locale)?;
    let content = read_xml(&adml_path).map_err(|error| {
        AdapterError::Resource(
            t!("admx.readFile", path = adml_path.display(), error = error).to_string(),
        )
    })?;
    let document = Document::parse(&content).map_err(|error| {
        AdapterError::Resource(
            t!("admx.parseFile", path = adml_path.display(), error = error).to_string(),
        )
    })?;
    Ok(document
        .descendants()
        .filter(|node| node.has_tag_name("string"))
        .filter_map(|node| Some((node.attribute("id")?.to_string(), node.text()?.to_string())))
        .collect())
}

fn load_adml_description(path: &Path, locale: &str) -> Option<String> {
    let adml_path = adml_path(path, locale).ok()?;
    let content = read_xml(&adml_path).ok()?;
    let document = Document::parse(&content).ok()?;
    document
        .root_element()
        .children()
        .find(|node| node.has_tag_name("description"))
        .and_then(|node| node.text())
        .map(ToString::to_string)
}

fn adml_path(admx_path: &Path, locale: &str) -> Result<PathBuf, AdapterError> {
    let file_name = admx_path
        .file_stem()
        .ok_or_else(|| AdapterError::Resource(t!("admx.invalidPath").to_string()))?;
    let base = admx_path
        .parent()
        .ok_or_else(|| AdapterError::Resource(t!("admx.invalidPath").to_string()))?;
    let localized = base.join(locale).join(file_name).with_extension("adml");
    if localized.exists() {
        return Ok(localized);
    }
    let fallback = base.join("en-US").join(file_name).with_extension("adml");
    if fallback.exists() {
        return Ok(fallback);
    }
    Err(AdapterError::Resource(
        t!(
            "admx.admlNotFound",
            template = admx_path.display(),
            locale = locale
        )
        .to_string(),
    ))
}

fn policy_definitions_path() -> Result<PathBuf, AdapterError> {
    let system_root = env::var_os("SystemRoot")
        .ok_or_else(|| AdapterError::Resource(t!("admx.systemRootNotFound").to_string()))?;
    Ok(PathBuf::from(system_root).join("PolicyDefinitions"))
}

fn user_locale() -> String {
    let mut buffer = [0_u16; LOCALE_NAME_MAX_LENGTH];
    // SAFETY: The buffer is writable for the specified length and the API writes a
    // null-terminated locale name no larger than LOCALE_NAME_MAX_LENGTH.
    let length = unsafe { GetUserDefaultLocaleName(&mut buffer) };
    if let Ok(length) = usize::try_from(length)
        && length > 1
    {
        String::from_utf16_lossy(&buffer[..length - 1])
    } else {
        "en-US".to_string()
    }
}

fn resolve_reference(value: &str, strings: &HashMap<String, String>) -> String {
    value
        .strip_prefix("$(string.")
        .and_then(|value| value.strip_suffix(')'))
        .and_then(|key| strings.get(key))
        .cloned()
        .unwrap_or_else(|| value.to_string())
}

fn reference_name(reference: &str) -> String {
    reference
        .rsplit_once(':')
        .map_or(reference, |(_, name)| name)
        .to_string()
}

fn resource_name_segment(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}

fn resource_type_name(parent_category: &str, category: &str) -> String {
    format!(
        "GPO.{}/{}",
        resource_name_segment(parent_category),
        resource_name_segment(category)
    )
}

fn child<'a>(node: Node<'a, 'a>, name: &str) -> Option<Node<'a, 'a>> {
    node.children().find(|child| child.has_tag_name(name))
}

fn parse_u32(value: &str) -> Result<u32, AdapterError> {
    value.parse().map_err(|error| {
        AdapterError::Resource(t!("admx.invalidNumber", value = value, error = error).to_string())
    })
}

fn parse_u64(value: &str) -> Result<u64, AdapterError> {
    value.parse().map_err(|error| {
        AdapterError::Resource(t!("admx.invalidNumber", value = value, error = error).to_string())
    })
}

fn parse_binary(value: &str) -> Result<Vec<u8>, AdapterError> {
    value
        .split([',', ' ', '\t', '\r', '\n'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            u8::from_str_radix(part.trim_start_matches("0x"), 16).map_err(|error| {
                AdapterError::Resource(
                    t!("admx.invalidBinary", value = part, error = error).to_string(),
                )
            })
        })
        .collect()
}

fn read_xml(path: &Path) -> Result<String, std::io::Error> {
    let bytes = fs::read(path)?;
    if bytes.starts_with(&[0xff, 0xfe]) {
        let (chunks, remainder) = bytes[2..].as_chunks::<2>();
        if !remainder.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                t!("admx.invalidUtf16Length", path = path.display()).to_string(),
            ));
        }
        let words = chunks
            .iter()
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16(&words)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error));
    }
    if bytes.starts_with(&[0xfe, 0xff]) {
        let (chunks, remainder) = bytes[2..].as_chunks::<2>();
        if !remainder.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                t!("admx.invalidUtf16Length", path = path.display()).to_string(),
            ));
        }
        let words = chunks
            .iter()
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        return String::from_utf16(&words)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error));
    }
    String::from_utf8(bytes)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
}

#[cfg(test)]
mod tests {
    use super::{parse_binary, reference_name, resource_name_segment, resource_type_name};

    #[test]
    fn normalizes_resource_name_parts() {
        assert_eq!(
            reference_name("windows:WindowsComponents"),
            "WindowsComponents"
        );
        assert_eq!(
            resource_name_segment("Windows PowerShell"),
            "Windows_PowerShell"
        );
        assert_eq!(resource_name_segment("App-V (Client)"), "App_V__Client_");
        assert_eq!(
            resource_type_name("WindowsComponents", "PowerShell"),
            "GPO.WindowsComponents/PowerShell"
        );
    }

    #[test]
    fn parses_binary_values() {
        assert_eq!(parse_binary("01, ff, 0A").unwrap(), vec![1, 255, 10]);
    }
}
