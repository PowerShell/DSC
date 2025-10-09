// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Provides helpers for working with JSON Schemas and VS Code.

/// Defines the available keywords for VS Code's extended vocabulary.
///
/// These keywords are annotation keywords that don't change the validation processing, so any
/// consumer of a schema using these keywords can safely ignore them if it doesn't understand
/// the keywords.
///
/// The transformers and generators in this library strip the VS Code keywords from canonical
/// schemas, as they are primarily for improving the development experience in a code editor, not
/// machine processing. Removing them from the canonical schemas makes the canonical schemas
/// smaller and more compatible, as some JSON Schema implementations may error on unrecognized
/// keywords instead of ignoring them.
pub const VSCODE_KEYWORDS: [&str; 11] = [
    "defaultSnippets",
    "errorMessage",
    "patternErrorMessage",
    "deprecationMessage",
    "enumDescriptions",
    "markdownEnumDescriptions",
    "markdownDescription",
    "doNotSuggest",
    "suggestSortText",
    "allowComments",
    "allowTrailingCommas",
];
