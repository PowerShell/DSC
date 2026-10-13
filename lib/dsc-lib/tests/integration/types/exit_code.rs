// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    use dsc_lib::{dscerror::DscError, types::ExitCode};
    use test_case::test_case;

    #[test]
    fn new() {
        let _ = ExitCode::new(0);
    }

    #[test_case("0" => matches Ok(_); "zero is valid exit code")]
    #[test_case("1" => matches Ok(_); "positive integer is valid exit code")]
    #[test_case("-1" => matches Ok(_); "negative integer is valid exit code")]
    #[test_case("a" => matches Err(_); "arbitrary string raises error")]
    #[test_case("1.5" => matches Err(_); "floating point number raises error")]
    #[test_case("9223372036854775807" => matches Err(_); "integer outside 32-bit range raises error")]
    #[test_case("+123" => matches Err(_); "leading plus sign raises error")]
    fn parse(text: &str) -> Result<ExitCode, DscError> {
        ExitCode::parse(text)
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::types::ExitCode;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case("0"; "exit code zero")]
    #[test_case("1"; "positive integer exit code")]
    #[test_case("-1"; "negative integer exit code")]
    fn serializing(code: &str) {
        let actual = serde_json::to_string(
            &ExitCode::parse(code).expect("parse should never fail"),
        )
        .expect("serialization should never fail");

        let expected = format!(r#""{code}""#);

        pretty_assertions::assert_eq!(actual, expected);
    }

    #[test_case(json!("0") => matches Ok(_); "zero as string value is valid")]
    #[test_case(json!("1") => matches Ok(_); "positive integer as string value is valid")]
    #[test_case(json!("-1") => matches Ok(_); "negative integer string value is valid")]
    #[test_case(json!("1.2") => matches Err(_); "float as string value is invalid")]
    #[test_case(json!("abc") => matches Err(_); "arbitrary string value is invalid")]
    #[test_case(json!(true) => matches Err(_); "boolean value is invalid")]
    #[test_case(json!(1) => matches Err(_); "integer value is invalid")]
    #[test_case(json!(1.2) => matches Err(_); "float value is invalid")]
    #[test_case(json!({"code": "0"}) => matches Err(_); "object value is invalid")]
    #[test_case(json!(["0"]) => matches Err(_); "array value is invalid")]
    #[test_case(serde_json::Value::Null => matches Err(_); "null value is invalid")]
    fn deserializing(value: Value) -> Result<ExitCode, serde_json::Error> {
        serde_json::from_value::<ExitCode>(value)
    }
}

#[cfg(test)]
mod traits {
    #[cfg(test)]
    mod as_ref {
        use dsc_lib::types::ExitCode;

        #[test]
        fn i32() {
            let _: &i32 = ExitCode::new(0).as_ref();
        }
    }

    #[cfg(test)]
    mod deref {
        use dsc_lib::types::ExitCode;

        #[test]
        fn i32() {
            let c = ExitCode::new(-1);

            pretty_assertions::assert_eq!(c.abs(), 1);
        }
    }

    #[cfg(test)]
    mod borrow {
        use std::borrow::Borrow;

        use dsc_lib::types::ExitCode;

        #[test]
        fn i32() {
            let _: &i32 = ExitCode::new(0).borrow();
        }
    }

    #[cfg(test)]
    mod display {
        use dsc_lib::types::ExitCode;
        use test_case::test_case;

        #[test_case(0, "0"; "zero exit code")]
        #[test_case(1, "1"; "positive integer exit code")]
        #[test_case(-1, "-1"; "negative integer exit code")]
        fn format(code: i32, expected: &str) {
            pretty_assertions::assert_eq!(
                format!("code: '{}'", ExitCode::new(code)),
                format!("code: '{}'", expected)
            )
        }

        #[test_case(0, "0"; "zero exit code")]
        #[test_case(1, "1"; "positive integer exit code")]
        #[test_case(-1, "-1"; "negative integer exit code")]
        fn to_string(code: i32, expected: &str) {
            pretty_assertions::assert_eq!(
                ExitCode::new(code).to_string(),
                expected.to_string()
            )
        }
    }

    #[cfg(test)]
    mod from_str {
        use std::str::FromStr;

        use dsc_lib::{dscerror::DscError, types::ExitCode};
        use test_case::test_case;

        #[test_case("0" => matches Ok(_); "zero is valid exit code")]
        #[test_case("1" => matches Ok(_); "positive integer is valid exit code")]
        #[test_case("-1" => matches Ok(_); "negative integer is valid exit code")]
        #[test_case("a" => matches Err(_); "arbitrary string raises error")]
        #[test_case("1.5" => matches Err(_); "floating point number raises error")]
        #[test_case("9223372036854775807" => matches Err(_); "integer outside 32-bit range raises error")]
        fn from_str(text: &str) -> Result<ExitCode, DscError> {
            ExitCode::from_str(text)
        }

        #[test_case("0" => matches Ok(_); "zero is valid exit code")]
        #[test_case("1" => matches Ok(_); "positive integer is valid exit code")]
        #[test_case("-1" => matches Ok(_); "negative integer is valid exit code")]
        #[test_case("a" => matches Err(_); "arbitrary string raises error")]
        #[test_case("1.5" => matches Err(_); "floating point number raises error")]
        #[test_case("9223372036854775807" => matches Err(_); "integer outside 32-bit range raises error")]
        fn parse(text: &str) -> Result<ExitCode, DscError> {
            text.parse()
        }
    }

    #[cfg(test)]
    mod from {
        use dsc_lib::types::ExitCode;

        #[test]
        fn i32() {
            let _ = ExitCode::from(0);
        }
    }

    #[cfg(test)]
    mod try_from {
        use dsc_lib::{dscerror::DscError, types::ExitCode};
        use test_case::test_case;

        #[test_case("0" => matches Ok(_); "zero is valid exit code")]
        #[test_case("1" => matches Ok(_); "positive integer is valid exit code")]
        #[test_case("-1" => matches Ok(_); "negative integer is valid exit code")]
        #[test_case("a" => matches Err(_); "arbitrary string raises error")]
        #[test_case("1.5" => matches Err(_); "floating point number raises error")]
        #[test_case("9223372036854775807" => matches Err(_); "integer outside 32-bit range raises error")]
        fn string(text: &str) -> Result<ExitCode, DscError> {
            ExitCode::try_from(text.to_string())
        }

        #[test_case("0" => matches Ok(_); "zero is valid exit code")]
        #[test_case("1" => matches Ok(_); "positive integer is valid exit code")]
        #[test_case("-1" => matches Ok(_); "negative integer is valid exit code")]
        #[test_case("a" => matches Err(_); "arbitrary string raises error")]
        #[test_case("1.5" => matches Err(_); "floating point number raises error")]
        #[test_case("9223372036854775807" => matches Err(_); "integer outside 32-bit range raises error")]
        fn str(text: &str) -> Result<ExitCode, DscError> {
            ExitCode::try_from(text)
        }
    }

    #[cfg(test)]
    mod into {
        use dsc_lib::types::ExitCode;

        #[test]
        fn i32() {
            let _: i32 = ExitCode::new(0).into();
        }

        #[test]
        fn string() {
            let _: String = ExitCode::new(0).into();
        }
    }

    #[cfg(test)]
    mod partial_eq {
        use dsc_lib::types::ExitCode;
        use test_case::test_case;

        #[test_case(0, 0, true; "identical codes are equal")]
        #[test_case(-1, 1, false; "different codes are unequal")]
        fn exit_code(lhs: i32, rhs: i32, should_be_equal: bool) {
            if should_be_equal {
                pretty_assertions::assert_eq!(
                    ExitCode::new(lhs),
                    ExitCode::new(rhs)
                )
            } else {
                pretty_assertions::assert_ne!(
                    ExitCode::new(lhs),
                    ExitCode::new(rhs)
                )
            }
        }

        #[test_case(0, 0, true; "identical codes are equal")]
        #[test_case(-1, 1, false; "different codes are unequal")]
        fn i32(code_int: i32, int: i32, should_be_equal: bool) {
            let code = ExitCode::new(code_int);

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                code == int,
                should_be_equal,
                "expected comparison of {code} and {int} to be {should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                int == code,
                should_be_equal,
                "expected comparison of {int} and {code} to be {should_be_equal}"
            );
        }
    }
}
