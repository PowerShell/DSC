// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::collections::HashMap;

use serde_json::Value;

use crate::{
    dscerror::DscError,
    parser::{expressions::{Accessor, Expression}, functions::{Function, FunctionArg}, ParsedStatement, Statement},
};

use super::{context::ResourceSite, pointer::JsonPointer};

/// Where in the document an expression appears, which decides how its functions are resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionScope {
    /// Inside the resource with the given index in the context's resource list.
    Resource(usize),
    /// Inside the default value of the named parameter.
    Parameter(String),
    /// Inside the named variable.
    Variable(String),
    /// Inside the named output.
    Output(String),
    /// Inside the body of a user-defined function, where `parameters()` refers to the
    /// function's own parameters.
    UserFunction,
}

/// One expression found in the document.
pub struct ExpressionSite {
    /// The pointer to the string that holds the expression.
    pub pointer: JsonPointer,
    /// The expression text as written.
    pub text: String,
    /// Where the expression appears.
    pub scope: ExpressionScope,
    /// The parsed expression, or the parse error.
    pub parsed: Result<ParsedStatement, DscError>,
}

/// A function call found inside an expression.
pub struct FunctionCall<'a> {
    /// The expression the call appears in.
    pub site: &'a ExpressionSite,
    /// The call.
    pub function: &'a Function,
}

/// Every expression in a document, parsed once so rules can inspect them without re-parsing.
pub struct ExpressionIndex {
    sites: Vec<ExpressionSite>,
    by_pointer: HashMap<String, usize>,
}

impl ExpressionIndex {
    /// Find and parse every expression in a document.
    ///
    /// Strings are walked in the places DSC evaluates expressions: parameter default values,
    /// variables, outputs, user-defined function bodies, and for each resource its name,
    /// condition, `dependsOn` entries, copy counts, and properties.
    ///
    /// # Arguments
    ///
    /// * `raw` - The document.
    /// * `resources` - The resource sites of the document, in the order the context lists them.
    /// * `parser` - The parser to parse expressions with.
    #[must_use]
    pub fn build(raw: &Value, resources: &[ResourceSite<'_>], parser: &mut Statement) -> Self {
        let mut index = Self {
            sites: Vec::new(),
            by_pointer: HashMap::new(),
        };
        let root = JsonPointer::root();

        if let Some(parameters) = raw.get("parameters").and_then(Value::as_object) {
            let base = root.push("parameters");
            for (name, parameter) in parameters {
                if let Some(default_value) = parameter.get("defaultValue") {
                    index.walk(default_value, &base.push(name).push("defaultValue"), &ExpressionScope::Parameter(name.clone()), parser);
                }
            }
        }

        if let Some(variables) = raw.get("variables").and_then(Value::as_object) {
            let base = root.push("variables");
            for (name, variable) in variables {
                index.walk(variable, &base.push(name), &ExpressionScope::Variable(name.clone()), parser);
            }
        }

        if let Some(outputs) = raw.get("outputs").and_then(Value::as_object) {
            let base = root.push("outputs");
            for (name, output) in outputs {
                let scope = ExpressionScope::Output(name.clone());
                for key in ["value", "condition"] {
                    if let Some(member) = output.get(key) {
                        index.walk(member, &base.push(name).push(key), &scope, parser);
                    }
                }
            }
        }

        if let Some(functions) = raw.get("functions").and_then(Value::as_array) {
            let base = root.push("functions");
            for (position, function) in functions.iter().enumerate() {
                let Some(members) = function.get("members").and_then(Value::as_object) else {
                    continue;
                };
                for (member_name, member) in members {
                    if let Some(value) = member.get("output").and_then(|output| output.get("value")) {
                        let pointer = base.push_index(position).push("members").push(member_name).push("output").push("value");
                        index.walk(value, &pointer, &ExpressionScope::UserFunction, parser);
                    }
                }
            }
        }

        for (position, site) in resources.iter().enumerate() {
            let Some(resource) = site.pointer.resolve(raw) else {
                continue;
            };
            let scope = ExpressionScope::Resource(position);
            for key in ["name", "condition", "dependsOn", "properties"] {
                if let Some(member) = resource.get(key) {
                    index.walk(member, &site.pointer.push(key), &scope, parser);
                }
            }
            if let Some(copy) = resource.get("copy") {
                for key in ["count", "batchSize"] {
                    if let Some(member) = copy.get(key) {
                        index.walk(member, &site.pointer.push("copy").push(key), &scope, parser);
                    }
                }
            }
        }

        index
    }

    /// Walk a value, parsing every string that starts with `[`.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to walk.
    /// * `pointer` - The pointer to `value`.
    /// * `scope` - The scope every expression found belongs to.
    /// * `parser` - The parser to parse expressions with.
    fn walk(&mut self, value: &Value, pointer: &JsonPointer, scope: &ExpressionScope, parser: &mut Statement) {
        match value {
            Value::String(text) if text.starts_with('[') => {
                self.by_pointer.insert(pointer.to_string(), self.sites.len());
                self.sites.push(ExpressionSite {
                    pointer: pointer.clone(),
                    text: text.clone(),
                    scope: scope.clone(),
                    parsed: parser.parse(text),
                });
            },
            Value::Array(items) => {
                for (position, item) in items.iter().enumerate() {
                    self.walk(item, &pointer.push_index(position), scope, parser);
                }
            },
            Value::Object(members) => {
                for (key, member) in members {
                    self.walk(member, &pointer.push(key), scope, parser);
                }
            },
            _ => {},
        }
    }

    /// Every expression site in document order.
    #[must_use]
    pub fn sites(&self) -> &[ExpressionSite] {
        &self.sites
    }

    /// The expression site at a pointer, if the string there is an expression.
    ///
    /// # Arguments
    ///
    /// * `pointer` - The pointer to the string.
    #[must_use]
    pub fn site_at(&self, pointer: &JsonPointer) -> Option<&ExpressionSite> {
        self.by_pointer.get(pointer.as_str()).map(|position| &self.sites[*position])
    }

    /// Every function call in every successfully parsed expression, outermost first.
    #[must_use]
    pub fn calls(&self) -> Vec<FunctionCall<'_>> {
        let mut calls = Vec::new();
        for site in &self.sites {
            if let Ok(ParsedStatement::Expression(expression)) = &site.parsed {
                visit_expression(expression, &mut |function| calls.push(FunctionCall { site, function }));
            }
        }
        calls
    }

    /// Every call of the named function; names are case-sensitive, as at runtime.
    ///
    /// # Arguments
    ///
    /// * `name` - The function name.
    #[must_use]
    pub fn calls_named(&self, name: &str) -> Vec<FunctionCall<'_>> {
        self.calls().into_iter().filter(|call| call.function.name() == name).collect()
    }
}

/// Visit a function and, depth first, every function nested in its arguments and accessors.
///
/// # Arguments
///
/// * `expression` - The expression to visit.
/// * `visit` - Called with each function found, outermost first.
pub fn visit_expression<'a>(expression: &'a Expression, visit: &mut dyn FnMut(&'a Function)) {
    let function = expression.function();
    visit(function);
    if let Some(args) = function.args() {
        for arg in args {
            match arg {
                FunctionArg::Expression(nested) => visit_expression(nested, visit),
                FunctionArg::Lambda(lambda) => visit_expression(&lambda.body, visit),
                FunctionArg::Value(_) => {},
            }
        }
    }
    for accessor in expression.accessors() {
        if let Accessor::IndexExpression(nested) = accessor {
            visit_expression(nested, visit);
        }
    }
}

/// The literal string argument at a position of a call, if that argument is a literal string.
///
/// # Arguments
///
/// * `function` - The call.
/// * `position` - The zero-based argument position.
#[must_use]
pub fn literal_string_arg(function: &Function, position: usize) -> Option<&str> {
    match function.args()?.get(position)? {
        FunctionArg::Value(Value::String(text)) => Some(text),
        _ => None,
    }
}

/// The literal values of a call's arguments, with `None` for arguments that are nested
/// expressions.
///
/// # Arguments
///
/// * `function` - The call.
#[must_use]
pub fn literal_args(function: &Function) -> Vec<Option<&Value>> {
    function.args().map_or_else(Vec::new, |args| {
        args.iter().map(|arg| match arg {
            FunctionArg::Value(value) => Some(value),
            _ => None,
        }).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{configure::config_doc::Configuration, lint::context::collect_resource_sites};

    fn index_for(yaml: &str) -> (Value, ExpressionIndex) {
        let raw: Value = serde_yaml::from_str(yaml).unwrap();
        let config: Configuration = serde_json::from_value(raw.clone()).unwrap();
        let sites = collect_resource_sites(&config);
        let mut parser = Statement::new().unwrap();
        let index = ExpressionIndex::build(&raw, &sites, &mut parser);
        (raw, index)
    }

    #[test]
    fn indexes_expressions_in_every_scope() {
        let (_, index) = index_for(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
parameters:
  p:
    type: string
    defaultValue: "[concat('a', 'b')]"
variables:
  v: "[parameters('p')]"
functions:
- namespace: my
  members:
    twice:
      output:
        type: string
        value: "[concat(parameters('x'), parameters('x'))]"
outputs:
  o:
    type: string
    value: "[variables('v')]"
    condition: "[true()]"
resources:
- name: "[concat('n', '1')]"
  type: Microsoft.DSC.Debug/Echo
  condition: "[true()]"
  dependsOn:
  - "[resourceId('Microsoft.DSC.Debug/Echo', 'other')]"
  properties:
    output:
      nested: "[parameters('p')]"
      literal: "[[escaped]"
- name: other
  type: Microsoft.DSC.Debug/Echo
"#);
        let scopes: Vec<(&str, &ExpressionScope)> = index.sites().iter().map(|site| (site.pointer.as_str(), &site.scope)).collect();
        assert!(scopes.contains(&("/parameters/p/defaultValue", &ExpressionScope::Parameter("p".to_string()))));
        assert!(scopes.contains(&("/variables/v", &ExpressionScope::Variable("v".to_string()))));
        assert!(scopes.contains(&("/functions/0/members/twice/output/value", &ExpressionScope::UserFunction)));
        assert!(scopes.contains(&("/outputs/o/value", &ExpressionScope::Output("o".to_string()))));
        assert!(scopes.contains(&("/outputs/o/condition", &ExpressionScope::Output("o".to_string()))));
        assert!(scopes.contains(&("/resources/0/name", &ExpressionScope::Resource(0))));
        assert!(scopes.contains(&("/resources/0/condition", &ExpressionScope::Resource(0))));
        assert!(scopes.contains(&("/resources/0/dependsOn/0", &ExpressionScope::Resource(0))));
        assert!(scopes.contains(&("/resources/0/properties/output/nested", &ExpressionScope::Resource(0))));
        // escaped strings are indexed so rules can tell them apart from literals
        let escaped = index.site_at(&JsonPointer::from_pointer("/resources/0/properties/output/literal")).unwrap();
        assert!(matches!(escaped.parsed, Ok(ParsedStatement::Escaped(_))));
        assert!(index.sites().iter().all(|site| site.parsed.is_ok()));
    }

    #[test]
    fn collects_nested_calls() {
        let (_, index) = index_for(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[concat(parameters('p'), variables('v')[int('0')])]"
"#);
        let names: Vec<&str> = index.calls().iter().map(|call| call.function.name()).collect();
        assert_eq!(names, vec!["concat", "parameters", "variables", "int"]);
        let parameters = index.calls_named("parameters");
        assert_eq!(parameters.len(), 1);
        assert_eq!(literal_string_arg(parameters[0].function, 0), Some("p"));
        assert_eq!(literal_args(index.calls_named("concat")[0].function), vec![None, None]);
    }

    #[test]
    fn records_parse_failures() {
        let (_, index) = index_for(r#"
$schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
resources:
- name: a
  type: Microsoft.DSC.Debug/Echo
  properties:
    output: "[concat('unterminated)]"
"#);
        assert_eq!(index.sites().len(), 1);
        assert!(index.sites()[0].parsed.is_err());
        assert!(index.calls().is_empty());
    }
}
