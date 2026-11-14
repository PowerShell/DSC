// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use dsc_lib::settings::DscSettingsScope;

#[cfg(test)]
mod methods {
    use super::*;
    use dsc_lib::settings::DscSettingsError;
    use test_case::test_case;

    // Minimal testing, see `traits::from_str` for more comprehensive tests.
    #[test_case("default" => matches Ok(DscSettingsScope::Default); "default scope is valid")]
    #[test_case("machine" => matches Ok(DscSettingsScope::Machine); "machine scope is valid")]
    #[test_case("user" => matches Ok(DscSettingsScope::User); "user scope is valid")]
    #[test_case("workspace" => matches Ok(DscSettingsScope::Workspace); "workspace scope is valid")]
    #[test_case("environment" => matches Ok(DscSettingsScope::Environment); "environment scope is valid")]
    #[test_case("cli" => matches Ok(DscSettingsScope::CommandLine); "command line scope is valid")]
    #[test_case("policy" => matches Ok(DscSettingsScope::Policy); "policy scope is valid")]
    #[test_case("unknown" => matches Err(DscSettingsError::InvalidScope{ text: _ }); "unknown scope is invalid")]
    #[test_case("" => matches Err(DscSettingsError::InvalidScope{ text: _ }); "empty string is invalid")]
    #[test_case(" " => matches Err(DscSettingsError::InvalidScope{ text: _ }); "whitespace string is invalid")]
    fn parse(input: &str) -> Result<DscSettingsScope, DscSettingsError> {
        DscSettingsScope::parse(input)
    }

    #[test_case(DscSettingsScope::Machine => true; "machine scope is")]
    #[test_case(DscSettingsScope::User => true; "user scope is")]
    #[test_case(DscSettingsScope::Workspace => true; "workspace scope is")]
    #[test_case(DscSettingsScope::Default => false; "default scope is not")]
    #[test_case(DscSettingsScope::Environment => false; "environment scope is not")]
    #[test_case(DscSettingsScope::CommandLine => false; "command line scope is not")]
    #[test_case(DscSettingsScope::Policy => true; "policy scope is")]
    fn is_file_based(scope: DscSettingsScope) -> bool {
        scope.is_file_based()
    }

    #[test_case(DscSettingsScope::Machine => true; "machine scope is")]
    #[test_case(DscSettingsScope::User => true; "user scope is")]
    #[test_case(DscSettingsScope::Workspace => true; "workspace scope is")]
    #[test_case(DscSettingsScope::Default => false; "default scope is not")]
    #[test_case(DscSettingsScope::Environment => false; "environment scope is not")]
    #[test_case(DscSettingsScope::CommandLine => false; "command line scope is not")]
    #[test_case(DscSettingsScope::Policy => false; "policy scope is not")]
    fn is_preference(scope: DscSettingsScope) -> bool {
        scope.is_preference()
    }

    #[test_case(DscSettingsScope::Machine => false; "machine scope is not")]
    #[test_case(DscSettingsScope::User => false; "user scope is not")]
    #[test_case(DscSettingsScope::Workspace => false; "workspace scope is not")]
    #[test_case(DscSettingsScope::Default => false; "default scope is not")]
    #[test_case(DscSettingsScope::Environment => false; "environment scope is not")]
    #[test_case(DscSettingsScope::CommandLine => false; "command line scope is not")]
    #[test_case(DscSettingsScope::Policy => true; "policy scope is")]
    fn is_policy(scope: DscSettingsScope) -> bool {
        scope.is_policy()
    }
}

crate::macros::test_dsc_repo_schema! {
    define_statics: {
        dsc_lib::settings::DscSettingsScope
    },
    test_meta_schema: {},
    test_docs: {
        title => "title",
        description => "description",
        markdown_description => "markdownDescription",
        markdown_enum_descriptions => "markdownEnumDescriptions"
    },
    test_validation: {
        lower_case_default_is_valid => {
            input_json: json!("default"),
            expected_valid: true,
        },
        upper_case_default_is_invalid => {
            input_json: json!("DEFAULT"),
            expected_valid: false,
        },
        mixed_case_default_is_invalid => {
            input_json: json!("DeFaUlT"),
            expected_valid: false,
        },
        lower_case_machine_is_valid => {
            input_json: json!("machine"),
            expected_valid: true,
        },
        upper_case_machine_is_invalid => {
            input_json: json!("MACHINE"),
            expected_valid: false,
        },
        mixed_case_machine_is_invalid => {
            input_json: json!("MaChInE"),
            expected_valid: false,
        },
        lower_case_user_is_valid => {
            input_json: json!("user"),
            expected_valid: true,
        },
        upper_case_user_is_invalid => {
            input_json: json!("USER"),
            expected_valid: false,
        },
        mixed_case_user_is_invalid => {
            input_json: json!("UsEr"),
            expected_valid: false,
        },
        lower_case_workspace_is_valid => {
            input_json: json!("workspace"),
            expected_valid: true,
        },
        upper_case_workspace_is_invalid => {
            input_json: json!("WORKSPACE"),
            expected_valid: false,
        },
        mixed_case_workspace_is_invalid => {
            input_json: json!("WoRkSpAcE"),
            expected_valid: false,
        },
        lower_case_environment_is_valid => {
            input_json: json!("environment"),
            expected_valid: true,
        },
        upper_case_environment_is_invalid => {
            input_json: json!("ENVIRONMENT"),
            expected_valid: false,
        },
        mixed_case_environment_is_invalid => {
            input_json: json!("EnViRoNmEnT"),
            expected_valid: false,
        },
        lower_case_cli_is_valid => {
            input_json: json!("cli"),
            expected_valid: true,
        },
        upper_case_cli_is_invalid => {
            input_json: json!("CLI"),
            expected_valid: false,
        },
        mixed_case_cli_is_invalid => {
            input_json: json!("ClI"),
            expected_valid: false,
        },
        lower_case_policy_is_valid => {
            input_json: json!("policy"),
            expected_valid: true,
        },
        upper_case_policy_is_invalid => {
            input_json: json!("POLICY"),
            expected_valid: false,
        },
        mixed_case_policy_is_invalid => {
            input_json: json!("PoLiCy"),
            expected_valid: false,
        },
        unknown_scope_string_is_invalid => {
            input_json: json!("unknown"),
            expected_valid: false,
        },
        empty_string_is_invalid => {
            input_json: json!(""),
            expected_valid: false,
        },
        null_value_is_invalid => {
            input_json: json!(null),
            expected_valid: false,
        },
        integer_value_is_invalid => {
            input_json: json!(42),
            expected_valid: false,
        },
        number_value_is_invalid => {
            input_json: json!(3.14),
            expected_valid: false,
        },
        boolean_value_is_invalid => {
            input_json: json!(true),
            expected_valid: false,
        },
        array_value_is_invalid => {
            input_json: json!(["default"]),
            expected_valid: false,
        },
        object_value_is_invalid => {
            input_json: json!({"scope": "default"}),
            expected_valid: false,
        },
    }
}

#[cfg(test)]
mod serde {
    use test_case::test_case;
    use serde_json::{json, Value};

    use super::*;

    #[test_case(&DscSettingsScope::Default, json!("default"); "default scope")]
    #[test_case(&DscSettingsScope::Machine, json!("machine"); "machine scope")]
    #[test_case(&DscSettingsScope::User, json!("user"); "user scope")]
    #[test_case(&DscSettingsScope::Workspace, json!("workspace"); "workspace scope")]
    #[test_case(&DscSettingsScope::Environment, json!("environment"); "environment scope")]
    #[test_case(&DscSettingsScope::CommandLine, json!("cli"); "command line scope")]
    #[test_case(&DscSettingsScope::Policy, json!("policy"); "policy scope")]
    fn serializing(scope: &DscSettingsScope, expected: Value) {
        let actual = serde_json::to_value(scope)
            .expect("serialization should never fail");

        pretty_assertions::assert_eq!(actual, expected);
    }

    #[test_case(vec!["default", "Default", "DEFAULT"], &DscSettingsScope::Default; "default scope")]
    #[test_case(vec!["machine", "Machine", "MACHINE"], &DscSettingsScope::Machine; "machine scope")]
    #[test_case(vec!["user", "User", "USER"], &DscSettingsScope::User; "user scope")]
    #[test_case(vec!["workspace", "Workspace", "WORKSPACE"], &DscSettingsScope::Workspace; "workspace scope")]
    #[test_case(vec!["environment", "Environment", "ENVIRONMENT"], &DscSettingsScope::Environment; "environment scope")]
    #[test_case(vec!["cli", "Cli", "CLI"], &DscSettingsScope::CommandLine; "command line scope")]
    #[test_case(vec!["policy", "Policy", "POLICY"], &DscSettingsScope::Policy; "policy scope")]
    fn deserializing_valid(values: Vec<&str>, expected: &DscSettingsScope) {
        for value in values {
            let json = json!(value);
            match serde_json::from_value::<DscSettingsScope>(json) {
                Ok(actual_scope) => pretty_assertions::assert_eq!(&actual_scope, expected),
                Err(err) => panic!("Expected '{value}' to parse as {expected}, but got error: {err}"),
            }
        }
    }

    #[test_case(json!("unknown"); "unknown string variant")]
    #[test_case(json!(""); "empty string")]
    #[test_case(json!(" "); "whitespace string")]
    #[test_case(json!(null); "null value")]
    #[test_case(json!(123); "numeric value")]
    #[test_case(json!(["default"]); "array value")]
    #[test_case(json!({ "scope": "default" }); "object value")]
    fn deserializing_invalid(value: Value) {
        assert!(serde_json::from_value::<DscSettingsScope>(value).is_err());
    }
}

#[cfg(test)]
mod traits {
    use super::*;

    #[cfg(test)]
    mod display {
        use super::*;
        use test_case::test_case;

        #[test_case(DscSettingsScope::Default => "default"; "default scope")]
        #[test_case(DscSettingsScope::Machine => "machine"; "machine scope")]
        #[test_case(DscSettingsScope::User => "user"; "user scope")]
        #[test_case(DscSettingsScope::Workspace => "workspace"; "workspace scope")]
        #[test_case(DscSettingsScope::Environment => "environment"; "environment scope")]
        #[test_case(DscSettingsScope::CommandLine => "cli"; "command line scope")]
        #[test_case(DscSettingsScope::Policy => "policy"; "policy scope")]
        fn to_string(scope: DscSettingsScope) -> String {
            scope.to_string()
        }
    }

    #[cfg(test)]
    mod from_str {
        use super::*;
        use dsc_lib::settings::DscSettingsError;
        use test_case::test_case;

        #[test_case(vec!["default", "Default", "DEFAULT"], DscSettingsScope::Default; "default scope")]
        #[test_case(vec!["machine", "Machine", "MACHINE"], DscSettingsScope::Machine; "machine scope")]
        #[test_case(vec!["user", "User", "USER"], DscSettingsScope::User; "user scope")]
        #[test_case(vec!["workspace", "Workspace", "WORKSPACE"], DscSettingsScope::Workspace; "workspace scope")]
        #[test_case(vec!["environment", "Environment", "ENVIRONMENT"], DscSettingsScope::Environment; "environment scope")]
        #[test_case(vec!["cli", "Cli", "CLI"], DscSettingsScope::CommandLine; "command line scope")]
        #[test_case(vec!["policy", "Policy", "POLICY"], DscSettingsScope::Policy; "policy scope")]
        fn parse_valid(inputs: Vec<&str>, expected: DscSettingsScope) {
            for s in inputs {
                let actual = s.parse::<DscSettingsScope>().expect("should parse");
                assert_eq!(actual, expected);
            }
        }

        #[test_case("invalid"; "invalid_scope")]
        #[test_case(""; "empty_string")]
        #[test_case(" "; "whitespace_string")]
        #[test_case("1"; "numeric_string")]
        fn parse_invalid(input: &str) {
            let err = input.parse::<DscSettingsScope>().expect_err("shouldn't parse");
            assert!(matches!(err, DscSettingsError::InvalidScope { text } if text == input));
        }
    }

    #[cfg(test)]
    mod partial_eq {
        use super::*;
        use test_case::test_case;

        #[test_case(DscSettingsScope::Default, "Default"; "default scope")]
        #[test_case(DscSettingsScope::Machine, "Machine"; "machine scope")]
        #[test_case(DscSettingsScope::User, "User"; "user scope")]
        #[test_case(DscSettingsScope::Workspace, "Workspace"; "workspace scope")]
        #[test_case(DscSettingsScope::Environment, "Environment"; "environment scope")]
        #[test_case(DscSettingsScope::CommandLine, "Cli"; "command line scope")]
        #[test_case(DscSettingsScope::Policy, "Policy"; "policy scope")]
        fn dsc_settings_scope_eq_str(scope: DscSettingsScope, s: &str) {
            assert!(scope == s);
            assert!(scope == s.to_lowercase().as_ref());
            assert!(scope == s.to_uppercase().as_ref());
            assert!(scope != "unknown");
            assert!(scope != "");
        }

        #[test_case("Default", DscSettingsScope::Default; "default scope")]
        #[test_case("Machine", DscSettingsScope::Machine; "machine scope")]
        #[test_case("User", DscSettingsScope::User; "user scope")]
        #[test_case("Workspace", DscSettingsScope::Workspace; "workspace scope")]
        #[test_case("Environment", DscSettingsScope::Environment; "environment scope")]
        #[test_case("Cli", DscSettingsScope::CommandLine; "command line scope")]
        #[test_case("Policy", DscSettingsScope::Policy; "policy scope")]
        fn str_eq_dsc_settings_scope(s: &str, scope: DscSettingsScope) {
            assert!(s == scope);
            assert!(s.to_lowercase().as_ref() == scope);
            assert!(s.to_uppercase().as_ref() == scope);
            assert!("unknown" != scope);
            assert!("" != scope);
        }

        #[test_case(DscSettingsScope::Default, "Default".to_string(); "default scope")]
        #[test_case(DscSettingsScope::Machine, "Machine".to_string(); "machine scope")]
        #[test_case(DscSettingsScope::User, "User".to_string(); "user scope")]
        #[test_case(DscSettingsScope::Workspace, "Workspace".to_string(); "workspace scope")]
        #[test_case(DscSettingsScope::Environment, "Environment".to_string(); "environment scope")]
        #[test_case(DscSettingsScope::CommandLine, "Cli".to_string(); "command line scope")]
        #[test_case(DscSettingsScope::Policy, "Policy".to_string(); "policy scope")]
        fn dsc_settings_scope_eq_string(scope: DscSettingsScope, s: String) {
            assert!(scope == s);
            assert!(scope == s.to_lowercase());
            assert!(scope == s.to_uppercase());
            assert!(scope != "unknown".to_string());
            assert!(scope != "".to_string());
        }

        #[test_case("Default".to_string(), DscSettingsScope::Default; "default scope")]
        #[test_case("Machine".to_string(), DscSettingsScope::Machine; "machine scope")]
        #[test_case("User".to_string(), DscSettingsScope::User; "user scope")]
        #[test_case("Workspace".to_string(), DscSettingsScope::Workspace; "workspace scope")]
        #[test_case("Environment".to_string(), DscSettingsScope::Environment; "environment scope")]
        #[test_case("Cli".to_string(), DscSettingsScope::CommandLine; "command line scope")]
        #[test_case("Policy".to_string(), DscSettingsScope::Policy; "policy scope")]
        fn string_eq_dsc_settings_scope(s: String, scope: DscSettingsScope) {
            assert!(s == scope);
            assert!(s.to_lowercase() == scope);
            assert!(s.to_uppercase() == scope);
            assert!("unknown".to_string() != scope);
            assert!("".to_string() != scope);
        }
    }
}
