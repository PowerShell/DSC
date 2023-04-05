// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::discovery::discovery_trait::{ResourceDiscovery};
use crate::dscresources::dscresource::{DscResource, ImplementedAs};
use std::process::Command;
use base64::{Engine as _, engine::{general_purpose}};
use serde_json::Value;
use crate::dscerror::DscError;

pub struct PowerShellDiscovery {
    pub resources: Vec<DscResource>,
    initialized: bool,
}

impl PowerShellDiscovery {
    pub fn new() -> PowerShellDiscovery {
        PowerShellDiscovery {
            resources: Vec::new(),
            initialized: false,
        }
    }
}

impl Default for PowerShellDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceDiscovery for PowerShellDiscovery {
    fn discover(&self) -> Box<dyn Iterator<Item = DscResource>> {

        match self.initialized {
            true => Box::new(self.resources.clone().into_iter()),
            false => Box::new(vec![].into_iter()),
        }
    }

    fn initialize(&mut self) -> Result<(), DscError>{
        if self.initialized {
            return Ok(());
        }
        
        let script_text = "$r=Get-DscResource;$m=gmo PSDesiredStateConfiguration;$r+=@{\"DebugInfo\"=@{\"ModuleVersion\"=$m.Version.ToString();\"ModulePath\"=$m.Path;\"PSVersion\"=$PSVersionTable.PSVersion.ToString();\"PSPath\"=$PSHome}};$r|ConvertTo-Json -Depth 3";

        // PowerShell's -EncodedCommand uses UTF16 wrapped into BASE64
        let v: Vec<u16> = script_text.encode_utf16().collect();
        let mut bytes: Vec<u8> = vec![];
        for character in v
        {
            let char_bytes = character.to_le_bytes();
            bytes.push(char_bytes[0]);
            bytes.push(char_bytes[1]);
        }
        let b64 = general_purpose::STANDARD.encode(&bytes);

        let exe = "pwsh.exe"; //TODO: add support for Windows PowerShell
        let exe_args = vec![
        "-NoLogo",
        "-NonInteractive",
        "-EncodedCommand",
        &b64
        ];

        //println!("    Running {} {:?}", exe, exe_args);
        let output = Command::new(exe)
            .args(exe_args)
            .output()
            .expect("failed to execute process");

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        //println!("    {}", output.status);
        //println!("    stderr: {}", String::from_utf8_lossy(&output.stderr));
        //println!("    stdout: {}", stdout);

        let sub_values: Vec<Value>  = serde_json::from_str(&stdout)?;
        for item in sub_values.iter() {
            let v = item["Name"].as_str();
            if v.is_some() // if the object has a 'Name' property - it is a resource info object
            {
                // format resource version string
                let mut version_string:String = String::new();
                let version_object = &item["Version"];
                if ! version_object.is_null()
                {
                    let v_major = version_object["Major"].as_i64().unwrap_or_else(||-1);
                    if v_major >= 0 {version_string += &v_major.to_string();}
                    let v_minor = version_object["Minor"].as_i64().unwrap_or_else(||-1);
                    if v_minor >= 0 {version_string.push_str(".");version_string.push_str(&v_minor.to_string());}
                    let v_revision = version_object["Revision"].as_i64().unwrap_or_else(||-1);
                    if v_revision >= 0 {version_string.push_str(".");version_string.push_str(&v_revision.to_string());}
                    let v_build = version_object["Build"].as_i64().unwrap_or_else(||-1);
                    if v_build >= 0 {version_string.push_str(".");version_string.push_str(&v_build.to_string());}
                }
                
                // read resource properties
                let mut properties: Vec<String> = vec![];
                let json_props: Vec<Value> = item["Properties"].as_array().unwrap().to_vec();
                for prop in json_props.iter()
                {
                    properties.push(prop["Name"].as_str().unwrap_or_default().to_owned());
                }

                // construct result - resource info object
                let resource = DscResource {
                    resource_type: Some(item["ResourceType"].as_str().unwrap_or_default().to_owned()),
                    name: v.unwrap().to_owned(),
                    friendly_name: Some(item["FriendlyName"].as_str().unwrap_or_default().to_owned()),
                    module_name: Some(item["ModuleName"].as_str().unwrap_or_default().to_owned()),
                    version: version_string,
                    path: item["Path"].as_str().unwrap_or_default().to_owned(),
                    parent_path: item["ParentPath"].as_str().unwrap_or_default().to_owned(),
                    implemented_as: ImplementedAs::PowerShell,
                    company_name: Some(item["CompanyName"].as_str().unwrap_or_default().to_owned()),
                    properties: properties,
                    ..Default::default()
                };

                self.resources.push(resource);
            }
            else { // the object is a DebugInfo that needs to be put to the Debug stream
                let debug_info = &item["DebugInfo"];
                println!("Debug: PSVersion {}",debug_info["PSVersion"].as_str().unwrap_or_default());
                println!("Debug: PSPath {}",debug_info["PSPath"].as_str().unwrap_or_default());
                println!("Debug: ModuleVersion {}",debug_info["ModuleVersion"].as_str().unwrap_or_default());
                println!("Debug: ModulePath {}",debug_info["ModulePath"].as_str().unwrap_or_default());
            }
        }

        self.initialized = true;
        Ok(())
    }
}
