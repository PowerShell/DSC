// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, FunctionCategory, FunctionMetadata};
use base32::{self, Alphabet};
use murmurhash64::murmur_hash64a;
use rust_i18n::t;
use serde_json::Value;
use super::Function;
use tracing::debug;

#[derive(Debug, Default)]
pub struct UniqueString {}

impl Function for UniqueString {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "uniqueString".to_string(),
            description: t!("functions.uniqueString.description").to_string(),
            category: FunctionCategory::String,
            min_args: 1,
            max_args: usize::MAX,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::String],
            ],
            remaining_arg_accepted_types: Some(vec![FunctionArgKind::String]),
            return_types: vec![FunctionArgKind::String],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.uniqueString.invoked"));
        // concatenate all string arguments into a single string with dash separator
        let concatenated = args.iter()
            .filter_map(|arg| arg.as_str())
            .collect::<Vec<&str>>()
            .join("-");
        // generate MurmurHash64 then Base32 encode it
        let hash = murmur_hash64a(concatenated.as_bytes(), 0);
        let base32_encoded = base32::encode(Alphabet::Rfc4648Lower { padding: false }, &hash.to_le_bytes());
        // return the Base32 encoded string
        Ok(Value::String(base32_encoded))
    }
}
