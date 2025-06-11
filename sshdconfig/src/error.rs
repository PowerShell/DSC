// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SshdConfigError {
    #[error("Command: {0}")]
    CommandError(String),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Language: {0}")]
    LanguageError(#[from] tree_sitter::LanguageError),
    #[error("Parser: {0}")]
    ParserError(String),
    #[error("Parser Int: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),
}
