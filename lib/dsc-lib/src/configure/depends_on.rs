// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::config_doc::Resource;
use crate::configure::{Configuration, IntOrExpression, ProcessMode};
use crate::DscError;
use crate::parser::Statement;

use rust_i18n::t;
use serde_json::Value;
use super::context::Context;
use tracing::debug;

/// Gets the invocation order of resources based on their dependencies
///
/// # Arguments
///
/// * `config` - The configuration to get the invocation order for
///
/// # Returns
///
/// * `Result<Vec<Resource>, DscError>` - The invocation order of resources
///
/// # Errors
///
/// * `DscError::Validation` - The configuration is invalid
pub fn get_resource_invocation_order(config: &Configuration, parser: &mut Statement, context: &mut Context) -> Result<Vec<Resource>, DscError> {
    debug!("Getting resource invocation order");
    let mut order: Vec<Resource> = Vec::new();
    for resource in &config.resources {
        // validate that the resource isn't specified more than once in the config
        if config.resources.iter().filter(|r| r.name == resource.name && r.resource_type == resource.resource_type).count() > 1 {
            return Err(DscError::Validation(t!("configure.dependsOn.duplicateResource", name = resource.name, type_name = resource.resource_type).to_string()));
        }

        let mut dependency_already_in_order = true;
        // Skip dependency validation for copy loop resources here - it will be handled in unroll_and_push
        // where the copy context is properly set up for copyIndex() expressions in dependsOn
        if resource.copy.is_none() {
            if let Some(depends_on) = resource.depends_on.clone() {
                for dependency in depends_on {
                    let statement = parser.parse_and_execute(&dependency, context)?;
                    let Some(string_result) = statement.as_str() else {
                        return Err(DscError::Validation(t!("configure.dependsOn.syntaxIncorrect", dependency = dependency).to_string()));
                    };
                    let (resource_type, resource_name) = get_type_and_name(string_result)?;

                    if order.iter().any(|r| r.name == resource_name && r.resource_type == resource_type) {
                        continue;
                    }

                    let Some(dependency_resource) = config.resources.iter().find(|r| r.name.eq(&resource_name)) else {
                        return Err(DscError::Validation(t!("configure.dependsOn.dependencyNotFound", dependency_name = resource_name, resource_name = resource.name).to_string()));
                    };

                    if dependency_resource.resource_type != resource_type {
                        return Err(DscError::Validation(t!("configure.dependsOn.dependencyTypeMismatch", resource_type = resource_type, dependency_type = dependency_resource.resource_type, resource_name = resource.name).to_string()));
                    }

                    unroll_and_push(&mut order, dependency_resource, parser, context, config)?;
                    dependency_already_in_order = false;
                }
            }
        }

        if order.iter().any(|r| r.name == resource.name && r.resource_type == resource.resource_type) {
            // if dependencies were already in the order, then this might be a circular dependency
            // Skip this check for copy loop resources as their expanded names are different
            if dependency_already_in_order && resource.copy.is_none() {
                let Some(ref depends_on) = resource.depends_on else {
                  continue;
                };
                // check if the order has resource before its dependencies
                let resource_index = order.iter().position(|r| r.name == resource.name && r.resource_type == resource.resource_type).ok_or(DscError::Validation(t!("configure.dependsOn.resourceNotInOrder").to_string()))?;
                for dependency in depends_on {
                  let statement = parser.parse_and_execute(dependency, context)?;
                  let Some(string_result) = statement.as_str() else {
                      return Err(DscError::Validation(t!("configure.dependsOn.syntaxIncorrect", dependency = dependency).to_string()));
                  };
                  let (resource_type, resource_name) = get_type_and_name(string_result)?;
                  let dependency_index = order.iter().position(|r| r.name == resource_name && r.resource_type == resource_type).ok_or(DscError::Validation(t!("configure.dependsOn.dependencyNotInOrder").to_string()))?;
                  if resource_index < dependency_index {
                      return Err(DscError::Validation(t!("configure.dependsOn.circularDependency", resource_name = resource.name).to_string()));
                  }
                }
            }

            continue;
        }

        unroll_and_push(&mut order, resource, parser, context, config)?;
    }

    debug!("{}: {order:?}", t!("configure.dependsOn.invocationOrder"));
    Ok(order)
}

/// Unrolls a resource (expanding copy loops if present) and pushes it to the order list.
///
/// This function handles both regular resources and copy loop resources. For copy loop resources,
/// it expands the loop by creating individual resource instances with resolved names and properties.
///
/// # Copy Loop Handling
///
/// When a resource has a `copy` block, this function:
/// 1. Sets up the copy context (`ProcessMode::Copy` and loop name)
/// 2. Iterates `count` times, setting `copyIndex()` for each iteration
/// 3. For each iteration:
///    - Resolves dependencies that may use `copyIndex()` in their `dependsOn` expressions
///    - Evaluates the resource name expression (e.g., `[format('Policy-{0}', copyIndex())]` -> `Policy-0`)
///    - Stores the copy loop context in resource tags for later use by `reference()` function
/// 4. Clears the copy context after expansion
///
/// # Dependency Resolution in Copy Loops
///
/// When a copy loop resource depends on another copy loop resource (e.g., `Permission-0` depends on `Policy-0`),
/// the dependency must be resolved during the copy expansion phase where `copyIndex()` has the correct value.
/// This function handles this by:
/// - Evaluating `dependsOn` expressions with the current copy context
/// - Recursively expanding dependency copy loops if they haven't been expanded yet
/// - Preserving and restoring the copy context when recursing into dependencies
///
/// # Arguments
///
/// * `order` - The mutable list of resources in invocation order
/// * `resource` - The resource to unroll and push
/// * `parser` - The statement parser for evaluating expressions
/// * `context` - The evaluation context containing copy loop state
/// * `config` - The full configuration for finding dependency resources
///
/// # Returns
///
/// * `Result<(), DscError>` - Ok if successful, or an error if expansion fails
///
/// # Errors
///
/// * `DscError::Parser` - If copy count or name expressions fail to evaluate
/// * `DscError::Validation` - If dependency syntax is incorrect
fn unroll_and_push(order: &mut Vec<Resource>, resource: &Resource, parser: &mut Statement, context: &mut Context, config: &Configuration) -> Result<(), DscError> {
  // if the resource contains `Copy`, unroll it
  if let Some(copy) = &resource.copy {
      debug!("{}", t!("configure.mod.unrollingCopy", name = &copy.name, count = copy.count));
      context.process_mode = ProcessMode::Copy;
      context.copy_current_loop_name.clone_from(&copy.name);
      let mut copy_resources = Vec::<Resource>::new();
      let count: i64 = match &copy.count {
          IntOrExpression::Int(i) => *i,
          IntOrExpression::Expression(e) => {
              let Value::Number(n) = parser.parse_and_execute(e, context)? else {
                  return Err(DscError::Parser(t!("configure.mod.copyCountResultNotInteger", expression = e).to_string()))
              };
              n.as_i64().ok_or_else(|| DscError::Parser(t!("configure.mod.copyCountResultNotInteger", expression = e).to_string()))?
          },
      };
      for i in 0..count {
          context.copy.insert(copy.name.clone(), i);

          // Handle dependencies for this copy iteration
          if let Some(depends_on) = &resource.depends_on {
              for dependency in depends_on {
                  let statement = parser.parse_and_execute(dependency, context)?;
                  let Some(string_result) = statement.as_str() else {
                      return Err(DscError::Validation(t!("configure.dependsOn.syntaxIncorrect", dependency = dependency).to_string()));
                  };
                  let (resource_type, resource_name) = get_type_and_name(string_result)?;

                  // Check if the dependency is already in the order (expanded)
                  if order.iter().any(|r| r.name == resource_name && r.resource_type == resource_type) {
                      continue;
                  }

                  // Check if the dependency is also in copy_resources we're building
                  if copy_resources.iter().any(|r| r.name == resource_name && r.resource_type == resource_type) {
                      continue;
                  }

                  // Find the dependency in config.resources - it might be a copy loop template
                  // We need to find by type since the name is the template expression
                  if let Some(dependency_resource) = config.resources.iter().find(|r| r.resource_type == resource_type) {
                      // If it's a copy loop resource, we need to expand it first
                      if dependency_resource.copy.is_some() {
                          // Save current copy context
                          let saved_loop_name = context.copy_current_loop_name.clone();
                          let saved_copy = context.copy.clone();

                          // Recursively unroll the dependency
                          unroll_and_push(order, dependency_resource, parser, context, config)?;

                          // Restore copy context
                          context.copy_current_loop_name = saved_loop_name;
                          context.copy = saved_copy;
                          context.process_mode = ProcessMode::Copy;
                      } else {
                          order.push(dependency_resource.clone());
                      }
                  }
              }
          }

          let mut new_resource = resource.clone();
          let Value::String(new_name) = parser.parse_and_execute(&resource.name, context)? else {
              return Err(DscError::Parser(t!("configure.mod.copyNameResultNotString").to_string()))
          };
          new_resource.name = new_name.to_string();

          // Store copy loop context in resource tags for later use by reference()
          let mut tags = new_resource.tags.clone().unwrap_or_default();
          tags.insert(format!("__dsc_copy_loop_{}", copy.name), Value::Number(i.into()));
          new_resource.tags = Some(tags);

          new_resource.copy = None;
          copy_resources.push(new_resource);
      }
      context.process_mode = ProcessMode::Normal;
      order.extend(copy_resources);
  } else {
      order.push(resource.clone());
  }
  Ok(())
}

/// Parses a resource reference statement into type and name components.
///
/// Resource references in dependsOn and resourceId use the format "Type:Name" where
/// the name portion is URL-encoded to handle special characters.
///
/// # Arguments
/// * `statement` - A resource reference in the format "Microsoft.Resource/Type:EncodedName"
///
/// # Returns
/// A tuple of (resource_type, decoded_name) on success.
///
/// # Errors
/// Returns `DscError::Validation` if the statement doesn't contain exactly one colon
/// separator or if the name portion cannot be URL-decoded.
///
/// # Examples
/// - Input: `"Microsoft.DSC.Debug/Echo:Policy%2D0"` → Output: `("Microsoft.DSC.Debug/Echo", "Policy-0")`
fn get_type_and_name(statement: &str) -> Result<(&str, String), DscError> {
    let parts: Vec<&str> = statement.split(':').collect();
    if parts.len() != 2 {
        return Err(DscError::Validation(t!("configure.dependsOn.syntaxIncorrect", dependency = statement).to_string()));
    }
    // the name is url encoded so we need to decode it
    let decoded_name = urlencoding::decode(parts[1]).map_err(|_| DscError::Validation(t!("configure.dependsOn.syntaxIncorrect", dependency = statement).to_string()))?;
    Ok((parts[0], decoded_name.into_owned()))
}

#[cfg(test)]
mod tests {
    use crate::parser;

    use super::*;

    #[test]
    fn test_simple_order() {
        let config_yaml: &str = r#"
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Second
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','First')]"
        - name: First
          type: Test/Null
        "#;

        let config: Configuration = serde_yaml::from_str(config_yaml).unwrap();
        let mut parser = parser::Statement::new().unwrap();
        let mut context = Context::new();
        let order = get_resource_invocation_order(&config, &mut parser, &mut context).unwrap();
        assert_eq!(order[0].name, "First");
        assert_eq!(order[1].name, "Second");
    }

    #[test]
    fn test_duplicate_name() {
        let config_yaml: &str = r#"
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: First
          type: Test/Null
        - name: Second
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','First')]"
        - name: First
          type: Test/Null
        "#;

        let config: Configuration = serde_yaml::from_str(config_yaml).unwrap();
        let mut parser = parser::Statement::new().unwrap();
        let mut context = Context::new();
        let order = get_resource_invocation_order(&config, &mut parser, &mut context);
        assert!(order.is_err());
    }

    #[test]
    fn test_missing_dependency() {
        let config_yaml: &str = r#"
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Second
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','First')]"
        "#;

        let config: Configuration = serde_yaml::from_str(config_yaml).unwrap();
        let mut parser = parser::Statement::new().unwrap();
        let mut context = Context::new();
        let order = get_resource_invocation_order(&config, &mut parser, &mut context);
        assert!(order.is_err());
    }

    #[test]
    fn test_multiple_same_dependency() {
        let config_yaml: &str = r#"
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Second
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','First')]"
        - name: First
          type: Test/Null
        - name: Third
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','First')]"
        "#;

        let config: Configuration = serde_yaml::from_str(config_yaml).unwrap();
        let mut parser = parser::Statement::new().unwrap();
        let mut context = Context::new();
        let order = get_resource_invocation_order(&config, &mut parser, &mut context).unwrap();
        assert_eq!(order[0].name, "First");
        assert_eq!(order[1].name, "Second");
        assert_eq!(order[2].name, "Third");
    }

    #[test]
    fn test_circular_dependency() {
        let config_yaml: &str = r#"
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Second
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','First')]"
        - name: First
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','Second')]"
        "#;

        let config: Configuration = serde_yaml::from_str(config_yaml).unwrap();
        let mut parser = parser::Statement::new().unwrap();
        let mut context = Context::new();
        let order = get_resource_invocation_order(&config, &mut parser, &mut context);
        assert!(order.is_err());
    }

    #[test]
    fn test_multiple_dependencies() {
        let config_yaml: &str = r#"
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Third
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','First')]"
          - "[resourceId('Test/Null','Second')]"
        - name: First
          type: Test/Null
        - name: Second
          type: Test/Null
        "#;

        let config: Configuration = serde_yaml::from_str(config_yaml).unwrap();
        let mut parser = parser::Statement::new().unwrap();
        let mut context = Context::new();
        let order = get_resource_invocation_order(&config, &mut parser, &mut context).unwrap();
        assert_eq!(order[0].name, "First");
        assert_eq!(order[1].name, "Second");
        assert_eq!(order[2].name, "Third");
    }

    #[test]
    fn test_complex_circular_dependency() {
        let config_yaml: &str = r#"
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Third
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','First')]"
          - "[resourceId('Test/Null','Second')]"
        - name: First
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','Second')]"
        - name: Second
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','Third')]"
        "#;

        let config: Configuration = serde_yaml::from_str(config_yaml).unwrap();
        let mut parser = parser::Statement::new().unwrap();
        let mut context = Context::new();
        let order = get_resource_invocation_order(&config, &mut parser, &mut context);
        assert!(order.is_err());
    }

    #[test]
    fn test_complex_dependency() {
        let config_yaml: &str = r#"
        $schema: https://aka.ms/dsc/schemas/v3/bundled/config/document.json
        resources:
        - name: Third
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','First')]"
          - "[resourceId('Test/Null','Second')]"
        - name: Second
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','First')]"
        - name: First
          type: Test/Null
        - name: Fourth
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','Third')]"
        "#;

        let config: Configuration = serde_yaml::from_str(config_yaml).unwrap();
        let mut parser = parser::Statement::new().unwrap();
        let mut context = Context::new();
        let order = get_resource_invocation_order(&config, &mut parser, &mut context).unwrap();
        assert_eq!(order[0].name, "First");
        assert_eq!(order[1].name, "Second");
        assert_eq!(order[2].name, "Third");
        assert_eq!(order[3].name, "Fourth");
    }
}
