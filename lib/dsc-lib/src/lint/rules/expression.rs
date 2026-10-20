// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Rules about expressions and the functions they call.

use rust_i18n::t;

use crate::lint::{
    context::LintContext,
    diagnostic::LintCategory,
    expression_index::{literal_args, literal_string_arg, ExpressionScope, FunctionCall},
    options::RuleLevel,
    pointer::JsonPointer,
    rule::{DiagnosticSink, Rule, RuleTier},
};

/// The rules in this module.
#[must_use]
pub fn rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "invalid-expression",
            category: LintCategory::Expression,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.expression.invalidExpressionDescription").to_string(),
            check: invalid_expression,
        },
        Rule {
            id: "unknown-function",
            category: LintCategory::Expression,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.expression.unknownFunctionDescription").to_string(),
            check: unknown_function,
        },
        Rule {
            id: "invalid-argument-count",
            category: LintCategory::Expression,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.expression.invalidArgumentCountDescription").to_string(),
            check: invalid_argument_count,
        },
        Rule {
            id: "invalid-argument-type",
            category: LintCategory::Expression,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.expression.invalidArgumentTypeDescription").to_string(),
            check: invalid_argument_type,
        },
        Rule {
            id: "undefined-parameter",
            category: LintCategory::Expression,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.expression.undefinedParameterDescription").to_string(),
            check: undefined_parameter,
        },
        Rule {
            id: "undefined-variable",
            category: LintCategory::Expression,
            tier: RuleTier::Static,
            default_level: RuleLevel::Error,
            description: || t!("lint.rules.expression.undefinedVariableDescription").to_string(),
            check: undefined_variable,
        },
        Rule {
            id: "condition-not-expression",
            category: LintCategory::Expression,
            tier: RuleTier::Static,
            default_level: RuleLevel::Warning,
            description: || t!("lint.rules.expression.conditionNotExpressionDescription").to_string(),
            check: condition_not_expression,
        },
    ]
}

/// Whether a call is to `lambda()`, which the runtime dispatches specially with no arguments.
///
/// # Arguments
///
/// * `call` - The call.
fn is_lambda(call: &FunctionCall) -> bool {
    call.function.name().eq_ignore_ascii_case("lambda")
}

/// Whether a call can be checked against built-in function metadata.
///
/// User-defined functions (`namespace.member`) and `lambda()` are excluded.
///
/// # Arguments
///
/// * `context` - The lint context.
/// * `call` - The call.
fn is_checkable_builtin(context: &LintContext, call: &FunctionCall) -> bool {
    let name = call.function.name();
    !name.contains('.') && !is_lambda(call) && context.functions.get_metadata(name).is_some()
}

/// Report expressions that fail to parse.
fn invalid_expression(context: &LintContext, sink: &mut DiagnosticSink) {
    for site in context.expressions.sites() {
        if let Err(err) = &site.parsed {
            sink.report(
                &site.pointer,
                t!(
                    "lint.rules.expression.invalidExpression",
                    expression = site.text,
                    error = LintContext::error_message(err)
                ).to_string(),
            );
        }
    }
}

/// Report calls to functions that are neither built in nor declared in `functions`.
fn unknown_function(context: &LintContext, sink: &mut DiagnosticSink) {
    let user_functions = context.user_function_names();
    for call in context.expressions.calls() {
        let name = call.function.name();
        if name.contains('.') {
            if !user_functions.contains(name) {
                sink.report(&call.site.pointer, t!("lint.rules.expression.unknownUserFunction", name = name).to_string());
            }
            continue;
        }
        if is_lambda(&call) || context.functions.get_metadata(name).is_some() {
            continue;
        }
        let suggestion = context.functions.list().into_iter().map(|function| function.name).find(|known| known.eq_ignore_ascii_case(name));
        let message = match suggestion {
            Some(suggestion) => t!("lint.rules.expression.unknownFunctionSuggestion", name = name, suggestion = suggestion),
            None => t!("lint.rules.expression.unknownFunction", name = name),
        };
        sink.report(&call.site.pointer, message.to_string());
    }
}

/// Report calls that pass a number of arguments the function doesn't accept.
fn invalid_argument_count(context: &LintContext, sink: &mut DiagnosticSink) {
    for call in context.expressions.calls() {
        if !is_checkable_builtin(context, &call) {
            continue;
        }
        let count = call.function.args().map_or(0, <[_]>::len);
        if let Err(err) = context.functions.validate_arg_count(call.function.name(), count) {
            sink.report(&call.site.pointer, LintContext::error_message(&err));
        }
    }
}

/// Report calls whose literal arguments have a kind the function doesn't accept.
fn invalid_argument_type(context: &LintContext, sink: &mut DiagnosticSink) {
    for call in context.expressions.calls() {
        if !is_checkable_builtin(context, &call) {
            continue;
        }
        let args = literal_args(call.function);
        if let Err(err) = context.functions.validate_literal_args(call.function.name(), &args) {
            sink.report(&call.site.pointer, LintContext::error_message(&err));
        }
    }
}

/// Report `parameters()` calls that name an undeclared parameter.
///
/// Calls inside user-defined function bodies refer to the function's own parameters and are
/// skipped.
fn undefined_parameter(context: &LintContext, sink: &mut DiagnosticSink) {
    for call in context.expressions.calls_named("parameters") {
        if call.site.scope == ExpressionScope::UserFunction {
            continue;
        }
        let Some(name) = literal_string_arg(call.function, 0) else {
            continue;
        };
        if context.declared_parameter(name).is_none() {
            sink.report(&call.site.pointer, t!("lint.rules.expression.undefinedParameter", name = name).to_string());
        }
    }
}

/// Report `variables()` calls that name an undeclared variable.
fn undefined_variable(context: &LintContext, sink: &mut DiagnosticSink) {
    for call in context.expressions.calls_named("variables") {
        let Some(name) = literal_string_arg(call.function, 0) else {
            continue;
        };
        let declared = context.config.variables.as_ref().is_some_and(|variables| variables.contains_key(name));
        if !declared {
            sink.report(&call.site.pointer, t!("lint.rules.expression.undefinedVariable", name = name).to_string());
        }
    }
}

/// Report resource and output conditions that are plain strings rather than expressions.
fn condition_not_expression(context: &LintContext, sink: &mut DiagnosticSink) {
    for site in &context.resources {
        if let Some(condition) = &site.resource.condition
            && !LintContext::is_expression(condition) {
                sink.report(
                    &site.pointer.push("condition"),
                    t!("lint.rules.expression.conditionNotExpression", condition = condition).to_string(),
                );
            }
    }
    if let Some(outputs) = &context.config.outputs {
        for (name, output) in outputs {
            if let Some(condition) = &output.condition
                && !LintContext::is_expression(condition) {
                    sink.report(
                        &JsonPointer::root().push("outputs").push(name).push("condition"),
                        t!("lint.rules.expression.conditionNotExpression", condition = condition).to_string(),
                    );
                }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lint::test_util::{lint_yaml, rule_messages, rule_paths};

    #[test]
    fn reports_invalid_expressions() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[concat('unterminated)]"
    fine: "[[not an expression]"
"#);
        assert_eq!(rule_paths(&result, "invalid-expression"), vec!["/resources/0/properties/output"]);
    }

    #[test]
    fn reports_unknown_functions_with_case_suggestion() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
functions:
- namespace: my
  members:
    hello:
      output:
        type: string
        value: "[concat('hi', parameters('who'))]"
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      a: "[Concat('a', 'b')]"
      b: "[nosuchfunction()]"
      c: "[my.hello('x')]"
      d: "[my.missing('x')]"
      e: "[map(createArray(1), lambda('item', lambdaVariables('item')))]"
"#);
        assert_eq!(
            rule_paths(&result, "unknown-function"),
            vec!["/resources/0/properties/output/a", "/resources/0/properties/output/b", "/resources/0/properties/output/d"]
        );
        let messages = rule_messages(&result, "unknown-function");
        assert!(messages[0].contains("'concat'"), "expected a case suggestion, got {}", messages[0]);
        assert!(rule_paths(&result, "undefined-parameter").is_empty(), "user function parameters must not be checked");
    }

    #[test]
    fn reports_argument_count_and_type() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      count: "[int('1', '2')]"
      kind: "[int(true)]"
      nested: "[int(concat('1', '2'))]"
"#);
        assert_eq!(rule_paths(&result, "invalid-argument-count"), vec!["/resources/0/properties/output/count"]);
        assert_eq!(rule_paths(&result, "invalid-argument-type"), vec!["/resources/0/properties/output/kind"]);
    }

    #[test]
    fn reports_undefined_parameters_and_variables() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  known:
    type: string
    defaultValue: x
variables:
  v: 1
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output:
      p1: "[parameters('known')]"
      p2: "[parameters('unknown')]"
      v1: "[variables('v')]"
      v2: "[variables('w')]"
"#);
        assert_eq!(rule_paths(&result, "undefined-parameter"), vec!["/resources/0/properties/output/p2"]);
        assert_eq!(rule_paths(&result, "undefined-variable"), vec!["/resources/0/properties/output/v2"]);
    }

    #[test]
    fn reports_plain_string_conditions() {
        let result = lint_yaml(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
outputs:
  o:
    type: string
    value: x
    condition: "true"
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  condition: "true"
- name: b
  type: Microsoft.DSC.Debug/Echo
  condition: "[true()]"
"#);
        assert_eq!(rule_paths(&result, "condition-not-expression"), vec!["/outputs/o/condition", "/resources/0/condition"]);
    }
}
