use jsonschema::JSONSchema;

use crate::dscerror::DscError;
use crate::dscresources::dscresource::{Invoke};
use crate::discovery::{Discovery};
use self::config_doc::Configuration;
use self::config_result::{ConfigurationGetResult, ConfigurationSetResult, ConfigurationTestResult};

pub mod config_doc;
pub mod config_result;

pub struct Configurator {
    config: String,
    discovery: Discovery,
}

pub enum ErrorAction {
    Continue,
    Stop,
}

impl Configurator {
    pub fn new(config: &String) -> Result<Configurator, DscError> {
        let mut discovery = Discovery::new()?;
        discovery.initialize()?;
        Ok(Configurator {
            config: config.clone(),
            discovery,
        })
    }

    pub fn invoke_get(&self, _error_action: ErrorAction, _progress_callback: impl Fn() + 'static) -> Result<ConfigurationGetResult, DscError> {
        let config = self.validate_config()?;

        let mut result = ConfigurationGetResult::new();

        for resource in &config.resources {
            let dsc_resource = match self.discovery.find_resource(&resource.resource_type).next() {
                Some(dsc_resource) => dsc_resource,
                None => {
                    return Err(DscError::ResourceNotFound(resource.resource_type.clone()));
                }
            };
            let filter = serde_json::to_string(&resource.properties)?;
            let get_result = dsc_resource.get(&filter)?;
            let resource_get_result = config_result::ResourceGetResult {
                name: resource.name.clone(),
                resource_type: resource.resource_type.clone(),
                result: get_result,
            };
            result.results.push(resource_get_result);
        }

        Ok(result)
    }

    fn validate_config(&self) -> Result<Configuration, DscError> {
        let config: Configuration = serde_json::from_str(self.config.as_str())?;
        let mut errors: Vec<String> = Vec::new();
        for resource in &config.resources {
            let dsc_resource = match self.discovery.find_resource(&resource.resource_type).next() {
                Some(dsc_resource) => dsc_resource,
                None => {
                    return Err(DscError::ResourceNotFound(resource.resource_type.clone()));
                }
            };
            let input = serde_json::to_string(&resource.properties)?;
            let schema = match dsc_resource.schema() {
                Ok(schema) => schema,
                Err(DscError::SchemaNotAvailable(_) ) => {
                    // TODO: a way to emit a warning
                    continue;
                },
                Err(e) => {
                    return Err(e);
                },
            };
            let schema = serde_json::from_str(&schema)?;
            let compiled_schema = match JSONSchema::compile(&schema) {
                Ok(schema) => schema,
                Err(e) => {
                    errors.push(format!("Resource '{}' failed to compile schema: {}", resource.name, e));
                    continue;
                },
            };
            let input = serde_json::from_str(&input)?;
            match compiled_schema.validate(&input) {
                Err(err) => {
                    let mut error = format!("Resource '{}' failed validation: ", resource.name);
                    for e in err {
                        error.push_str(&format!("\n\t{} ", e));
                    }
                    errors.push(error);
                    continue;
                },
                Ok(_) => {},
            };
        
        }
        if !errors.is_empty() {
            let errors = errors.join("\n");
            return Err(DscError::InvalidConfiguration(errors));
        }

        Ok(config)
    }
}
