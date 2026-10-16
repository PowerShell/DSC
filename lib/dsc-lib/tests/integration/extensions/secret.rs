// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

pub(super) mod helpers {
    //! Defines helper functions for creating variants of [`SecretArgKind`], valid [`SecretArgs`],
    //! and extension manifests that define the `secret` capability.

    use dsc_lib::extensions::extension_manifest::ExtensionManifest;
    use dsc_lib::extensions::secret::{SecretArgKind, SecretArgs};
    use dsc_lib::schemas::dsc_repo::DscRepoSchema;
    use serde_json::{json, Value};

    /// Returns a static string argument definition.
    pub(super) fn string_arg(value: &str) -> SecretArgKind {
        SecretArgKind::String(value.to_string())
    }

    /// Returns the secret name input argument definition `--name`.
    pub(super) fn name_arg() -> SecretArgKind {
        SecretArgKind::Name { name_arg: "--name".to_string() }
    }

    /// Returns the vault input argument definition `--vault`.
    pub(super) fn vault_arg() -> SecretArgKind {
        SecretArgKind::Vault { vault_arg: "--vault".to_string() }
    }

    /// Returns valid arguments that define every argument kind.
    pub(super) fn args() -> SecretArgs {
        SecretArgs::from_args(vec![string_arg("get"), name_arg(), vault_arg()])
            .expect("arguments should be valid")
    }

    /// Returns an extension manifest that defines the `secret` capability with `secret`.
    pub(super) fn manifest_with_secret(secret: Value) -> Value {
        json!({
            "$schema": ExtensionManifest::default_schema_id_uri(),
            "type": "Test/Secret",
            "version": "0.1.0",
            "secret": secret
        })
    }
}

#[cfg(test)]
mod methods {
    #[cfg(test)]
    mod from_args {
        use dsc_lib::extensions::secret::{SecretArgKind, SecretArgs, SecretArgsError};
        use test_case::test_case;

        use super::super::helpers::{name_arg, string_arg, vault_arg};

        #[test_case(vec![name_arg()]; "only name arg")]
        #[test_case(vec![string_arg("get"), name_arg()]; "string arg and name arg")]
        #[test_case(vec![vault_arg(), name_arg()]; "vault arg and name arg")]
        #[test_case(vec![string_arg("get"), name_arg(), vault_arg(), string_arg("--quiet")]; "every argument kind")]
        fn valid(args: Vec<SecretArgKind>) {
            let secret_args = SecretArgs::from_args(args.clone()).expect("arguments should be valid");

            assert_eq!(secret_args, args);
        }

        #[test_case(vec![] =>
            SecretArgsError::InvalidArgs {
                errors: vec![
                    SecretArgsError::MissingNameArg,
                ],
            }; "empty args"
        )]
        #[test_case(vec![string_arg("get"), vault_arg()] =>
            SecretArgsError::InvalidArgs {
                errors: vec![
                    SecretArgsError::MissingNameArg,
                ],
            }; "missing name arg"
        )]
        #[test_case(vec![name_arg(), name_arg()] =>
            SecretArgsError::InvalidArgs {
                errors: vec![
                    SecretArgsError::MultipleNameArgs { count: 2 },
                ],
            }; "multiple name args"
        )]
        #[test_case(vec![name_arg(), vault_arg(), vault_arg()] =>
            SecretArgsError::InvalidArgs {
                errors: vec![
                    SecretArgsError::MultipleVaultArgs { count: 2 },
                ],
            }; "multiple vault args"
        )]
        #[test_case(vec![name_arg(), name_arg(), vault_arg(), vault_arg()] =>
            SecretArgsError::InvalidArgs {
                errors: vec![
                    SecretArgsError::MultipleNameArgs { count: 2 },
                    SecretArgsError::MultipleVaultArgs { count: 2 },
                ],
            }; "reports all errors in the args"
        )]
        fn invalid(args: Vec<SecretArgKind>) -> SecretArgsError {
            SecretArgs::from_args(args).unwrap_err()
        }
    }

    #[cfg(test)]
    mod from_values {
        use dsc_lib::extensions::secret::{SecretArgs, SecretArgsError};
        use serde_json::{json, Value};
        use test_case::test_case;

        use super::super::helpers::{name_arg, string_arg, vault_arg};

        #[test]
        fn converts_each_argument_kind() {
            let values = vec![json!("get"), json!({ "nameArg": "--name" }), json!({ "vaultArg": "--vault" })];

            let args = SecretArgs::from_values(values).expect("values should be valid");

            assert_eq!(args, vec![string_arg("get"), name_arg(), vault_arg()]);
        }

        #[test_case(vec![json!("get"), json!({ "nameArg": "--name", "vaultArg": "--vault" })] =>
            SecretArgsError::InvalidArgs {
                errors: vec![
                    SecretArgsError::InvalidArg {
                        position: 2,
                        value: json!({ "nameArg": "--name", "vaultArg": "--vault" }),
                    },
                    SecretArgsError::MissingNameArg,
                ],
            }; "object with name arg and vault arg"
        )]
        #[test_case(vec![json!({ "nameArg": "--name", "other": true })] =>
            SecretArgsError::InvalidArgs {
                errors: vec![
                    SecretArgsError::InvalidArg {
                        position: 1,
                        value: json!({ "nameArg": "--name", "other": true }),
                    },
                    SecretArgsError::MissingNameArg,
                ],
            }; "object with unknown property"
        )]
        #[test_case(vec![json!(5), json!({ "nameArg": "--name" })] =>
            SecretArgsError::InvalidArgs {
                errors: vec![
                    SecretArgsError::InvalidArg {
                        position: 1,
                        value: json!(5),
                    },
                ],
            }; "non-string scalar"
        )]
        #[test_case(vec![json!(5), json!(true), json!({ "nameArg": "--name" }), json!({ "nameArg": "--secret" })] =>
            SecretArgsError::InvalidArgs {
                errors: vec![
                    SecretArgsError::InvalidArg {
                        position: 1,
                        value: json!(5),
                    },
                    SecretArgsError::InvalidArg {
                        position: 2,
                        value: json!(true),
                    },
                    SecretArgsError::MultipleNameArgs { count: 2 },
                ],
            }; "reports all errors in the values"
        )]
        fn invalid(values: Vec<Value>) -> SecretArgsError {
            SecretArgs::from_values(values).unwrap_err()
        }
    }

    #[cfg(test)]
    mod to_command_args {
        use dsc_lib::extensions::secret::{SecretArgKind, SecretArgs};
        use test_case::test_case;

        use super::super::helpers::{name_arg, string_arg, vault_arg};

        #[test_case(
            vec![string_arg("get"), vault_arg(), name_arg(), string_arg("--quiet")], "apiToken", Some("services") =>
            vec!["get", "--vault", "services", "--name", "apiToken", "--quiet"];
            "preserves order and inserts the secret and vault names"
        )]
        #[test_case(
            vec![string_arg("get"), name_arg(), vault_arg()], "apiToken", None =>
            vec!["get", "--name", "apiToken"];
            "omits the vault arg without a vault"
        )]
        #[test_case(
            vec![name_arg()], "apiToken", Some("services") =>
            vec!["--name", "apiToken"];
            "ignores the vault without a vault arg"
        )]
        fn resolves(args: Vec<SecretArgKind>, name: &str, vault: Option<&str>) -> Vec<String> {
            SecretArgs::from_args(args)
                .expect("arguments should be valid")
                .to_command_args(name, vault)
        }
    }
}

#[cfg(test)]
mod schema {
    use std::sync::LazyLock;

    use dsc_lib::extensions::secret::SecretMethod;
    use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    use jsonschema::Validator;
    use regex::Regex;
    use schemars::{schema_for, Schema};
    use serde_json::{json, Value};
    use test_case::test_case;

    static SCHEMA: LazyLock<Schema> = LazyLock::new(|| schema_for!(SecretMethod));
    static VALIDATOR: LazyLock<Validator> =
        LazyLock::new(|| Validator::new((&*SCHEMA).as_value()).unwrap());
    static KEYWORD_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\w+(\.\w+)+$").expect("pattern is valid"));

    #[test_case("title")]
    #[test_case("description")]
    #[test_case("markdownDescription")]
    fn has_documentation_keyword(keyword: &str) {
        let schema = &*SCHEMA;
        let value = schema
            .get_keyword_as_str(keyword)
            .expect(format!("expected keyword '{keyword}' to be defined").as_str());

        assert!(
            !(&*KEYWORD_PATTERN).is_match(value),
            "Expected keyword '{keyword}' to be defined in translation, but was set to i18n key '{value}'"
        );
    }

    #[test_case("executable", "title")]
    #[test_case("executable", "description")]
    #[test_case("executable", "markdownDescription")]
    #[test_case("args", "title")]
    #[test_case("args", "description")]
    #[test_case("args", "markdownDescription")]
    fn property_has_documentation_keyword(property: &str, keyword: &str) {
        let schema = (&*SCHEMA)
            .get_property_subschema(property)
            .expect(format!("expected property '{property}' to be defined").as_str());
        let value = schema
            .get_keyword_as_str(keyword)
            .expect(format!("expected keyword '{keyword}' to be defined for property '{property}'").as_str());

        assert!(
            !(&*KEYWORD_PATTERN).is_match(value),
            "Expected keyword '{keyword}' for property '{property}' to be defined in translation, but was set to i18n key '{value}'"
        );
    }

    #[test_case("executable")]
    #[test_case("args")]
    fn requires_property(property: &str) {
        let required = (&*SCHEMA).get_required_property_names();

        assert!(required.contains(&property.to_string()), "expected '{property}' to be required, but required properties are {required:?}");
    }

    #[test]
    fn constrains_input_arguments() {
        let args = (&*SCHEMA)
            .get_property_subschema("args")
            .expect("expected property 'args' to be defined");
        assert_eq!(args.get_keyword_as_str("type"), Some("array"));

        let constraints = args
            .get_keyword_as_array("allOf")
            .expect("expected 'allOf' to be defined for property 'args'");
        assert_eq!(constraints.len(), 2, "expected one constraint per input argument rule");

        assert_eq!(constraints[0]["contains"]["required"], json!(["nameArg"]));
        assert_eq!(constraints[0]["minContains"], json!(1));
        assert_eq!(constraints[0]["maxContains"], json!(1));
        assert_eq!(constraints[1]["contains"]["required"], json!(["vaultArg"]));
        assert_eq!(constraints[1]["minContains"], json!(0));
        assert_eq!(constraints[1]["maxContains"], json!(1));

        for constraint in constraints {
            for keyword in ["title", "description", "markdownDescription", "errorMessage"] {
                let value = constraint[keyword].as_str()
                    .expect(format!("expected keyword '{keyword}' to be defined for constraint {constraint}").as_str());
                assert!(
                    !(&*KEYWORD_PATTERN).is_match(value),
                    "Expected keyword '{keyword}' to be defined in translation, but was set to i18n key '{value}'"
                );
            }
        }
    }

    #[test_case(&json!({ "executable": "secret", "args": ["get", { "nameArg": "--name" }] }) => true; "args with one name arg is valid")]
    #[test_case(&json!({ "executable": "secret", "args": [{ "vaultArg": "--vault" }, { "nameArg": "--name" }] }) => true; "args with one name arg and one vault arg is valid")]
    #[test_case(&json!({ "executable": "secret", "args": [{ "nameArg": "--name" }] }) => true; "args with only a name arg is valid")]
    #[test_case(&json!({ "executable": "secret" }) => false; "missing args is invalid")]
    #[test_case(&json!({ "args": [{ "nameArg": "--name" }] }) => false; "missing executable is invalid")]
    #[test_case(&json!({ "executable": "secret", "args": [] }) => false; "empty args is invalid")]
    #[test_case(&json!({ "executable": "secret", "args": ["get"] }) => false; "args without name arg is invalid")]
    #[test_case(&json!({ "executable": "secret", "args": [{ "nameArg": "--name" }, { "nameArg": "--secret" }] }) => false; "args with two name args is invalid")]
    #[test_case(&json!({ "executable": "secret", "args": [{ "nameArg": "--name" }, { "vaultArg": "--vault" }, { "vaultArg": "--store" }] }) => false; "args with two vault args is invalid")]
    #[test_case(&json!({ "executable": "secret", "args": [{ "nameArg": "--name", "vaultArg": "--vault" }] }) => false; "arg with name arg and vault arg in one object is invalid")]
    #[test_case(&json!({ "executable": "secret", "args": [{ "nameArg": "--name", "other": true }] }) => false; "arg with unknown property is invalid")]
    #[test_case(&json!({ "executable": "secret", "args": [5, { "nameArg": "--name" }] }) => false; "arg that is not a string or object is invalid")]
    fn validation(input_json: &Value) -> bool {
        (&*VALIDATOR).validate(input_json).is_ok()
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::extensions::extension_manifest::ExtensionManifest;
    use dsc_lib::extensions::secret::SecretArgs;
    use serde_json::{json, Value};
    use test_case::test_case;

    use super::helpers::{manifest_with_secret, name_arg, string_arg, vault_arg};

    #[test]
    fn serializing() {
        let args = SecretArgs::from_args(vec![string_arg("get"), name_arg(), vault_arg()])
            .expect("arguments should be valid");

        let actual = serde_json::to_value(&args).expect("serialization should never fail");

        pretty_assertions::assert_eq!(actual, json!(["get", { "nameArg": "--name" }, { "vaultArg": "--vault" }]));
    }

    #[test_case(json!(["get", { "nameArg": "--name" }]) => matches Ok(_); "array with one name arg is valid")]
    #[test_case(json!([{ "vaultArg": "--vault" }, { "nameArg": "--name" }]) => matches Ok(_); "array with one name arg and one vault arg is valid")]
    #[test_case(json!([]) => matches Err(_); "empty array raises error")]
    #[test_case(json!(["get"]) => matches Err(_); "array without name arg raises error")]
    #[test_case(json!([{ "nameArg": "--name" }, { "nameArg": "--secret" }]) => matches Err(_); "array with two name args raises error")]
    #[test_case(json!([{ "nameArg": "--name" }, { "vaultArg": "--vault" }, { "vaultArg": "--store" }]) => matches Err(_); "array with two vault args raises error")]
    #[test_case(json!([{ "nameArg": "--name", "vaultArg": "--vault" }]) => matches Err(_); "array with name arg and vault arg in one object raises error")]
    #[test_case(json!([{ "nameArg": "--name", "other": true }]) => matches Err(_); "array with unknown property in object raises error")]
    #[test_case(json!([5, { "nameArg": "--name" }]) => matches Err(_); "array with non-string scalar raises error")]
    #[test_case(json!("--name") => matches Err(_); "string value raises error")]
    #[test_case(json!({ "nameArg": "--name" }) => matches Err(_); "object value raises error")]
    #[test_case(Value::Null => matches Err(_); "null value raises error")]
    fn deserializing(value: Value) -> Result<SecretArgs, serde_json::Error> {
        serde_json::from_value::<SecretArgs>(value)
    }

    #[test_case(
        json!({ "executable": "secret" }),
        "missing field `args`";
        "manifest without args"
    )]
    #[test_case(
        json!({ "executable": "secret", "args": ["get"] }),
        "doesn't define the secret name input argument";
        "manifest without name arg"
    )]
    #[test_case(
        json!({ "executable": "secret", "args": [{ "nameArg": "--name" }, { "nameArg": "--secret" }] }),
        "defines the secret name input argument 2 times";
        "manifest with multiple name args"
    )]
    #[test_case(
        json!({ "executable": "secret", "args": [{ "nameArg": "--name" }, { "vaultArg": "--vault" }, { "vaultArg": "--store" }] }),
        "defines the vault input argument 2 times";
        "manifest with multiple vault args"
    )]
    #[test_case(
        json!({ "executable": "secret", "args": [{ "nameArg": "--name", "vaultArg": "--vault" }] }),
        "Argument 1 of the 'secret' command isn't valid";
        "manifest with ambiguous arg"
    )]
    fn deserializing_invalid_manifest_explains_problem(secret: Value, expected_message: &str) {
        let err = serde_json::from_value::<ExtensionManifest>(manifest_with_secret(secret)).unwrap_err();

        assert!(
            err.to_string().contains(expected_message),
            "expected error to contain '{expected_message}', but was: {err}"
        );
    }

    #[test]
    fn deserializing_invalid_manifest_explains_every_problem() {
        let secret = json!({ "executable": "secret", "args": [5, { "nameArg": "--name" }, { "nameArg": "--secret" }] });

        let err = serde_json::from_value::<ExtensionManifest>(manifest_with_secret(secret)).unwrap_err();

        let message = err.to_string();
        assert!(message.contains("Argument 1 of the 'secret' command isn't valid"), "{message}");
        assert!(message.contains("defines the secret name input argument 2 times"), "{message}");
    }

    #[test]
    fn round_trips_through_manifest() {
        let args = json!(["get", { "nameArg": "--name" }, { "vaultArg": "--vault" }]);
        let manifest = manifest_with_secret(json!({ "executable": "secret", "args": args }));

        let manifest = serde_json::from_value::<ExtensionManifest>(manifest).expect("manifest should be valid");
        let secret = manifest.secret.expect("secret should be defined");

        assert_eq!(secret.args, vec![string_arg("get"), name_arg(), vault_arg()]);
        pretty_assertions::assert_eq!(serde_json::to_value(&secret).unwrap()["args"], args);
    }
}

#[cfg(test)]
mod traits {
    #[cfg(test)]
    mod as_ref {
        use dsc_lib::extensions::secret::SecretArgKind;

        use super::super::helpers::args;

        #[test]
        fn vec_secret_arg_kind() {
            let args = args();

            let inner: &Vec<SecretArgKind> = args.as_ref();

            assert_eq!(inner.len(), 3);
        }
    }

    #[cfg(test)]
    mod deref {
        use super::super::helpers::{args, string_arg};

        #[test]
        fn vec_secret_arg_kind() {
            let args = args();

            assert_eq!(args.len(), 3);
            assert_eq!(args.first(), Some(&string_arg("get")));
            assert!(args.iter().any(|arg| arg == &string_arg("get")));
        }
    }

    #[cfg(test)]
    mod display {
        use dsc_lib::extensions::secret::SecretArgKind;
        use test_case::test_case;

        use super::super::helpers::{args, name_arg, string_arg, vault_arg};

        #[test_case(string_arg("get") => r#""get""#; "string arg displays as JSON string")]
        #[test_case(name_arg() => r#"{"nameArg":"--name"}"#; "name arg displays as JSON object")]
        #[test_case(vault_arg() => r#"{"vaultArg":"--vault"}"#; "vault arg displays as JSON object")]
        fn secret_arg_kind(arg: SecretArgKind) -> String {
            arg.to_string()
        }

        #[test]
        fn secret_args() {
            assert_eq!(args().to_string(), r#"["get",{"nameArg":"--name"},{"vaultArg":"--vault"}]"#);
        }
    }

    #[cfg(test)]
    mod from {
        use dsc_lib::extensions::secret::SecretArgKind;

        use super::super::helpers::{args, name_arg, string_arg, vault_arg};

        #[test]
        fn vec_secret_arg_kind() {
            let inner: Vec<SecretArgKind> = args().into();

            assert_eq!(inner, vec![string_arg("get"), name_arg(), vault_arg()]);
        }
    }

    #[cfg(test)]
    mod into_iterator {
        use dsc_lib::extensions::secret::SecretArgKind;

        use super::super::helpers::{args, name_arg, string_arg, vault_arg};

        #[test]
        fn secret_arg_kind() {
            let mut collected = Vec::new();
            for arg in args() {
                collected.push(arg);
            }

            assert_eq!(collected, vec![string_arg("get"), name_arg(), vault_arg()]);
        }

        #[test]
        fn secret_arg_kind_by_reference() {
            let args = args();

            let mut collected = Vec::<&SecretArgKind>::new();
            for arg in &args {
                collected.push(arg);
            }

            assert_eq!(collected, vec![&string_arg("get"), &name_arg(), &vault_arg()]);
        }
    }

    #[cfg(test)]
    mod partial_eq {
        use super::super::helpers::{args, name_arg, string_arg, vault_arg};

        #[test]
        fn secret_args_with_vec_secret_arg_kind() {
            assert_eq!(args(), vec![string_arg("get"), name_arg(), vault_arg()]);
            assert_ne!(args(), vec![name_arg()]);
        }

        #[test]
        fn vec_secret_arg_kind_with_secret_args() {
            assert_eq!(vec![string_arg("get"), name_arg(), vault_arg()], args());
            assert_ne!(vec![name_arg()], args());
        }
    }

    #[cfg(test)]
    mod try_from {
        use dsc_lib::extensions::secret::{SecretArgKind, SecretArgs, SecretArgsError};
        use serde_json::{json, Value};

        use super::super::helpers::{name_arg, string_arg};

        #[test]
        fn vec_secret_arg_kind() {
            let valid: Result<SecretArgs, SecretArgsError> = vec![string_arg("get"), name_arg()].try_into();
            let invalid: Result<SecretArgs, SecretArgsError> = Vec::<SecretArgKind>::new().try_into();

            assert!(valid.is_ok());
            assert!(invalid.is_err());
        }

        #[test]
        fn vec_value() {
            let valid: Result<SecretArgs, SecretArgsError> = vec![json!("get"), json!({ "nameArg": "--name" })].try_into();
            let invalid: Result<SecretArgs, SecretArgsError> = Vec::<Value>::new().try_into();

            assert!(valid.is_ok());
            assert!(invalid.is_err());
        }
    }
}
