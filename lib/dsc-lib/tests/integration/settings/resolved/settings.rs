// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

crate::macros::test_dsc_repo_schema! {
    define_statics: {
        dsc_lib::settings::DscSettingsResolved,
        properties: {
            FORBID_IGNORE_SETTINGS_FILE_SCHEMA => "forbidIgnoreSettingsFile",
            IGNORE_SETTINGS_FILE_SCHEMA => "ignoreSettingsFile",
            TRACING_SCHEMA => "tracing",
            RESOURCE_PATH_SCHEMA => "resourcePath",
        }
    },
    test_meta_schema: {},
    test_docs: {
        title => "title",
        description => "description",
        markdown_description => "markdownDescription",
        forbid_ignore_settings_file_title => (FORBID_IGNORE_SETTINGS_FILE_SCHEMA, "title"),
        forbid_ignore_settings_file_description => (FORBID_IGNORE_SETTINGS_FILE_SCHEMA, "description"),
        forbid_ignore_settings_file_markdown_description => (FORBID_IGNORE_SETTINGS_FILE_SCHEMA, "markdownDescription"),
        ignore_settings_file_title => (IGNORE_SETTINGS_FILE_SCHEMA, "title"),
        ignore_settings_file_description => (IGNORE_SETTINGS_FILE_SCHEMA, "description"),
        ignore_settings_file_markdown_description => (IGNORE_SETTINGS_FILE_SCHEMA, "markdownDescription"),
        tracing_title => (TRACING_SCHEMA, "title"),
        tracing_description => (TRACING_SCHEMA, "description"),
        tracing_markdown_description => (TRACING_SCHEMA, "markdownDescription"),
        resource_path_title => (RESOURCE_PATH_SCHEMA, "title"),
        resource_path_description => (RESOURCE_PATH_SCHEMA, "description"),
        resource_path_markdown_description => (RESOURCE_PATH_SCHEMA, "markdownDescription"),
    },
    test_validation: {
        with_all_defined_is_valid => {
            input_json: json!({
                "forbidIgnoreSettingsFile": {
                    "value": false,
                    "scope": "policy"
                },
                "ignoreSettingsFile": {
                    "value": false,
                    "scope": "default"
                },
                "tracing": {
                    "level": {
                        "value": "warn",
                        "scope": "default"
                    },
                    "format": {
                        "value": "json",
                        "scope": "user"
                    }
                },
                "resourcePath": {
                    "appendEnvPath": {
                        "value": false,
                        "scope": "policy"
                    },
                    "directories": {
                        "value": ["/dsc/resources", "/dsc/extensions"],
                        "scope": "policy"
                    },
                    "restricted": {
                        "value": true,
                        "scope": "policy"
                    },
                }
            }),
            expected_valid: true,
        }
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::settings::{
        DscSettingsResolved,
        DscSettingsResolvedField,
        fields::*,
    };


    #[test] fn serializing() {
        let settings = DscSettingsResolved {
            forbid_ignore_settings_file: DscSettingsResolvedField::for_policy(false.into()),
            ignore_settings_file: DscSettingsResolvedField::for_code_default(false.into()),
            tracing: TracingResolvedSettings {
                level: DscSettingsResolvedField::for_code_default(TracingLevelField::Warn),
                format: DscSettingsResolvedField::for_user(TracingFormatField::Json)
            },
            resource_path: ResourcePathResolvedSettings {
                append_env_path: DscSettingsResolvedField::for_policy(false.into()),
                directories: DscSettingsResolvedField::for_policy(vec!["/dsc/resources".into(), "/dsc/extensions".into()]),
                restricted: DscSettingsResolvedField::for_policy(true.into()),
            },
        };

        serde_json::to_value(&settings)
            .expect("serialization should never fail");
    }

    // No tests for deserializing, as the type only implements Serialize.
}

#[cfg(test)]
mod traits {}
