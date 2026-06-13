// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod args;
mod echo;

use args::Args;
use clap::Parser;
use rust_i18n::{i18n, t};
use schemars::schema_for;
use serde_json::{Map, Value};
use crate::echo::{Echo, Output, SecureObject, SecureString};

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
        match echo.output {
            Output::SecureString(s) => {
                if echo.show_secrets == Some(true) {
                    echo.output = Output::String(s.secure_string);
                } else {
                    echo.output = Output::String(SECURE_VALUE_REDACTED.to_string());
                }
            },
            Output::SecureObject(o) => {
                if echo.show_secrets == Some(true) {
                    echo.output = Output::Object(o.secure_object);
                } else {
                    echo.output = Output::String(SECURE_VALUE_REDACTED.to_string());
                }
            },
            Output::Array(ref mut arr) => {
                for item in arr.iter_mut() {
                    if echo.show_secrets == Some(true) {
                        *item = get_secure_contents(item);
                    } else {
                        *item = redact(item);
                    }
                }
            },
            _ => {}
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
    if let Some(obj) = value.as_object()
        && obj.len() == 1
        && (obj.contains_key("secureString") || obj.contains_key("secureObject")) {
            return true;
        }

    false
}

fn get_secure_contents(value: &Value) -> Value {
    if let Ok(secure_string) = serde_json::from_value::<SecureString>(value.clone()) {
        return Value::String(secure_string.secure_string);
    } else if let Ok(secure_object) = serde_json::from_value::<SecureObject>(value.clone()) {
        return Value::Object(secure_object.secure_object);
    }

    value.clone()
}

fn redact(value: &Value) -> Value {
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
