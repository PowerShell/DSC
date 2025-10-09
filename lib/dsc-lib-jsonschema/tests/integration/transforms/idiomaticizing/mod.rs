// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Integration tests for idiomaticizing the generated schemas. The schemas that [`schemars`]
//! generates are sometimes non-idiomatic, especially when you use annotation keywords for variants
//! and fields.

#[cfg(test)] mod enums;
