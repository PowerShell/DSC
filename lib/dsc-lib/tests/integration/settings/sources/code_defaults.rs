// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

crate::macros::test_dsc_repo_schema! {
    define_statics: {
        dsc_lib::settings::sources::DscSettingsCodeDefaults
    },
    test_meta_schema: {},
    test_docs: {},
    test_validation: {
        defaults_is_valid => {
            input_json: json!(dsc_lib::settings::sources::DSC_SETTINGS_CODE_DEFAULTS),
            expected_valid: true,
        }
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::settings::sources::DscSettingsCodeDefaults;

    #[test]
    fn serialization() {
        serde_json::to_string(&DscSettingsCodeDefaults::default())
            .expect("serialization should never fail");
    }

    // No test for deserialization since the code defaults are only serialized,
    // never deserialized, and the type doesn't implement `Deserialize`.
}
