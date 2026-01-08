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
            let resource1 = DscResource {
                type_name: "Test/TestResource1".parse().unwrap(),
                kind: Kind::Resource,
                version: "1.0.0".to_string(),
                capabilities: vec![Capability::Get, Capability::Set],
                description: Some("This is a test resource.".to_string()),
                implemented_as: ImplementedAs::Custom("TestResource".to_string()),
                path: PathBuf::from("test_resource1"),
                directory: PathBuf::from("test_directory"),
                author: Some("Microsoft".to_string()),
                properties: vec!["Property1".to_string(), "Property2".to_string()],
                require_adapter: Some("Test/TestGroup".parse().unwrap()),
                target_resource: None,
                manifest: Some(serde_json::to_value(ResourceManifest {
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
                }).unwrap()),
            };
            let resource2 = DscResource {
                type_name: "Test/TestResource2".parse().unwrap(),
                kind: Kind::Resource,
                version: "1.0.1".to_string(),
                capabilities: vec![Capability::Get, Capability::Set],
                description: Some("This is a test resource.".to_string()),
                implemented_as: ImplementedAs::Custom("TestResource".to_string()),
                path: PathBuf::from("test_resource2"),
                directory: PathBuf::from("test_directory"),
                author: Some("Microsoft".to_string()),
                properties: vec!["Property1".to_string(), "Property2".to_string()],
                require_adapter: Some("Test/TestGroup".parse().unwrap()),
                target_resource: None,
                manifest: Some(serde_json::to_value(ResourceManifest {
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
                }).unwrap()),
            };
            println!("{}", serde_json::to_string(&resource1).unwrap());
            println!("{}", serde_json::to_string(&resource2).unwrap());
        },
        SubCommand::ListMissingRequires => {
            let resource1 = DscResource {
                type_name: "InvalidResource".parse().unwrap(),
                kind: Kind::Resource,
                version: "1.0.0".to_string(),
                capabilities: vec![Capability::Get],
                description: Some("This is a test resource.".to_string()),
                implemented_as: ImplementedAs::Custom("TestResource".to_string()),
                path: PathBuf::from("test_resource1"),
                directory: PathBuf::from("test_directory"),
                author: Some("Microsoft".to_string()),
                properties: vec!["Property1".to_string(), "Property2".to_string()],
                require_adapter: None,
                target_resource: None,
                manifest: None,
            };
            println!("{}", serde_json::to_string(&resource1).unwrap());
        }
    }
}
