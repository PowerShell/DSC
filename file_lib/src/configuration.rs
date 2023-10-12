// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Algorithm;

/// Definition for a file resource configuration, including the path, hash, and content.
///
/// * `path` - The path to the file.
/// * `hash` - The hash of the file to either compare or compute.
/// * `content` - The content to use when asserting or setting the desired state.
/// * `exist` - The well-known flag indicating whether or not the file exists or should exist.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "DSC.FileConfiguration", description = "File resource configuration.")]
pub struct File {
    pub path: String,
    #[serde(rename = "hash", skip_serializing_if = "Option::is_none")]
    pub hash: Option<Hash>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(rename = "_exist", skip_serializing_if = "Option::is_none")]
    pub exist: Option<bool>,
}

impl File {
    /// Serialize the file configuration to a JSON string.
    ///
    /// # Return value
    ///
    /// The file configuration instance as a JSON string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use file_lib::configuration::*;
    /// # use file_lib::checksum::Algorithm;
    /// # let EXPECTED_PATH = "path/to/file";
    /// # let EXPECTED_ALGORITHM = Algorithm::Sha512;
    /// # let EXPECTED_CHECKSUM = "checksum-of-file";
    /// # let EXPECTED_CONTENT = "content-of-file";
    /// # let file = File {
    /// #     path: EXPECTED_PATH.to_string(),
    /// #     hash: Some(Hash {
    /// #         algorithm: EXPECTED_ALGORITHM,
    /// #         checksum: Some(EXPECTED_CHECKSUM.to_string()),
    /// #     }),
    /// #     content: Some(EXPECTED_CONTENT.to_string()),
    /// #     exist: None,
    /// # };
    /// let json = file.to_json();
    /// assert!(json.contains(EXPECTED_PATH));
    /// assert!(json.contains(EXPECTED_ALGORITHM.to_string().as_str()));
    /// assert!(json.contains(EXPECTED_CHECKSUM));
    /// assert!(json.contains(EXPECTED_CONTENT));
    /// ```
    #[must_use]
    pub fn to_json(&self) -> String {
        match serde_json::to_string(self) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("Failed to serialize to JSON: {e}");
                String::new()
            }
        }
    }

    /// Deserialize a file configuration from a JSON string.
    ///
    /// * `json` - The JSON string to deserialize.
    ///
    /// # Return value
    /// On success, the deserialized file configuration; otherwise, `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use file_lib::configuration::File;
    /// # use file_lib::checksum::Algorithm;
    /// # let EXPECTED_PATH = "path/to/file";
    /// # let EXPECTED_ALGORITHM = Algorithm::Sha512;
    /// # let EXPECTED_CHECKSUM = "checksum-of-file";
    /// # let EXPECTED_CONTENT = "content-of-file";
    /// # let JSON = format!(
    /// #     r#"{{"path":"{path}","hash":{{"algorithm":"{algorithm}","checksum":"{checksum}"}},"content":"{content}"}}"#,
    /// #     path = EXPECTED_PATH,
    /// #     algorithm = EXPECTED_ALGORITHM,
    /// #     checksum = EXPECTED_CHECKSUM,
    /// #     content = EXPECTED_CONTENT);
    /// let file = File::from_json(&JSON).unwrap();
    /// assert_eq!(file.path, EXPECTED_PATH);
    /// assert_eq!(&file.content.unwrap(), EXPECTED_CONTENT);
    ///
    /// let hash = file.hash.unwrap();
    /// assert_eq!(hash.algorithm, EXPECTED_ALGORITHM);
    /// assert_eq!(hash.checksum.unwrap(), EXPECTED_CHECKSUM);
    /// ```
    #[must_use]
    pub fn from_json(json: &str) -> Option<File> {
        match serde_json::from_str(json) {
            Ok(file) => Some(file),
            Err(e) => {
                eprintln!("Failed to deserialize from JSON: {e}");
                None
            }
        }
    }

    /// Get the JSON schema for the file resource configuration.
    ///
    /// * `pretty` - Flag indicating whether or not to pretty print the schema.
    ///
    /// # Return value
    ///
    /// The JSON schema as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use file_lib::configuration::File;
    /// let schema = File::get_schema(false);
    /// assert!(schema.unwrap().contains(r#""$schema":"http://json-schema.org/draft-07/schema#""#));
    /// ```
    ///
    /// # Errors
    ///
    /// Serialization fails if the schema cannot be generated.
    pub fn get_schema(pretty: bool) -> Result<String, serde_json::Error> {
        let schema = schemars::schema_for!(File);
        if pretty {
            serde_json::to_string_pretty(&schema)
        } else {
            serde_json::to_string(&schema)
        }
    }
}

impl Default for File {
    /// Create an empty file configuration.
    fn default() -> Self {
        Self {
            path: String::new(),
            hash: None,
            content: None,
            exist: None,
        }
    }
}

/// Definition for a hash using a given algorithm, and an optional checksum.
///
/// * `algorithm` - The algorithm to use when comparing or computing the checksum.
/// * `checksum` - The checksum to compare against or the computed result.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "DSC.HashConfiguration", description = "Hash configuration.")]
pub struct Hash {
    pub algorithm: Algorithm,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
}

impl Default for Hash {
    /// Create a default hash configuration with no checksum.
    fn default() -> Self {
        Self {
            algorithm: Algorithm::Sha512,
            checksum: None,
        }
    }
}
