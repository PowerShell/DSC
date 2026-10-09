// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

mod builders;

mod fields {
    mod ignore_settings_file;
    mod forbid_ignore_settings_file;
    mod resource_path;
    mod tracing;
}

mod resolved {
    mod field;
    mod settings;
}

mod sources {
    mod command_line;
    mod code_defaults;
    mod environment;
    mod preference_file;
    mod policy_file;
}

mod dsc_settings_scope;
mod dsc_settings;
