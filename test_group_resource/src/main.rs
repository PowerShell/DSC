// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod args;
mod schema;

use args::{Args, SubCommand};
use clap::Parser;
use dsc_lib::dscresources::dscresource::{DscResource, ImplementedAs};
use schema::TestManifest;

fn main() {
    let args = Args::parse();
    match args.subcommand {
        SubCommand::List => {
            let resource1 = DscResource {
                type_name: "TestResource1".to_string(),
                version: "1.0.0".to_string(),
                implemented_as: ImplementedAs::Custom("TestResource".to_string()),
                path: "test_resource1".to_string(),
                directory: "test_directory".to_string(),
                author: Some("Microsoft".to_string()),
                properties: vec!["Property1".to_string(), "Property2".to_string()],
                requires: Some("Test/TestGroup".to_string()),
                manifest: Some(serde_json::to_value(TestManifest {
                    description: "This is a test resource.".to_string(),
                }).unwrap()),
            };
            let resource2 = DscResource {
                type_name: "TestResource2".to_string(),
                version: "1.0.1".to_string(),
                implemented_as: ImplementedAs::Custom("TestResource".to_string()),
                path: "test_resource2".to_string(),
                directory: "test_directory".to_string(),
                author: Some("Microsoft".to_string()),
                properties: vec!["Property1".to_string(), "Property2".to_string()],
                requires: Some("Test/TestGroup".to_string()),
                manifest: Some(serde_json::to_value(TestManifest {
                    description: "This is a test resource.".to_string(),
                }).unwrap()),
            };
            println!("{}", serde_json::to_string(&resource1).unwrap());
            println!("{}", serde_json::to_string(&resource2).unwrap());
        },
        SubCommand::ListMissingRequires => {
            let resource1 = DscResource {
                type_name: "InvalidResource".to_string(),
                version: "1.0.0".to_string(),
                implemented_as: ImplementedAs::Custom("TestResource".to_string()),
                path: "test_resource1".to_string(),
                directory: "test_directory".to_string(),
                author: Some("Microsoft".to_string()),
                properties: vec!["Property1".to_string(), "Property2".to_string()],
                requires: None,
                manifest: None,
            };
            println!("{}", serde_json::to_string(&resource1).unwrap());
        }
    }
}
