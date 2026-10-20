// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Rules about parameter and variable declarations and the values supplied for them.

use std::{collections::HashSet, sync::LazyLock};

use regex::Regex;
use rust_i18n::t;
use serde_json::Value;

use crate::{
    configure::{
        config_doc::{DataType, Parameter, ValueOrCopy},
        constraints::{check_allowed_values, check_length, check_number_limits},
    },
    lint::{
        context::LintContext,
        diagnostic::LintCategory,
        expression_index::{literal_string_arg, ExpressionScope},
        options::RuleLevel,
        pointer::JsonPointer,
        rule::{DiagnosticSink, Rule, RuleTier},
    },
};

/// Parameter names that suggest the value is a secret.
static SECRET_NAME_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(password|passwd|secret|token|apikey|api[-_]?key|credential|connectionstring)")
        .expect("the secret name pattern is a valid regex")
});

/// The rules in this module.
#[must_use]
pub fn rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "no-unused-parameters",
            category: LintCategory::Parameter,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.parameter.noUnusedParametersDescription").to_string(),
            check: no_unused_parameters,
        },
        Rule {
            id: "no-unused-variables",
            category: LintCategory::Parameter,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.parameter.noUnusedVariablesDescription").to_string(),
            check: no_unused_variables,
        },
        Rule {
            id: "parameter-default-type",
            category: LintCategory::Parameter,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.parameter.parameterDefaultTypeDescription").to_string(),
            check: parameter_default_type,
        },
        Rule {
            id: "invalid-parameter-constraints",
            category: LintCategory::Parameter,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.parameter.invalidParameterConstraintsDescription").to_string(),
            check: invalid_parameter_constraints,
        },
        Rule {
            id: "secure-parameter-default",
            category: LintCategory::Parameter,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.parameter.secureParameterDefaultDescription").to_string(),
            check: secure_parameter_default,
        },
        Rule {
            id: "prefer-secure-type",
            category: LintCategory::Parameter,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.parameter.preferSecureTypeDescription").to_string(),
            check: prefer_secure_type,
        },
        Rule {
            id: "secure-value-in-output",
            category: LintCategory::Parameter,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.parameter.secureValueInOutputDescription").to_string(),
            check: secure_value_in_output,
        },
        Rule {
            id: "use-parameter-description",
            category: LintCategory::Parameter,
            tier: RuleTier::Static,
            default_level: RuleLevel::Off,
            description: || t!("lint.rules.parameter.useParameterDescriptionDescription").to_string(),
            check: use_parameter_description,
        },
        Rule {
            id: "undeclared-parameter-value",
            category: LintCategory::Parameter,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.parameter.undeclaredParameterValueDescription").to_string(),
            check: undeclared_parameter_value,
        },
        Rule {
            id: "missing-parameter-value",
            category: LintCategory::Parameter,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.parameter.missingParameterValueDescription").to_string(),
            check: missing_parameter_value,
        },
        Rule {
            id: "parameter-value-type",
            category: LintCategory::Parameter,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.parameter.parameterValueTypeDescription").to_string(),
            check: parameter_value_type,
        },
    ]
}

/// The names passed as the literal first argument of every call of a function, outside
/// user-defined function bodies.
///
/// # Arguments
///
/// * `context` - The lint context.
/// * `function_name` - The function, such as `parameters` or `variables`.
fn referenced_names(context: &LintContext, function_name: &str) -> HashSet<String> {
    context.expressions.calls_named(function_name).into_iter()
        .filter(|call| call.site.scope != ExpressionScope::UserFunction)
        .filter_map(|call| literal_string_arg(call.function, 0).map(String::from))
        .collect()
}

/// The pointer to a declared parameter.
///
/// # Arguments
///
/// * `name` - The parameter name.
fn parameter_pointer(name: &str) -> JsonPointer {
    JsonPointer::root().push("parameters").push(name)
}

/// Whether a value is acceptable for a parameter type.
///
/// Secure types accept the plain value as well as the wrapped form (`{"secureString": ...}`),
/// because DSC wraps plain values when it imports parameters.
///
/// # Arguments
///
/// * `value` - The value.
/// * `data_type` - The parameter type.
fn value_matches_type(value: &Value, data_type: &DataType) -> bool {
    match data_type {
        DataType::String => value.is_string(),
        DataType::SecureString => value.is_string() || wrapped_secure_value(value, "secureString").is_some_and(Value::is_string),
        DataType::Int => value.is_i64(),
        DataType::Bool => value.is_boolean(),
        DataType::Object => value.is_object(),
        DataType::SecureObject => value.is_object(),
        DataType::Array => value.is_array(),
    }
}

/// The inner value of a wrapped secure value such as `{"secureString": "..."}`.
///
/// # Arguments
///
/// * `value` - The value to unwrap.
/// * `key` - The wrapper key, `secureString` or `secureObject`.
fn wrapped_secure_value<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    let object = value.as_object()?;
    if object.len() == 1 { object.get(key) } else { None }
}

/// Check a literal value against a parameter's type and constraints.
///
/// # Arguments
///
/// * `name` - The parameter name.
/// * `value` - The value.
/// * `parameter` - The parameter declaration.
///
/// # Errors
///
/// Returns the reason the value is invalid.
fn check_value(name: &str, value: &Value, parameter: &Parameter) -> Result<(), String> {
    if !value_matches_type(value, &parameter.parameter_type) {
        return Err(t!("lint.rules.parameter.valueNotOfType", type_name = parameter.parameter_type).to_string());
    }
    check_allowed_values(name, value, parameter).map_err(|err| LintContext::error_message(&err))?;
    if parameter.parameter_type == DataType::Int {
        check_number_limits(name, value, parameter).map_err(|err| LintContext::error_message(&err))?;
    }
    if (value.is_string() || value.is_array()) && has_length_type(parameter) {
        check_length(name, value, parameter).map_err(|err| LintContext::error_message(&err))?;
    }
    Ok(())
}

/// Whether length constraints apply to the parameter's type.
///
/// # Arguments
///
/// * `parameter` - The parameter declaration.
fn has_length_type(parameter: &Parameter) -> bool {
    matches!(parameter.parameter_type, DataType::String | DataType::SecureString | DataType::Array)
}

/// Whether a parameter has a secure type.
///
/// # Arguments
///
/// * `parameter` - The parameter declaration.
fn is_secure(parameter: &Parameter) -> bool {
    matches!(parameter.parameter_type, DataType::SecureString | DataType::SecureObject)
}

/// Report declared parameters that no expression references.
fn no_unused_parameters(context: &LintContext, sink: &mut DiagnosticSink) {
    let Some(parameters) = &context.config.parameters else {
        return;
    };
    let referenced = referenced_names(context, "parameters");
    for name in parameters.keys() {
        if !referenced.contains(name) {
            sink.report(&parameter_pointer(name), t!("lint.rules.parameter.noUnusedParameters", name = name).to_string());
        }
    }
}

/// Report declared variables that no expression references.
fn no_unused_variables(context: &LintContext, sink: &mut DiagnosticSink) {
    let Some(variables) = &context.config.variables else {
        return;
    };
    let referenced = referenced_names(context, "variables");
    for name in variables.keys() {
        if !referenced.contains(name) {
            sink.report(&JsonPointer::root().push("variables").push(name), t!("lint.rules.parameter.noUnusedVariables", name = name).to_string());
        }
    }
}

/// Report literal default values that don't satisfy the parameter's type or constraints.
fn parameter_default_type(context: &LintContext, sink: &mut DiagnosticSink) {
    let Some(parameters) = &context.config.parameters else {
        return;
    };
    for (name, parameter) in parameters {
        let Some(value) = &parameter.default_value else {
            continue;
        };
        if value.as_str().is_some_and(LintContext::is_expression) {
            continue;
        }
        if let Err(reason) = check_value(name, value, parameter) {
            sink.report(
                &parameter_pointer(name).push("defaultValue"),
                t!("lint.rules.parameter.parameterDefaultType", name = name, reason = reason).to_string(),
            );
        }
    }
}

/// Report constraints that contradict each other or don't apply to the parameter's type.
fn invalid_parameter_constraints(context: &LintContext, sink: &mut DiagnosticSink) {
    let Some(parameters) = &context.config.parameters else {
        return;
    };
    for (name, parameter) in parameters {
        let base = parameter_pointer(name);
        let type_name = &parameter.parameter_type;
        if let (Some(min), Some(max)) = (parameter.min_value, parameter.max_value)
            && min > max {
                sink.report(&base.push("minValue"), t!("lint.rules.parameter.minValueExceedsMaxValue", name = name, min = min, max = max).to_string());
            }
        if let (Some(min), Some(max)) = (parameter.min_length, parameter.max_length)
            && min > max {
                sink.report(&base.push("minLength"), t!("lint.rules.parameter.minLengthExceedsMaxLength", name = name, min = min, max = max).to_string());
            }
        if *type_name != DataType::Int {
            for (constraint, limit) in [("minValue", parameter.min_value), ("maxValue", parameter.max_value)] {
                if limit.is_some() {
                    sink.report(&base.push(constraint), t!("lint.rules.parameter.valueLimitOnNonInt", name = name, constraint = constraint, type_name = type_name).to_string());
                }
            }
        }
        for (constraint, limit) in [("minLength", parameter.min_length), ("maxLength", parameter.max_length)] {
            let Some(limit) = limit else {
                continue;
            };
            if !has_length_type(parameter) {
                sink.report(&base.push(constraint), t!("lint.rules.parameter.lengthLimitOnOtherType", name = name, constraint = constraint, type_name = type_name).to_string());
            } else if limit < 0 {
                sink.report(&base.push(constraint), t!("lint.rules.parameter.negativeLength", name = name, constraint = constraint).to_string());
            }
        }
        if let Some(allowed_values) = &parameter.allowed_values {
            if allowed_values.is_empty() {
                sink.report(&base.push("allowedValues"), t!("lint.rules.parameter.emptyAllowedValues", name = name).to_string());
            }
            for (position, allowed) in allowed_values.iter().enumerate() {
                if !value_matches_type(allowed, type_name) {
                    sink.report(
                        &base.push("allowedValues").push_index(position),
                        t!("lint.rules.parameter.allowedValueTypeMismatch", name = name, type_name = type_name).to_string(),
                    );
                }
            }
        }
    }
}

/// Report secure parameters with a hard-coded default value.
fn secure_parameter_default(context: &LintContext, sink: &mut DiagnosticSink) {
    let Some(parameters) = &context.config.parameters else {
        return;
    };
    for (name, parameter) in parameters {
        if !is_secure(parameter) {
            continue;
        }
        let Some(value) = &parameter.default_value else {
            continue;
        };
        let harmless = value.as_str().is_some_and(|text| text.is_empty() || LintContext::is_expression(text));
        if !harmless {
            sink.report(&parameter_pointer(name).push("defaultValue"), t!("lint.rules.parameter.secureParameterDefault", name = name).to_string());
        }
    }
}

/// Report parameters whose names suggest a secret but whose type isn't secure.
fn prefer_secure_type(context: &LintContext, sink: &mut DiagnosticSink) {
    let Some(parameters) = &context.config.parameters else {
        return;
    };
    for (name, parameter) in parameters {
        let secure_type = match parameter.parameter_type {
            DataType::String => DataType::SecureString,
            DataType::Object => DataType::SecureObject,
            _ => continue,
        };
        if SECRET_NAME_PATTERN.is_match(name) {
            sink.report(
                &parameter_pointer(name),
                t!("lint.rules.parameter.preferSecureType", name = name, secure_type = secure_type, type_name = parameter.parameter_type).to_string(),
            );
        }
    }
}

/// Report outputs whose value references a secure parameter or a secret.
fn secure_value_in_output(context: &LintContext, sink: &mut DiagnosticSink) {
    let Some(outputs) = &context.config.outputs else {
        return;
    };
    for (name, output) in outputs {
        if !matches!(output.value_or_copy, ValueOrCopy::Value(_)) {
            continue;
        }
        let pointer = JsonPointer::root().push("outputs").push(name).push("value");
        let calls = context.expressions.calls().into_iter().filter(|call| call.site.pointer == pointer);
        for call in calls {
            let source = match call.function.name() {
                "parameters" => {
                    let Some(parameter_name) = literal_string_arg(call.function, 0) else {
                        continue;
                    };
                    if !context.declared_parameter(parameter_name).is_some_and(is_secure) {
                        continue;
                    }
                    format!("parameters('{parameter_name}')")
                },
                "secret" => "secret()".to_string(),
                _ => continue,
            };
            sink.report(&pointer, t!("lint.rules.parameter.secureValueInOutput", name = name, source = source).to_string());
            break;
        }
    }
}

/// Report parameters without a description.
fn use_parameter_description(context: &LintContext, sink: &mut DiagnosticSink) {
    let Some(parameters) = &context.config.parameters else {
        return;
    };
    for (name, parameter) in parameters {
        if parameter.description.as_ref().is_none_or(|description| description.trim().is_empty()) {
            sink.report(&parameter_pointer(name), t!("lint.rules.parameter.useParameterDescription", name = name).to_string());
        }
    }
}

/// Report supplied parameter values that don't match a declared parameter.
fn undeclared_parameter_value(context: &LintContext, sink: &mut DiagnosticSink) {
    let Some(supplied) = &context.supplied_parameters else {
        return;
    };
    for name in supplied.keys() {
        if context.declared_parameter(name).is_none() {
            sink.report(&JsonPointer::root().push("parameters"), t!("lint.rules.parameter.undeclaredParameterValue", name = name).to_string());
        }
    }
}

/// Report declared parameters without a default that weren't supplied a value.
fn missing_parameter_value(context: &LintContext, sink: &mut DiagnosticSink) {
    let Some(supplied) = &context.supplied_parameters else {
        return;
    };
    let Some(parameters) = &context.config.parameters else {
        return;
    };
    for (name, parameter) in parameters {
        if parameter.default_value.is_none() && !supplied.contains_key(name) {
            sink.report(&parameter_pointer(name), t!("lint.rules.parameter.missingParameterValue", name = name).to_string());
        }
    }
}

/// Report supplied parameter values that don't satisfy the parameter's type or constraints.
fn parameter_value_type(context: &LintContext, sink: &mut DiagnosticSink) {
    let Some(supplied) = &context.supplied_parameters else {
        return;
    };
    for (name, value) in supplied {
        let Some(parameter) = context.declared_parameter(name) else {
            continue;
        };
        if let Err(reason) = check_value(name, value, parameter) {
            sink.report(&parameter_pointer(name), t!("lint.rules.parameter.parameterValueType", name = name, reason = reason).to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::lint::{options::LintOptions, test_util::{lint_with, lint_yaml, rule_paths}};

    #[test]
    fn reports_unused_parameters_and_variables() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  used:
    type: string
    defaultValue: a
  unused:
    type: string
    defaultValue: b
variables:
  used: 1
  unused: 2
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[concat(parameters('used'), string(variables('used')))]"
"#);
        assert_eq!(rule_paths(&result, "no-unused-parameters"), vec!["/parameters/unused"]);
        assert_eq!(rule_paths(&result, "no-unused-variables"), vec!["/variables/unused"]);
    }

    #[test]
    fn reports_invalid_default_values() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  wrongType:
    type: int
    defaultValue: ten
  notAllowed:
    type: string
    defaultValue: c
    allowedValues: [a, b]
  tooSmall:
    type: int
    defaultValue: 1
    minValue: 5
  tooShort:
    type: string
    defaultValue: ab
    minLength: 3
  expression:
    type: int
    defaultValue: "[int('1')]"
  secureOk:
    type: secureString
    defaultValue: ""
  fine:
    type: array
    defaultValue: [1]
resources: []
"#);
        assert_eq!(
            rule_paths(&result, "parameter-default-type"),
            vec![
                "/parameters/notAllowed/defaultValue",
                "/parameters/tooShort/defaultValue",
                "/parameters/tooSmall/defaultValue",
                "/parameters/wrongType/defaultValue",
            ]
        );
    }

    #[test]
    fn reports_invalid_constraints() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  flipped:
    type: int
    minValue: 10
    maxValue: 1
  lengthOnInt:
    type: int
    minLength: 1
  valueOnString:
    type: string
    maxValue: 1
  negative:
    type: string
    maxLength: -1
  empty:
    type: string
    allowedValues: []
  mixed:
    type: int
    allowedValues: [1, two]
resources: []
"#);
        assert_eq!(
            rule_paths(&result, "invalid-parameter-constraints"),
            vec![
                "/parameters/empty/allowedValues",
                "/parameters/flipped/minValue",
                "/parameters/lengthOnInt/minLength",
                "/parameters/mixed/allowedValues/1",
                "/parameters/negative/maxLength",
                "/parameters/valueOnString/maxValue",
            ]
        );
    }

    #[test]
    fn reports_secure_defaults_and_secret_like_names() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  adminPassword:
    type: string
    defaultValue: ""
  token:
    type: secureString
    defaultValue: hardcoded
  apiKey:
    type: secureString
  config:
    type: object
    defaultValue: {}
resources: []
"#);
        assert_eq!(rule_paths(&result, "secure-parameter-default"), vec!["/parameters/token/defaultValue"]);
        assert_eq!(rule_paths(&result, "prefer-secure-type"), vec!["/parameters/adminPassword"]);
    }

    #[test]
    fn reports_secure_values_in_outputs() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  password:
    type: secureString
  plain:
    type: string
    defaultValue: x
outputs:
  leaked:
    type: string
    value: "[parameters('password')]"
  viaSecret:
    type: string
    value: "[secret('vault')]"
  fine:
    type: string
    value: "[parameters('plain')]"
resources: []
"#);
        assert_eq!(rule_paths(&result, "secure-value-in-output"), vec!["/outputs/leaked/value", "/outputs/viaSecret/value"]);
    }

    #[test]
    fn checks_supplied_parameter_values() {
        let options = LintOptions {
            discovery: false,
            use_settings: false,
            parameters: Some(json!({"parameters": {"count": "five", "extra": 1}})),
            ..Default::default()
        };
        let result = lint_with(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  count:
    type: int
  required:
    type: string
  optional:
    type: string
    defaultValue: x
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[concat(string(parameters('count')), parameters('required'), parameters('optional'))]"
"#, options);
        assert_eq!(rule_paths(&result, "undeclared-parameter-value"), vec!["/parameters"]);
        assert_eq!(rule_paths(&result, "missing-parameter-value"), vec!["/parameters/required"]);
        assert_eq!(rule_paths(&result, "parameter-value-type"), vec!["/parameters/count"]);
    }

    #[test]
    fn parameter_value_rules_need_supplied_parameters() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  required:
    type: string
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[parameters('required')]"
"#);
        assert!(rule_paths(&result, "missing-parameter-value").is_empty());
    }
}
