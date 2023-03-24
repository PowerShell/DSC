use crate::dscerror::DscError;
use crate::discovery::Discovery;
use self::config_doc::Configuration;
use self::config_result::{ConfigurationGetResult, ConfigurationSetResult, ConfigurationTestResult};

pub mod config_doc;
pub mod config_result;

pub struct Configurator {
    config: Configuration,
}

pub enum ErrorAction {
    Continue,
    Stop,
}

impl Configurator {
    pub fn new(config: &Configuration) -> Configurator {
        Configurator {
            config: config.clone(),
        }
    }

    pub fn invoke_get(&self, error_action: ErrorAction, progress_callback: impl Fn() + 'static) -> Result<ConfigurationGetResult, DscError> {
        let mut result = ConfigurationGetResult::new();
        let mut discovery = Discovery::new()?;
        discovery.initialize()?;

        for resource in &self.config.resources {
            let dsc_resource = match discovery.find_resource(&resource.name).next() {
                Some(dsc_resource) => dsc_resource,
                None => {
                    return Err(DscError::ResourceNotFound(resource.name.clone()));
                }
            };
            let get_result = dsc_resource.get(&resource.name)?;
            result.add_get_result(get_result);
        }

        Ok(result)
    }
}
