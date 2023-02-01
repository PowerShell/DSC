use crate::config::config_data::Invoke;

pub mod config;
pub mod os_utils;
pub mod sshdconfig_error;

pub struct SshdManager {
    config_container: config::config_data::ConfigData,
}

impl SshdManager {
    pub fn new() -> Self {
        Self {
            config_container: config::config_data::ConfigData::new(),
        }
    }

    pub fn hello_world(&self) {
        println!("hello world from sshdManager");
    }

    pub fn import_sshd_config(&self, data: &String) {
        self.config_container.import_sshd_config(data);
    }

    pub fn import_json(&self, data: &String) {
        self.config_container.import_json(data);
    }

    pub fn get(&self, keywords: &Option<Vec<String>>) {
        self.config_container.get(&keywords);
    }

    pub fn set(&self) {
        self.config_container.set();
    }

    pub fn test(&self) {
        self.config_container.test();
    }

    pub fn get_keywords_from_text(&self, data: &String) -> Vec<String> {
        Vec::new()
    }

    pub fn get_keywords_from_json(&self, data: &String) -> Vec<String> {
        Vec::new()
    }
}

impl Default for SshdManager {
    fn default() -> Self {
        Self::new()
    }
}