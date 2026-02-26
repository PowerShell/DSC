// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod args;

use args::{Args, SubCommand};
use clap::Parser;
use dsc_lib::dscresources::dscresource::{Capability, DscResource, ImplementedAs};
use dsc_lib::dscresources::resource_manifest::{GetMethod, Kind, ResourceManifest};
use dsc_lib::schemas::dsc_repo::DscRepoSchema;
use std::path::PathBuf;

fn main() {
    let args = Args::parse();
    match args.subcommand {
        SubCommand::List => {
            let mut resource1 = DscResource::new();
            resource1.type_name = "Test/TestResource1".parse().unwrap();
            resource1.kind = Kind::Resource;
            resource1.version = "1.0.0".to_string();
            resource1.capabilities = vec![Capability::Get, Capability::Set];
            resource1.deprecation_message = None;
            resource1.description = Some("This is a test resource.".to_string());
            resource1.implemented_as = Some(ImplementedAs::Custom("TestResource".to_string()));
            resource1.path = PathBuf::from("test_resource1");
            resource1.directory = PathBuf::from("test_directory");
            resource1.author = Some("Microsoft".to_string());
            resource1.properties = Some(vec!["Property1".to_string(), "Property2".to_string()]);
            resource1.require_adapter = Some("Test/TestGroup".parse().unwrap());
            resource1.target_resource = None;
            resource1.schema = None;
            resource1.manifest = Some(ResourceManifest {
                description: Some("This is a test resource.".to_string()),
                schema_version: dsc_lib::dscresources::resource_manifest::ResourceManifest::default_schema_id_uri(),
                resource_type: "Test/TestResource1".parse().unwrap(),
                kind: Some(Kind::Resource),
                version: "1.0.0".to_string(),
                get: Some(GetMethod {
                    executable: String::new(),
                    ..Default::default()
                }),
                ..Default::default()
            });
            let mut resource2 = DscResource::new();
            resource2.type_name = "Test/TestResource2".parse().unwrap();
            resource2.kind = Kind::Resource;
            resource2.version = "1.0.1".to_string();
            resource2.capabilities = vec![Capability::Get, Capability::Set];
            resource2.deprecation_message = None;
            resource2.description = Some("This is a test resource.".to_string());
            resource2.implemented_as = Some(ImplementedAs::Custom("TestResource".to_string()));
            resource2.path = PathBuf::from("test_resource2");
            resource2.directory = PathBuf::from("test_directory");
            resource2.author = Some("Microsoft".to_string());
            resource2.properties = Some(vec!["Property1".to_string(), "Property2".to_string()]);
            resource2.require_adapter = Some("Test/TestGroup".parse().unwrap());
            resource2.target_resource = None;
            resource2.schema = None;
            resource2.manifest = Some(ResourceManifest {
                description: Some("This is a test resource.".to_string()),
                schema_version: dsc_lib::dscresources::resource_manifest::ResourceManifest::default_schema_id_uri(),
                resource_type: "Test/TestResource2".parse().unwrap(),
                kind: Some(Kind::Resource),
                version: "1.0.1".to_string(),
                get: Some(GetMethod {
                    executable: String::new(),
                    ..Default::default()
                }),
                ..Default::default()
            });
            println!("{}", serde_json::to_string(&resource1).unwrap());
            println!("{}", serde_json::to_string(&resource2).unwrap());
        },
        SubCommand::ListMissingRequires => {
            let mut resource1 = DscResource::new();
            resource1.type_name = "Test/InvalidResource".parse().unwrap();
            resource1.kind = Kind::Resource;
            resource1.version = "1.0.0".to_string();
            resource1.capabilities = vec![Capability::Get];
            resource1.deprecation_message = None;
            resource1.description = Some("This is a test resource.".to_string());
            resource1.implemented_as = Some(ImplementedAs::Custom("TestResource".to_string()));
            resource1.path = PathBuf::from("test_resource1");
            resource1.directory = PathBuf::from("test_directory");
            resource1.author = Some("Microsoft".to_string());
            resource1.properties = Some(vec!["Property1".to_string(), "Property2".to_string()]);
            resource1.require_adapter = None;
            resource1.target_resource = None;
            resource1.manifest = None;
            resource1.schema = None;
            println!("{}", serde_json::to_string(&resource1).unwrap());
        }
    }
}
