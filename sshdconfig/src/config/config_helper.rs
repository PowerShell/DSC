use crate::config::*;
use crate::sshdconfig_error::SshdConfigError;

pub fn config_get(config: &SshdConfig) -> Result<String, SshdConfigError> {
    Ok("".to_string())
}

pub fn config_set(config: &SshdConfig) -> Result<(String, bool), SshdConfigError> {
    let mut result: SshdConfig = Default::default();
    let in_desired_state = true;
    Ok((result.to_json(), in_desired_state))
}

pub fn config_test(config: &SshdConfig) -> Result<(String, bool), SshdConfigError> {
    Ok(test_value(config)?)
}

pub fn validate_config(config: &SshdConfig) -> Result<(), SshdConfigError>{
    Ok(())
}

fn test_value(config: &SshdConfig) -> Result<(String, bool), SshdConfigError> {
    let mut result: SshdConfig = Default::default();
    let mut in_desired_state = true;
    Ok((result.to_json(), in_desired_state))
}

fn test_key(config: &SshdConfig) -> Result<(String, bool), SshdConfigError> {
    let mut result: SshdConfig = Default::default();
    let mut in_desired_state = true;
    Ok((result.to_json(), in_desired_state))
}


