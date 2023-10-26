// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::configure::config_doc::Resource;
use crate::configure::Configuration;
use crate::DscError;

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
pub fn get_resource_invocation_order(config: &Configuration) -> Result<Vec<Resource>, DscError> {
    let mut order: Vec<Resource> = Vec::new();
    for resource in &config.resources {
        // validate that the resource isn't specified more than once in the config
        if config.resources.iter().filter(|r| r.name == resource.name && r.resource_type == resource.resource_type).count() > 1 {
            return Err(DscError::Validation(format!("Resource named '{0}' is specified more than once in the configuration", resource.name)));
        }

        let mut dependency_already_in_order = true;
        if let Some(depends_on) = resource.depends_on.clone() {
            for dependency in depends_on {
                let
                // validate dependency exists
                let Some(captures) = depends_on_regex.captures(&dependency) else {
                  return Err(DscError::Validation(format!("'dependsOn' syntax is incorrect for resource name '{0}': {dependency}", resource.name)));
                };
                let resource_type = captures.name("type").ok_or(DscError::Validation("Resource type missing".to_string()))?.as_str();
                let resource_name = captures.name("name").ok_or(DscError::Validation("Resource name missing".to_string()))?.as_str();
                // find the resource by name
                let Some(dependency_resource) = config.resources.iter().find(|r| r.name.eq(resource_name)) else {
                    return Err(DscError::Validation(format!("'dependsOn' resource name '{resource_name}' does not exist for resource named '{0}'", resource.name)));
                };
                // validate the type matches
                if dependency_resource.resource_type != resource_type {
                    return Err(DscError::Validation(format!("'dependsOn' resource type '{resource_type}' does not match resource type '{0}' for resource named '{1}'", dependency_resource.resource_type, dependency_resource.name)));
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
                let resource_index = order.iter().position(|r| r.name == resource.name && r.resource_type == resource.resource_type).ok_or(DscError::Validation("Resource not found in order".to_string()))?;
                for dependency in depends_on {
                    let Some(captures) = depends_on_regex.captures(dependency) else {
                      return Err(DscError::Validation(format!("'dependsOn' syntax is incorrect for resource name '{0}': {dependency}", resource.name)));
                    };
                    let resource_type = captures.name("type").ok_or(DscError::Validation("Resource type not found in dependency".to_string()))?.as_str();
                    let resource_name = captures.name("name").ok_or(DscError::Validation("Resource name not found in dependency".to_string()))?.as_str();
                    let dependency_index = order.iter().position(|r| r.name == resource_name && r.resource_type == resource_type).ok_or(DscError::Validation("Dependency not found in order".to_string()))?;
                    if resource_index < dependency_index {
                        return Err(DscError::Validation(format!("Circular dependency detected for resource named '{0}'", resource.name)));
                    }
                }
            }

            continue;
        }

        order.push(resource.clone());
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_order() {
        let config_yaml: &str = r#"
        $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
        resources:
        - name: Second
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','First')]"
        - name: First
          type: Test/Null
        "#;

        let config: Configuration = serde_yaml::from_str(config_yaml).unwrap();
        let order = get_resource_invocation_order(&config).unwrap();
        assert_eq!(order[0].name, "First");
        assert_eq!(order[1].name, "Second");
    }

    #[test]
    fn test_duplicate_name() {
        let config_yaml: &str = r#"
        $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
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
        let order = get_resource_invocation_order(&config);
        assert!(order.is_err());
    }

    #[test]
    fn test_missing_dependency() {
        let config_yaml: &str = r#"
        $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
        resources:
        - name: Second
          type: Test/Null
          dependsOn:
          - "[resourceId('Test/Null','First')]"
        "#;

        let config: Configuration = serde_yaml::from_str(config_yaml).unwrap();
        let order = get_resource_invocation_order(&config);
        assert!(order.is_err());
    }

    #[test]
    fn test_multiple_same_dependency() {
        let config_yaml: &str = r#"
        $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
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
        let order = get_resource_invocation_order(&config).unwrap();
        assert_eq!(order[0].name, "First");
        assert_eq!(order[1].name, "Second");
        assert_eq!(order[2].name, "Third");
    }

    #[test]
    fn test_circular_dependency() {
        let config_yaml: &str = r#"
        $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
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
        let order = get_resource_invocation_order(&config);
        assert!(order.is_err());
    }

    #[test]
    fn test_multiple_dependencies() {
        let config_yaml: &str = r#"
        $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
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
        let order = get_resource_invocation_order(&config).unwrap();
        assert_eq!(order[0].name, "First");
        assert_eq!(order[1].name, "Second");
        assert_eq!(order[2].name, "Third");
    }

    #[test]
    fn test_complex_circular_dependency() {
        let config_yaml: &str = r#"
        $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
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
        let order = get_resource_invocation_order(&config);
        assert!(order.is_err());
    }

    #[test]
    fn test_complex_dependency() {
        let config_yaml: &str = r#"
        $schema: https://raw.githubusercontent.com/PowerShell/DSC/main/schemas/2023/08/config/document.json
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
        let order = get_resource_invocation_order(&config).unwrap();
        assert_eq!(order[0].name, "First");
        assert_eq!(order[1].name, "Second");
        assert_eq!(order[2].name, "Third");
        assert_eq!(order[3].name, "Fourth");
    }
}
