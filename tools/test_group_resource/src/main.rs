// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod args;

use std::path::PathBuf;

use args::{Args, SubCommand};
use clap::Parser;
use dsc_lib::dscresources::resource_manifest::{ResourceManifest, GetMethod, Kind};
use dsc_lib::dscresources::dscresource::{Capability, DscResource, ImplementedAs};
use dsc_lib::schemas::dsc_repo::{DscRepoSchema, RecognizedSchemaVersion};
use dsc_lib::types::FullyQualifiedTypeName;

fn main() {
    let args = Args::parse();
    match args.subcommand {
        SubCommand::List => {
            let mut resource1 = DscResource::new();
            resource1.type_name = FullyQualifiedTypeName::new("Test/TestResource1").unwrap();
            resource1.kind = Kind::Resource;
            resource1.version = "1.0.0".to_string();
            resource1.capabilities = vec![Capability::Get, Capability::Set];
            resource1.description = Some("This is a test resource.".to_string());
            resource1.implemented_as = Some(ImplementedAs::Custom("TestResource".to_string()));
            resource1.author = Some("Microsoft".to_string());
            resource1.require_adapter = Some(FullyQualifiedTypeName::new("Test/TestGroup").unwrap());
            resource1.target_resource = None;
            resource1.manifest = Some(ResourceManifest {
                description: Some("This is a test resource.".to_string()),
                schema_version: dsc_lib::dscresources::resource_manifest::ResourceManifest::get_canonical_schema_id_uri(RecognizedSchemaVersion::V3),
                resource_type: FullyQualifiedTypeName::new("Test/TestResource1").unwrap(),
                    kind: Some(Kind::Resource),
                    version: "1.0.0".to_string(),
                    get: Some(GetMethod {
                        executable: String::new(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            );
            resource1.set_path(PathBuf::from("test_resource1"));
            resource1.set_directory(PathBuf::from("test_directory"));
            let mut resource2 = DscResource::new();
            resource2.type_name = FullyQualifiedTypeName::new("Test/TestResource2").unwrap();
            resource2.kind = Kind::Resource;
            resource2.version = "1.0.1".to_string();
            resource2.capabilities = vec![Capability::Get, Capability::Set];
            resource2.description = Some("This is a test resource.".to_string());
            resource2.implemented_as = Some(ImplementedAs::Custom("TestResource".to_string()));
            resource2.author = Some("Microsoft".to_string());
            resource2.require_adapter = Some(FullyQualifiedTypeName::new("Test/TestGroup").unwrap());
            resource2.target_resource = None;
            resource2.manifest = Some(ResourceManifest {
                    description: Some("This is a test resource.".to_string()),
                    schema_version: dsc_lib::dscresources::resource_manifest::ResourceManifest::get_canonical_schema_id_uri(RecognizedSchemaVersion::V3),
                    resource_type: FullyQualifiedTypeName::new("Test/TestResource2").unwrap(),
                    kind: Some(Kind::Resource),
                    version: "1.0.1".to_string(),
                    get: Some(GetMethod {
                        executable: String::new(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }
            );
            resource2.set_path(PathBuf::from("test_resource2"));
            resource2.set_directory(PathBuf::from("test_directory"));
            println!("{}", serde_json::to_string(&resource1).unwrap());
            println!("{}", serde_json::to_string(&resource2).unwrap());
        },
        SubCommand::ListMissingRequires => {
            let mut resource1 = DscResource::new();
            resource1.type_name = FullyQualifiedTypeName::new("InvalidResource").unwrap();
            resource1.kind = Kind::Resource;
            resource1.version = "1.0.0".to_string();
            resource1.capabilities = vec![Capability::Get];
            resource1.description = Some("This is a test resource.".to_string());
            resource1.implemented_as = Some(ImplementedAs::Custom("TestResource".to_string()));
            resource1.author = Some("Microsoft".to_string());
            resource1.require_adapter = None;
            resource1.target_resource = None;
            resource1.manifest = None;
            resource1.set_path(PathBuf::from("test_resource1"));
            resource1.set_directory(PathBuf::from("test_directory"));
            println!("{}", serde_json::to_string(&resource1).unwrap());
        }
    }
}
