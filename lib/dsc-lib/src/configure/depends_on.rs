// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::config_doc::Resource;
use crate::configure::Configuration;
use crate::DscError;
use crate::parser::Statement;

use rust_i18n::t;
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
pub fn get_resource_invocation_order(config: &Configuration, parser: &mut Statement, context: &Context) -> Result<Vec<Resource>, DscError> {
    debug!("Getting resource invocation order");
    let mut order: Vec<Resource> = Vec::new();
    for resource in &config.resources {
        // validate that the resource isn't specified more than once in the config
        if config.resources.iter().filter(|r| r.name == resource.name && r.resource_type == resource.resource_type).count() > 1 {
            return Err(DscError::Validation(t!("configure.dependsOn.duplicateResource", name = resource.name, type_name = resource.resource_type).to_string()));
        }

        let mut dependency_already_in_order = true;
        if let Some(depends_on) = resource.depends_on.clone() {
            for dependency in depends_on {
                let statement = parser.parse_and_execute(&dependency, context)?;
                let Some(string_result) = statement.as_str() else {
                    return Err(DscError::Validation(t!("configure.dependsOn.syntaxIncorrect", dependency = dependency).to_string()));
                };
                let (resource_type, resource_name) = get_type_and_name(string_result)?;

                // find the resource by name
                let Some(dependency_resource) = config.resources.iter().find(|r| r.name.eq(&resource_name)) else {
                    return Err(DscError::Validation(t!("configure.dependsOn.dependencyNotFound", dependency_name = resource_name, resource_name = resource.name).to_string()));
                };
                // validate the type matches
                if dependency_resource.resource_type != resource_type {
                    return Err(DscError::Validation(t!("configure.dependsOn.dependencyTypeMismatch", resource_type = resource_type, dependency_type = dependency_resource.resource_type, resource_name = resource.name).to_string()));
                }
                // see if the dependency is already in the order
                if order.iter().any(|r| r.name == resource_name && r.resource_type == resource_type) {
                    continue;
                }
                // add the dependency to the order
                order.push(dependency_resource.clone());
                dependency_already_in_order = false;
            }
        }

        // make sure the resource is not already in the order
        if order.iter().any(|r| r.name == resource.name && r.resource_type == resource.resource_type) {
            // if dependencies were already in the order, then this might be a circular dependency
            if dependency_already_in_order {
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

        order.push(resource.clone());
    }

    debug!("{}: {order:?}", t!("configure.dependsOn.invocationOrder"));
    Ok(order)
}

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
        let order = get_resource_invocation_order(&config, &mut parser, &Context::new()).unwrap();
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
        let order = get_resource_invocation_order(&config, &mut parser, &Context::new());
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
        let order = get_resource_invocation_order(&config, &mut parser, &Context::new());
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
        let order = get_resource_invocation_order(&config, &mut parser, &Context::new()).unwrap();
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
        let order = get_resource_invocation_order(&config, &mut parser, &Context::new());
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
        let order = get_resource_invocation_order(&config, &mut parser, &Context::new()).unwrap();
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
        let order = get_resource_invocation_order(&config, &mut parser, &Context::new());
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
        let order = get_resource_invocation_order(&config, &mut parser, &Context::new()).unwrap();
        assert_eq!(order[0].name, "First");
        assert_eq!(order[1].name, "Second");
        assert_eq!(order[2].name, "Third");
        assert_eq!(order[3].name, "Fourth");
    }
}
