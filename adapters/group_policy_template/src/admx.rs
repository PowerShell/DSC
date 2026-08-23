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
        .or_else(|| {
            value_name
                .as_ref()
                .map(|_| PolicyValue::Data(RegistryValueData::DWord(0)))
        });
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
        name: policy_property_name(required_attribute("name")?),
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
        let state_schema = json!({
            "type": "string",
            "title": t!("schema.stateTitle"),
            "enum": ["Enabled", "Disabled", "NotConfigured"]
        });
        let property = if policy.elements.is_empty() {
            json!({
                "type": "string",
                "title": policy.display_name,
                "description": policy.description,
                "enum": ["Enabled", "Disabled", "NotConfigured"]
            })
        } else {
            let mut element_properties = Map::new();
            element_properties.insert("state".to_string(), state_schema);
            for element in &policy.elements {
                element_properties.insert(element.id.clone(), element_schema(element));
            }
            json!({
                "type": "object",
                "title": policy.display_name,
                "description": policy.description,
                "additionalProperties": false,
                "properties": element_properties
            })
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
            "type": "array",
            "title": element.id,
            "items": { "type": "string" },
            "uniqueItems": true
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

fn policy_property_name(name: &str) -> String {
    name.strip_prefix("Enabled")
        .or_else(|| name.strip_prefix("Enable"))
        .filter(|name| !name.is_empty())
        .unwrap_or(name)
        .to_string()
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
    use super::{
        AdapterError, ElementKind, PolicyClass, PolicyValue, adml_path, create_listed_resource,
        parse_binary, parse_template, policy_property_name, policy_value_to_json, read_xml,
        reference_name, registry_value_to_json, resolve_reference, resource_name_segment,
        resource_type_name,
    };
    use dsc_lib_registry::config::RegistryValueData;
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        system_root: PathBuf,
        root: PathBuf,
        admx: PathBuf,
    }

    impl Fixture {
        fn new(admx: &str, adml: &str) -> Self {
            let id = FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
            let system_root = std::env::temp_dir().join(format!(
                "group_policy_template_{}_{}",
                std::process::id(),
                id
            ));
            let root = system_root.join("PolicyDefinitions");
            let locale = root.join("en-US");
            fs::create_dir_all(&locale).unwrap();
            let admx_path = root.join("fixture.admx");
            fs::write(&admx_path, admx).unwrap();
            fs::write(locale.join("fixture.adml"), adml).unwrap();
            Self {
                system_root,
                root,
                admx: admx_path,
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.system_root).unwrap();
        }
    }

    const ADML: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<policyDefinitionResources>
  <displayName>Fixture policies</displayName>
  <description>Localized template description</description>
  <resources>
    <stringTable>
      <string id="Category">Localized Category</string>
      <string id="Complex">Complex Policy</string>
      <string id="ComplexHelp">Localized policy help</string>
      <string id="ChoiceOne">First choice</string>
      <string id="ChoiceTwo">Second choice</string>
      <string id="Simple">Simple Policy</string>
    </stringTable>
  </resources>
</policyDefinitionResources>"#;

    const ADMX: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<policyDefinitions>
  <categories>
    <category name="Parent" displayName="$(string.Parent)" />
    <category name="StableCategory" displayName="$(string.Category)">
      <parentCategory ref="windows:WindowsComponents" />
    </category>
  </categories>
  <policies>
    <policy name="ComplexPolicy" class="Both" displayName="$(string.Complex)"
            explainText="$(string.ComplexHelp)" key="Software\Fixture" valueName="State">
      <parentCategory ref="StableCategory" />
      <enabledValue><decimal value="7" /></enabledValue>
      <disabledValue><delete /></disabledValue>
      <elements>
        <boolean id="BooleanValue" valueName="Boolean">
          <trueValue><string>yes</string></trueValue>
          <falseValue><string>no</string></falseValue>
        </boolean>
        <decimal id="NumberValue" valueName="Number" minValue="1" maxValue="10" />
        <decimal id="TextNumber" valueName="TextNumber" storeAsText="true" />
        <enum id="EnumValue" valueName="Enum">
          <item displayName="$(string.ChoiceOne)"><value><decimal value="1" /></value></item>
          <item displayName="$(string.ChoiceTwo)"><value><string>two</string></value></item>
        </enum>
        <list id="ListValue" key="Software\Fixture\List" />
        <multiText id="MultiValue" valueName="Multi" />
        <text id="TextValue" valueName="Text" />
        <text id="ExpandableValue" valueName="Expandable" expandable="true" />
      </elements>
      <enabledList>
        <item valueName="EnabledString"><value><expandableString>%TEMP%</expandableString></value></item>
        <item key="Software\Fixture\Other" valueName="EnabledMulti">
          <value><multiString><string>one</string><string>two</string></multiString></value>
        </item>
      </enabledList>
      <disabledList>
        <item valueName="DisabledBinary"><value><binary>01, ff, 0A</binary></value></item>
        <item valueName="DisabledQword"><value><longDecimal value="4294967296" /></value></item>
      </disabledList>
    </policy>
    <policy name="SimplePolicy" class="Machine" displayName="$(string.Simple)"
            key="Software\Fixture" valueName="Simple">
      <parentCategory ref="StableCategory" />
    </policy>
  </policies>
</policyDefinitions>"#;

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
        assert_eq!(policy_property_name("EnableModuleLogging"), "ModuleLogging");
        assert_eq!(policy_property_name("EnabledFeature"), "Feature");
        assert_eq!(policy_property_name("NoAddPage"), "NoAddPage");
    }

    #[test]
    fn parses_binary_values() {
        assert_eq!(parse_binary("01, ff, 0A").unwrap(), vec![1, 255, 10]);
        assert!(parse_binary("invalid").is_err());
    }

    #[test]
    fn parses_template_and_generates_localized_schema() {
        let fixture = Fixture::new(ADMX, ADML);
        let resources = parse_template(&fixture.admx, "fr-FR").unwrap();
        assert_eq!(resources.len(), 1);

        let resource = &resources[0];
        assert_eq!(resource.type_name, "GPO.WindowsComponents/StableCategory");
        assert_eq!(resource.display_name, "Localized Category");
        assert_eq!(resource.description, "Localized template description");
        assert_eq!(resource.policies.len(), 2);

        let complex = resource
            .policies
            .iter()
            .find(|policy| policy.name == "ComplexPolicy")
            .unwrap();
        assert_eq!(complex.display_name, "Complex Policy");
        assert_eq!(
            complex.description.as_deref(),
            Some("Localized policy help")
        );
        assert_eq!(complex.class, PolicyClass::Both);
        assert_eq!(
            complex.enabled,
            Some(PolicyValue::Data(RegistryValueData::DWord(7)))
        );
        assert_eq!(complex.disabled, Some(PolicyValue::Delete));
        assert_eq!(complex.elements.len(), 8);
        assert_eq!(complex.enabled_list.len(), 2);
        assert_eq!(complex.disabled_list.len(), 2);
        assert!(matches!(
            complex.elements[0].kind,
            ElementKind::Boolean { .. }
        ));
        assert!(matches!(
            complex.elements[1].kind,
            ElementKind::Decimal {
                minimum: Some(1),
                maximum: Some(10),
                store_as_text: false
            }
        ));
        assert!(matches!(complex.elements[3].kind, ElementKind::Enum(_)));
        assert!(matches!(complex.elements[4].kind, ElementKind::List));
        assert!(matches!(complex.elements[5].kind, ElementKind::MultiText));
        assert!(matches!(
            complex.elements[7].kind,
            ElementKind::Text { expandable: true }
        ));

        let simple = resource
            .policies
            .iter()
            .find(|policy| policy.name == "SimplePolicy")
            .unwrap();
        assert_eq!(simple.class, PolicyClass::Machine);
        assert_eq!(
            simple.enabled,
            Some(PolicyValue::Data(RegistryValueData::DWord(1)))
        );
        assert_eq!(
            simple.disabled,
            Some(PolicyValue::Data(RegistryValueData::DWord(0)))
        );

        let listed = create_listed_resource(resource, &fixture.admx);
        assert_eq!(listed.type_name, resource.type_name);
        assert_eq!(listed.require_adapter, super::ADAPTER_TYPE);
        assert_eq!(listed.capabilities, ["get", "set"]);
        assert_eq!(
            listed.schema["embedded"]["properties"]["scope"]["default"],
            "currentUser"
        );
        assert_eq!(
            listed.schema["embedded"]["properties"]["ComplexPolicy"]["properties"]["NumberValue"]["minimum"],
            1
        );
        assert_eq!(
            listed.schema["embedded"]["properties"]["ComplexPolicy"]["properties"]["EnumValue"]["oneOf"]
                [1]["title"],
            "Second choice"
        );
        assert_eq!(
            listed.schema["embedded"]["properties"]["ComplexPolicy"]["properties"]["ListValue"],
            json!({
                "type": "array",
                "title": "ListValue",
                "items": { "type": "string" },
                "uniqueItems": true
            })
        );
        assert_eq!(
            listed.schema["embedded"]["properties"]["SimplePolicy"]["enum"],
            json!(["Enabled", "Disabled", "NotConfigured"])
        );
    }

    #[test]
    fn lists_resources_from_policy_definitions() {
        let fixture = Fixture::new(ADMX, ADML);
        let original_system_root = std::env::var_os("SystemRoot");
        // SAFETY: This test restores the process environment before returning, and no
        // other adapter test reads SystemRoot.
        unsafe { std::env::set_var("SystemRoot", &fixture.system_root) };
        let result = super::list_resources();
        if let Some(original) = original_system_root {
            // SAFETY: Restores the value captured immediately before this test.
            unsafe { std::env::set_var("SystemRoot", original) };
        } else {
            // SAFETY: Restores the absence of the variable captured before this test.
            unsafe { std::env::remove_var("SystemRoot") };
        }

        let resources = result.unwrap();
        assert_eq!(resources.len(), 1);
        let resource: serde_json::Value = serde_json::from_str(&resources[0]).unwrap();
        assert_eq!(resource["type"], "GPO.WindowsComponents/StableCategory");
        assert_eq!(
            resource["requireAdapter"],
            "Microsoft.Adapter/GroupPolicyTemplate"
        );
    }

    #[test]
    fn converts_every_registry_value_to_json() {
        assert_eq!(
            registry_value_to_json(&RegistryValueData::String("value".to_string())),
            json!("value")
        );
        assert_eq!(
            registry_value_to_json(&RegistryValueData::ExpandString("%TEMP%".to_string())),
            json!("%TEMP%")
        );
        assert_eq!(
            registry_value_to_json(&RegistryValueData::DWord(42)),
            json!(42)
        );
        assert_eq!(
            registry_value_to_json(&RegistryValueData::QWord(4_294_967_296)),
            json!(4_294_967_296_u64)
        );
        assert_eq!(
            registry_value_to_json(&RegistryValueData::Binary(vec![1, 2])),
            json!([1, 2])
        );
        assert_eq!(
            registry_value_to_json(&RegistryValueData::MultiString(vec![
                "one".to_string(),
                "two".to_string()
            ])),
            json!(["one", "two"])
        );
        assert_eq!(
            registry_value_to_json(&RegistryValueData::None),
            json!(null)
        );
        assert_eq!(policy_value_to_json(&PolicyValue::Delete), json!(null));
    }

    #[test]
    fn reads_utf8_utf16_and_rejects_malformed_utf16() {
        let fixture = Fixture::new(ADMX, ADML);
        assert_eq!(read_xml(&fixture.admx).unwrap(), ADMX);

        let le = fixture.root.join("le.xml");
        let be = fixture.root.join("be.xml");
        let malformed = fixture.root.join("malformed.xml");
        let text = "<root>value</root>";
        let words: Vec<u16> = text.encode_utf16().collect();
        let mut le_bytes = vec![0xff, 0xfe];
        le_bytes.extend(words.iter().flat_map(|word| word.to_le_bytes()));
        let mut be_bytes = vec![0xfe, 0xff];
        be_bytes.extend(words.iter().flat_map(|word| word.to_be_bytes()));
        fs::write(&le, le_bytes).unwrap();
        fs::write(&be, be_bytes).unwrap();
        fs::write(&malformed, [0xff, 0xfe, 0x01]).unwrap();

        assert_eq!(read_xml(&le).unwrap(), text);
        assert_eq!(read_xml(&be).unwrap(), text);
        assert!(read_xml(&malformed).is_err());
    }

    #[test]
    fn handles_localization_and_template_errors() {
        let fixture = Fixture::new(ADMX, ADML);
        assert!(
            adml_path(&fixture.admx, "fr-FR")
                .unwrap()
                .ends_with(Path::new("en-US").join("fixture").with_extension("adml"))
        );
        assert_eq!(
            resolve_reference(
                "$(string.Category)",
                &std::collections::HashMap::from([(
                    "Category".to_string(),
                    "Localized".to_string()
                )])
            ),
            "Localized"
        );
        assert_eq!(
            resolve_reference("$(string.Missing)", &std::collections::HashMap::new()),
            "$(string.Missing)"
        );
        assert!(matches!(
            AdapterError::Input("input".to_string()),
            error if error.is_input_error()
        ));
        assert!(!AdapterError::Resource("resource".to_string()).is_input_error());

        let invalid_class = ADMX.replace("class=\"Both\"", "class=\"Invalid\"");
        let invalid = Fixture::new(&invalid_class, ADML);
        assert!(
            parse_template(&invalid.admx, "en-US")
                .unwrap_err()
                .to_string()
                .contains("Invalid")
        );

        let unsupported = ADMX.replace(
            "<text id=\"TextValue\" valueName=\"Text\" />",
            "<unsupported id=\"TextValue\" valueName=\"Text\" />",
        );
        let invalid = Fixture::new(&unsupported, ADML);
        assert!(
            parse_template(&invalid.admx, "en-US")
                .unwrap_err()
                .to_string()
                .contains("unsupported")
        );

        for invalid_admx in [
            ADMX.replace(" id=\"BooleanValue\"", ""),
            ADMX.replace("<decimal value=\"7\" />", "<unsupported>7</unsupported>"),
            ADMX.replace(" minValue=\"1\"", " minValue=\"invalid\""),
            ADMX.replace(" valueName=\"DisabledBinary\"", ""),
            ADMX.replace("<value><binary>01, ff, 0A</binary></value>", "<value />"),
        ] {
            let invalid = Fixture::new(&invalid_admx, ADML);
            assert!(parse_template(&invalid.admx, "en-US").is_err());
        }
    }
}
