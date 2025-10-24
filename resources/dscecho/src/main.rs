// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod args;
mod echo;

use args::Args;
use clap::Parser;
use rust_i18n::{i18n, t};
use schemars::schema_for;
use serde_json::{Map, Value};
use crate::echo::{Echo, Output};

i18n!("locales", fallback = "en-us");

const SECURE_VALUE_REDACTED: &str = "<secureValue>";

fn main() {
    let args = Args::parse();
    if let Some(input) = args.input {
        let mut echo = match serde_json::from_str::<Echo>(&input) {
            Ok(echo) => echo,
            Err(err) => {
                eprintln!("{}: {err}", t!("main.invalidJson"));
                std::process::exit(1);
            }
        };
        if echo.show_secrets != Some(true) {
            match echo.output {
                Output::SecureString(_) | Output::SecureObject(_) => {
                    echo.output = Output::String(SECURE_VALUE_REDACTED.to_string());
                },
                Output::Array(ref mut arr) => {
                    for item in arr.iter_mut() {
                        if is_secure_value(item) {
                            *item = Value::String(SECURE_VALUE_REDACTED.to_string());
                        } else {
                            *item = redact(item);
                        }
                    }
                },
                Output::Object(ref mut obj) => {
                    obj.clone_from(redact(&Value::Object(obj.clone()))
                        .as_object()
                        .expect("Expected redact() to return a Value::Object"));                },
                _ => {}
            }
        }
        let json = serde_json::to_string(&echo).unwrap();
        println!("{json}");
        return;
    }

    let schema = schema_for!(Echo);
    let json = serde_json::to_string_pretty(&schema).unwrap();
    println!("{json}");
}

fn is_secure_value(value: &Value) -> bool {
    if let Some(obj) = value.as_object() {
        if obj.len() == 1 && (obj.contains_key("secureString") || obj.contains_key("secureObject")) {
            return true;
        }
    }
    false
}

pub fn redact(value: &Value) -> Value {
    if is_secure_value(value) {
        return Value::String(SECURE_VALUE_REDACTED.to_string());
    }

    if let Some(map) = value.as_object() {
        let mut new_map = Map::new();
        for (key, val) in map {
            new_map.insert(key.clone(), redact(val));
        }
        return Value::Object(new_map);
    }

    if let Some(array) = value.as_array() {
        let new_array: Vec<Value> = array.iter().map(redact).collect();
        return Value::Array(new_array);
    }

    value.clone()
}
