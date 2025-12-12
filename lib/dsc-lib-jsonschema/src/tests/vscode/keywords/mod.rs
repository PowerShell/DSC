// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Creates a [`jsonschema::Validator`] with a specific keyword registered for unit testing
/// individual keywords.
/// 
/// The first argument must be the keyword type, like [`AllowCommentsKeyword`].
/// 
/// The second argument must be a reference to a JSON value to use as the testing schema.
/// 
/// # Examples
/// 
/// This example shows how you can pass a schema you know will fail validation to retrieve
/// the validation error.
/// 
/// ```rust
/// use serde_json::json;
/// 
/// use crate::vscode::AllowCommentsKeyword;
/// 
/// let validation_error = keyword_validator!(AllowCommentsKeyword,  &json!({
///     "allowComments": "yes"
/// })).unwrap_err().to_owned();
/// 
/// assert_eq!(
///     validation_error.instance_path().as_str(),
///     "/allowComments"
/// );
/// ```
macro_rules! keyword_validator {
    ($keyword:ident, $test_value:expr) => {
        jsonschema::options().with_keyword(
            $keyword::KEYWORD_NAME,
            $keyword::keyword_factory
        ).build($test_value)
    };
}

#[cfg(test)] mod allow_comments;
#[cfg(test)] mod allow_trailing_commas;
#[cfg(test)] mod completion_detail;
#[cfg(test)] mod default_snippets;
#[cfg(test)] mod deprecation_message;
#[cfg(test)] mod do_not_suggest;
#[cfg(test)] mod enum_descriptions;
#[cfg(test)] mod enum_details;
#[cfg(test)] mod enum_sort_texts;
#[cfg(test)] mod error_message;
#[cfg(test)] mod markdown_description;
#[cfg(test)] mod markdown_enum_descriptions;
#[cfg(test)] mod pattern_error_message;
#[cfg(test)] mod suggest_sort_text;