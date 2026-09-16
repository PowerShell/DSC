// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Returns the [`Schema`] for a given type.
/// 
/// This macro is only intended for use on types defined in the DSC repository.
/// It generates the JSON Schema for the type with [`schema_for!()`] and
/// then applies the [`canonicalize_refs_and_defs`] transform to get around the
/// reference/definition issues caused by schemars.
/// 
/// [`Schema`]: schemars::Schema
/// [`schema_for!()`]: schemars::schema_for!
/// [`canonicalize_refs_and_defs`]: crate::schemas::transforms::canonicalize_refs_and_defs
#[macro_export]
macro_rules! dsc_repo_schema_for {
    ($type:ty) => {{
        let mut schema = schemars::schema_for!($type);
        $crate::schemas::transforms::canonicalize_refs_and_defs(&mut schema);

        schema
    }};
}
