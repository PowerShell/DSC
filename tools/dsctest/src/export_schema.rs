// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
pub enum Names {
    Gijs,
    Steve,
    Tess,
}

impl Display for Names {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Names::Gijs => write!(f, "Gijs"),
            Names::Steve => write!(f, "Steve"),
            Names::Tess => write!(f, "Tess"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Schema {
    pub name: Names,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExportSchema {
    pub name: String,
}

pub fn invoke_export_schema(input: &str) -> String {
    let instances = vec![
        Schema {
            name: Names::Steve,
        },
        Schema {
            name: Names::Tess,
        },
        Schema {
            name: Names::Gijs,
        },
    ];
    let filter: ExportSchema = if !input.is_empty() {
        match serde_json::from_str(input) {
            Ok(filter) => filter,
            Err(err) => {
                eprintln!("Error JSON does not match schema: {err}");
                std::process::exit(1);
            }
        }
    } else {
        ExportSchema {
            name: "*".to_string(),
        }
    };
    let filtered_instances: Vec<Schema> = if filter.name.contains("*") {
        // convert the wildcard to a regex
        let regex = filter.name.replace("*", ".*");
        let regex = regex::Regex::new(&regex).unwrap();
        instances
            .into_iter()
            .filter(|instance| regex.is_match(&instance.name.to_string()))
            .collect()
    } else {
        instances
            .into_iter()
            .filter(|instance| instance.name.to_string() == filter.name)
            .collect()
    };
    let mut output = String::new();
    let mut count = filtered_instances.len();
    for instance in &filtered_instances {
        output.push_str(serde_json::to_string(instance).unwrap().as_str());
        if count > 1 {
            output.push('\n');
        }
        count -= 1;
    }
    output
}
