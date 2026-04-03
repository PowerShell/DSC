// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    use chrono::NaiveDate;
    use dsc_lib::types::{DateVersion, DateVersionError};
    use test_case::test_case;

    #[test_case("2026-02-28" => matches Ok(_); "ISO8601 date is valid")]
    #[test_case("2026-02-28-rc" => matches Ok(_); "ISO8601 date with prerelease is valid")]
    #[test_case("2028-02-29" => matches Ok(_); "leap day in leap year is valid")]
    #[test_case("2026-02-29" => matches Err(_); "leap day in non leap year is invalid")]
    #[test_case("123-02-03" => matches Err(_); "year with less than four digits is invalid")]
    #[test_case("1234-2-03" => matches Err(_); "month with less than two digits is invalid")]
    #[test_case("1234-02-3" => matches Err(_); "day with less than two digits is invalid")]
    #[test_case("0123-02-03" => matches Err(_); "year with leading zero is invalid")]
    #[test_case("1234-00-03" => matches Err(_); "zero month is invalid")]
    #[test_case("1234-02-00" => matches Err(_); "zero day is invalid")]
    #[test_case("1234-14-21" => matches Err(_); "impossible month is invalid")]
    #[test_case("1234-01-32" => matches Err(_); "impossible january date is invalid")]
    #[test_case("1234-02-30" => matches Err(_); "impossible february date is invalid")]
    #[test_case("1234-03-32" => matches Err(_); "impossible march date is invalid")]
    #[test_case("1234-04-31" => matches Err(_); "impossible april date is invalid")]
    #[test_case("1234-05-32" => matches Err(_); "impossible may date is invalid")]
    #[test_case("1234-06-31" => matches Err(_); "impossible june date is invalid")]
    #[test_case("1234-07-32" => matches Err(_); "impossible july date is invalid")]
    #[test_case("1234-08-32" => matches Err(_); "impossible august date is invalid")]
    #[test_case("1234-09-31" => matches Err(_); "impossible september date is invalid")]
    #[test_case("1234-10-32" => matches Err(_); "impossible october date is invalid")]
    #[test_case("1234-11-31" => matches Err(_); "impossible november date is invalid")]
    #[test_case("1234-12-32" => matches Err(_); "impossible december date is invalid")]
    #[test_case("2026-02-28-rc.1" => matches Err(_); "prerelease with non-ASCII alphabetic characters is invalid")]
    fn parse(text: &str) -> Result<DateVersion, DateVersionError> {
        DateVersion::parse(text)
    }

    #[test_case("2026-02-03", "2026-02-03"; "version without prerelease returns same naive date")]
    #[test_case("2026-02-03-rc", "2026-02-03"; "version with prerelease returns same naive date")]
    fn as_naive_date(date_version_text: &str, expected_naive_date_text: &str) {
        let version: DateVersion = date_version_text.parse().unwrap();
        let expected: NaiveDate = expected_naive_date_text.parse().unwrap();

        pretty_assertions::assert_eq!(version.as_naive_date(), expected);
    }

    #[test_case("2026-02-03" => false; "version without prerelease segment")]
    #[test_case("2026-02-03-rc" => true; "version with prerelease segment")]
    fn is_prerelease(text: &str) -> bool {
        DateVersion::parse(text).unwrap().is_prerelease()
    }

    #[test_case("2026-02-03", None; "version without prerelease segment")]
    #[test_case("2026-02-03-rc", Some(&String::from("rc")); "version with prerelease segment")]
    fn prerelease_segment(text: &str, expected: Option<&String>) {
        pretty_assertions::assert_eq!(
            DateVersion::parse(text).unwrap().prerelease(),
            expected
        );
    }
}

#[cfg(test)]
mod schema {
    use std::sync::LazyLock;

    use dsc_lib::types::DateVersion;
    use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    use jsonschema::Validator;
    use regex::Regex;
    use schemars::{schema_for, Schema};
    use serde_json::{json, Value};
    use test_case::test_case;

    static SCHEMA: LazyLock<Schema> = LazyLock::new(|| schema_for!(DateVersion));
    static VALIDATOR: LazyLock<Validator> =
        LazyLock::new(|| Validator::new((&*SCHEMA).as_value()).unwrap());
    static KEYWORD_PATTERN: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^\w+(\.\w+)+$").expect("pattern is valid"));

    #[test_case("title")]
    #[test_case("description")]
    #[test_case("markdownDescription")]
    #[test_case("patternErrorMessage")]
    fn has_documentation_keyword(keyword: &str) {
        let schema = &*SCHEMA;
        let value = schema
            .get_keyword_as_str(keyword)
            .expect(&format!("expected keyword '{keyword}' to be defined"));

        assert!(
            !(&*KEYWORD_PATTERN).is_match(value),
            "Expected keyword '{keyword}' to be defined in translation, but was set to i18n key '{value}'"
        );
    }

    #[test_case(&json!("2026-02-28") => matches true; "string value as ISO8601 date is valid")]
    #[test_case(&json!("2026-02-28-rc") => matches true; "string value as ISO8601 date with prerelease is valid")]
    #[test_case(&json!("2028-02-29") => matches true; "string value with leap day in leap year is valid")]
    #[test_case(&json!("2026-02-29") => matches true; "string value with leap day in non leap year is apparently valid")]
    #[test_case(&json!("123-02-03") => matches false; "string value with year with less than four digits is invalid")]
    #[test_case(&json!("1234-2-03") => matches false; "string value with month with less than two digits is invalid")]
    #[test_case(&json!("1234-02-3") => matches false; "string value with day with less than two digits is invalid")]
    #[test_case(&json!("0123-02-03") => matches false; "string value with year with leading zero is invalid")]
    #[test_case(&json!("1234-2-03") => matches false; "string value with month without leading zero is invalid")]
    #[test_case(&json!("1234-02-3") => matches false; "string value with day without leading zero is invalid")]
    #[test_case(&json!("1234-00-03") => matches false; "string value with zero month is invalid")]
    #[test_case(&json!("1234-02-00") => matches false; "string value with zero day is invalid")]
    #[test_case(&json!("1234-14-21") => matches false; "string value with impossible month is invalid")]
    #[test_case(&json!("1234-01-32") => matches false; "string value with impossible january date is invalid")]
    #[test_case(&json!("1234-02-30") => matches true; "string value with impossible february date is apparently valid")]
    #[test_case(&json!("1234-03-32") => matches false; "string value with impossible march date is invalid")]
    #[test_case(&json!("1234-04-31") => matches true; "string value with impossible april date is apparently valid")]
    #[test_case(&json!("1234-05-32") => matches false; "string value with impossible may date is invalid")]
    #[test_case(&json!("1234-06-31") => matches true; "string value with impossible june date is apparently valid")]
    #[test_case(&json!("1234-07-32") => matches false; "string value with impossible july date is invalid")]
    #[test_case(&json!("1234-08-32") => matches false; "string value with impossible august date is invalid")]
    #[test_case(&json!("1234-09-31") => matches true; "string value with impossible september date is apparently valid")]
    #[test_case(&json!("1234-10-32") => matches false; "string value with impossible october date is invalid")]
    #[test_case(&json!("1234-11-31") => matches true; "string value with impossible november date is apparently valid")]
    #[test_case(&json!("1234-12-32") => matches false; "string value with impossible december date is invalid")]
    #[test_case(&json!("2026-02-28-rc.1") => matches false; "prerelease with non-ASCII alphabetic characters is invalid")]
    #[test_case(&json!(true) => false; "boolean value is invalid")]
    #[test_case(&json!(1) => false; "integer value is invalid")]
    #[test_case(&json!(1.2) => false; "float value is invalid")]
    #[test_case(&json!({"req": "1.2.3"}) => false; "object value is invalid")]
    #[test_case(&json!(["1.2.3"]) => false; "array value is invalid")]
    #[test_case(&serde_json::Value::Null => false; "null value is invalid")]
    fn validation(input_json: &Value) -> bool {
        (&*VALIDATOR).validate(input_json).is_ok()
    }
}

#[cfg(test)]
mod serde {
    use dsc_lib::types::DateVersion;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case("2026-02-03", json!("2026-02-03"))]
    #[test_case("2026-02-03-rc", json!("2026-02-03-rc"))]
    fn serializing(version: &str, expected: Value) {
        let actual = serde_json::to_value(
            &DateVersion::parse(version).expect("parse should never fail"),
        )
        .expect("serialization should never fail");

        pretty_assertions::assert_eq!(actual, expected);
    }

    #[test_case(json!("2026-02-28") => matches Ok(_); "string value as ISO8601 date is valid")]
    #[test_case(json!("2026-02-28-rc") => matches Ok(_); "string value as ISO8601 date with prerelease is valid")]
    #[test_case(json!("2028-02-29") => matches Ok(_); "string value with leap day in leap year is valid")]
    #[test_case(json!("2026-02-29") => matches Err(_); "string value with leap day in non leap year is invalid")]
    #[test_case(json!("123-02-03") => matches Err(_); "string value with year with less than four digits is invalid")]
    #[test_case(json!("1234-2-03") => matches Err(_); "string value with month with less than two digits is invalid")]
    #[test_case(json!("1234-02-3") => matches Err(_); "string value with day with less than two digits is invalid")]
    #[test_case(json!("0123-02-03") => matches Err(_); "string value with year with leading zero is invalid")]
    #[test_case(json!("1234-2-03") => matches Err(_); "string value with month without leading zero is invalid")]
    #[test_case(json!("1234-02-3") => matches Err(_); "string value with day without leading zero is invalid")]
    #[test_case(json!("1234-00-03") => matches Err(_); "string value with zero month is invalid")]
    #[test_case(json!("1234-02-00") => matches Err(_); "string value with zero day is invalid")]
    #[test_case(json!("1234-14-21") => matches Err(_); "string value with impossible month is invalid")]
    #[test_case(json!("1234-01-32") => matches Err(_); "string value with impossible january date is invalid")]
    #[test_case(json!("1234-02-30") => matches Err(_); "string value with impossible february date is invalid")]
    #[test_case(json!("1234-03-32") => matches Err(_); "string value with impossible march date is invalid")]
    #[test_case(json!("1234-04-31") => matches Err(_); "string value with impossible april date is invalid")]
    #[test_case(json!("1234-05-32") => matches Err(_); "string value with impossible may date is invalid")]
    #[test_case(json!("1234-06-31") => matches Err(_); "string value with impossible june date is invalid")]
    #[test_case(json!("1234-07-32") => matches Err(_); "string value with impossible july date is invalid")]
    #[test_case(json!("1234-08-32") => matches Err(_); "string value with impossible august date is invalid")]
    #[test_case(json!("1234-09-31") => matches Err(_); "string value with impossible september date is invalid")]
    #[test_case(json!("1234-10-32") => matches Err(_); "string value with impossible october date is invalid")]
    #[test_case(json!("1234-11-31") => matches Err(_); "string value with impossible november date is invalid")]
    #[test_case(json!("1234-12-32") => matches Err(_); "string value with impossible december date is invalid")]
    #[test_case(json!("2026-02-28-rc.1") => matches Err(_); "prerelease with non-ASCII alphabetic characters is invalid")]
    #[test_case(json!(true) => matches Err(_); "boolean value is invalid")]
    #[test_case(json!(1) => matches Err(_); "integer value is invalid")]
    #[test_case(json!(1.2) => matches Err(_); "float value is invalid")]
    #[test_case(json!({"req": "1.2.3"}) => matches Err(_); "object value is invalid")]
    #[test_case(json!(["1.2.3"]) => matches Err(_); "array value is invalid")]
    #[test_case(serde_json::Value::Null => matches Err(_); "null value is invalid")]
    fn deserializing(value: Value) -> Result<DateVersion, serde_json::Error> {
        serde_json::from_value::<DateVersion>(value)
    }
}

#[cfg(test)]
mod traits {
    #[cfg(test)]
    mod date_like {
        use chrono::{Datelike, NaiveDate};
        use dsc_lib::types::DateVersion;
        use test_case::test_case;

        #[test_case("2026-02-03" => 3; "date version without prerelease")]
        #[test_case("2026-02-03-rc" => 3; "date version with prerelease")]
        fn day(version: &str) -> u32 {
            DateVersion::parse(version).unwrap().day()
        }

        #[test_case("2026-02-03" => 2; "date version without prerelease")]
        #[test_case("2026-02-03-rc" => 2; "date version with prerelease")]
        fn day0(version: &str) -> u32 {
            DateVersion::parse(version).unwrap().day0()
        }

        #[test_case("2026-02-03" => 2026; "date version without prerelease")]
        #[test_case("2026-02-03-rc" => 2026; "date version with prerelease")]
        fn year(version: &str) -> i32 {
            DateVersion::parse(version).unwrap().year()
        }

        #[test_case("2026-02-03" => 2; "date version without prerelease")]
        #[test_case("2026-02-03-rc" => 2; "date version with prerelease")]
        fn month(version: &str) -> u32 {
            DateVersion::parse(version).unwrap().month()
        }

        #[test_case("2026-02-03" => 1; "date version without prerelease")]
        #[test_case("2026-02-03-rc" => 1; "date version with prerelease")]
        fn month0(version: &str) -> u32 {
            DateVersion::parse(version).unwrap().month0()
        }

        #[test_case("2026-02-03" => 34; "date version without prerelease")]
        #[test_case("2026-02-03-rc" => 34; "date version with prerelease")]
        fn ordinal(version: &str) -> u32 {
            DateVersion::parse(version).unwrap().ordinal()
        }

        #[test_case("2026-02-03" => 33; "date version without prerelease")]
        #[test_case("2026-02-03-rc" => 33; "date version with prerelease")]
        fn ordinal0(version: &str) -> u32 {
            DateVersion::parse(version).unwrap().ordinal0()
        }

        #[test_case("2026-02-03" => chrono::Weekday::Tue; "date version without prerelease")]
        #[test_case("2026-02-03-rc" => chrono::Weekday::Tue; "date version with prerelease")]
        fn weekday(version: &str) -> chrono::Weekday {
            DateVersion::parse(version).unwrap().weekday()
        }

        #[test_case("2026-02-03" => NaiveDate::from_ymd_opt(2026, 2, 3).unwrap().iso_week(); "date version without prerelease")]
        #[test_case("2026-02-03-rc" => NaiveDate::from_ymd_opt(2026, 2, 3).unwrap().iso_week(); "date version with prerelease")]
        fn iso_week(version: &str) -> chrono::IsoWeek {
            DateVersion::parse(version).unwrap().iso_week()
        }

        #[test_case("2026-02-03" => Some(DateVersion::parse("2028-02-03").unwrap()); "date version without prerelease")]
        #[test_case("2026-02-03-rc" => Some(DateVersion::parse("2028-02-03-rc").unwrap()); "date version with prerelease")]
        fn with_year(version: &str) -> Option<DateVersion> {
            DateVersion::parse(version).unwrap().with_year(2028)
        }

        #[test_case("2026-02-03" => Some(DateVersion::parse("2026-01-03").unwrap()); "date version without prerelease")]
        #[test_case("2026-02-03-rc" => Some(DateVersion::parse("2026-01-03-rc").unwrap()); "date version with prerelease")]
        fn with_month(version: &str) -> Option<DateVersion> {
            DateVersion::parse(version).unwrap().with_month(1)
        }

        #[test_case("2026-02-03" => Some(DateVersion::parse("2026-04-03").unwrap()); "date version without prerelease")]
        #[test_case("2026-02-03-rc" => Some(DateVersion::parse("2026-04-03-rc").unwrap()); "date version with prerelease")]
        fn with_month0(version: &str) -> Option<DateVersion> {
            DateVersion::parse(version).unwrap().with_month0(3)
        }

        #[test_case("2026-02-03" => Some(DateVersion::parse("2026-02-01").unwrap()); "date version without prerelease")]
        #[test_case("2026-02-03-rc" => Some(DateVersion::parse("2026-02-01-rc").unwrap()); "date version with prerelease")]
        fn with_day(version: &str) -> Option<DateVersion> {
            DateVersion::parse(version).unwrap().with_day(1)
        }

        #[test_case("2026-02-03" => Some(DateVersion::parse("2026-02-04").unwrap()); "date version without prerelease")]
        #[test_case("2026-02-03-rc" => Some(DateVersion::parse("2026-02-04-rc").unwrap()); "date version with prerelease")]
        fn with_day0(version: &str) -> Option<DateVersion> {
            DateVersion::parse(version).unwrap().with_day0(3)
        }

        #[test_case("2026-02-03" => Some(DateVersion::parse("2026-02-11").unwrap()); "date version without prerelease")]
        #[test_case("2026-02-03-rc" => Some(DateVersion::parse("2026-02-11-rc").unwrap()); "date version with prerelease")]
        fn with_ordinal(version: &str) -> Option<DateVersion> {
            DateVersion::parse(version).unwrap().with_ordinal(42)
        }

        #[test_case("2026-02-03" => Some(DateVersion::parse("2026-02-14").unwrap()); "date version without prerelease")]
        #[test_case("2026-02-03-rc" => Some(DateVersion::parse("2026-02-14-rc").unwrap()); "date version with prerelease")]
        fn with_ordinal0(version: &str) -> Option<DateVersion> {
            DateVersion::parse(version).unwrap().with_ordinal0(44)
        }
    }

    #[cfg(test)]
    mod display {
        use dsc_lib::types::DateVersion;
        use test_case::test_case;

        #[test_case("2026-02-03", "2026-02-03"; "date version without prerelease")]
        #[test_case("2026-02-03-rc", "2026-02-03-rc"; "date version with prerelease")]
        fn format(version: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                format!("version: '{}'", DateVersion::parse(version).unwrap()),
                format!("version: '{}'", expected)
            )
        }

        #[test_case("2026-02-03", "2026-02-03"; "date version without prerelease")]
        #[test_case("2026-02-03-rc", "2026-02-03-rc"; "date version with prerelease")]
        fn to_string(version: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                DateVersion::parse(version).unwrap().to_string(),
                expected.to_string()
            )
        }
    }

    #[cfg(test)]
    mod from_str {
        use std::str::FromStr;

        use dsc_lib::types::{DateVersion, DateVersionError};
        use test_case::test_case;

        #[test_case("2026-02-28" => matches Ok(_); "ISO8601 date is valid")]
        #[test_case("2026-02-28-rc" => matches Ok(_); "ISO8601 date with prerelease is valid")]
        #[test_case("2028-02-29" => matches Ok(_); "leap day in leap year is valid")]
        #[test_case("2026-02-29" => matches Err(_); "leap day in non leap year is invalid")]
        #[test_case("123-02-03" => matches Err(_); "year with less than four digits is invalid")]
        #[test_case("1234-2-03" => matches Err(_); "month with less than two digits is invalid")]
        #[test_case("1234-02-3" => matches Err(_); "day with less than two digits is invalid")]
        #[test_case("0123-02-03" => matches Err(_); "year with leading zero is invalid")]
        #[test_case("1234-2-03" => matches Err(_); "month without leading zero is invalid")]
        #[test_case("1234-02-3" => matches Err(_); "day without leading zero is invalid")]
        #[test_case("1234-00-03" => matches Err(_); "zero month is invalid")]
        #[test_case("1234-02-00" => matches Err(_); "zero day is invalid")]
        #[test_case("1234-14-21" => matches Err(_); "impossible month is invalid")]
        #[test_case("1234-01-32" => matches Err(_); "impossible january date is invalid")]
        #[test_case("1234-02-30" => matches Err(_); "impossible february date is invalid")]
        #[test_case("1234-03-32" => matches Err(_); "impossible march date is invalid")]
        #[test_case("1234-04-31" => matches Err(_); "impossible april date is invalid")]
        #[test_case("1234-05-32" => matches Err(_); "impossible may date is invalid")]
        #[test_case("1234-06-31" => matches Err(_); "impossible june date is invalid")]
        #[test_case("1234-07-32" => matches Err(_); "impossible july date is invalid")]
        #[test_case("1234-08-32" => matches Err(_); "impossible august date is invalid")]
        #[test_case("1234-09-31" => matches Err(_); "impossible september date is invalid")]
        #[test_case("1234-10-32" => matches Err(_); "impossible october date is invalid")]
        #[test_case("1234-11-31" => matches Err(_); "impossible november date is invalid")]
        #[test_case("1234-12-32" => matches Err(_); "impossible december date is invalid")]
        #[test_case("2026-02-28-rc.1" => matches Err(_); "prerelease with non-ASCII alphabetic characters is invalid")]
        fn from_str(text: &str) -> Result<DateVersion, DateVersionError> {
            DateVersion::from_str(text)
        }

        #[test_case("2026-02-28" => matches Ok(_); "ISO8601 date is valid")]
        #[test_case("2026-02-28-rc" => matches Ok(_); "ISO8601 date with prerelease is valid")]
        #[test_case("2028-02-29" => matches Ok(_); "leap day in leap year is valid")]
        #[test_case("2026-02-29" => matches Err(_); "leap day in non leap year is invalid")]
        #[test_case("123-02-03" => matches Err(_); "year with less than four digits is invalid")]
        #[test_case("1234-2-03" => matches Err(_); "month with less than two digits is invalid")]
        #[test_case("1234-02-3" => matches Err(_); "day with less than two digits is invalid")]
        #[test_case("0123-02-03" => matches Err(_); "year with leading zero is invalid")]
        #[test_case("1234-2-03" => matches Err(_); "month without leading zero is invalid")]
        #[test_case("1234-02-3" => matches Err(_); "day without leading zero is invalid")]
        #[test_case("1234-00-03" => matches Err(_); "zero month is invalid")]
        #[test_case("1234-02-00" => matches Err(_); "zero day is invalid")]
        #[test_case("1234-14-21" => matches Err(_); "impossible month is invalid")]
        #[test_case("1234-01-32" => matches Err(_); "impossible january date is invalid")]
        #[test_case("1234-02-30" => matches Err(_); "impossible february date is invalid")]
        #[test_case("1234-03-32" => matches Err(_); "impossible march date is invalid")]
        #[test_case("1234-04-31" => matches Err(_); "impossible april date is invalid")]
        #[test_case("1234-05-32" => matches Err(_); "impossible may date is invalid")]
        #[test_case("1234-06-31" => matches Err(_); "impossible june date is invalid")]
        #[test_case("1234-07-32" => matches Err(_); "impossible july date is invalid")]
        #[test_case("1234-08-32" => matches Err(_); "impossible august date is invalid")]
        #[test_case("1234-09-31" => matches Err(_); "impossible september date is invalid")]
        #[test_case("1234-10-32" => matches Err(_); "impossible october date is invalid")]
        #[test_case("1234-11-31" => matches Err(_); "impossible november date is invalid")]
        #[test_case("1234-12-32" => matches Err(_); "impossible december date is invalid")]
        #[test_case("2026-02-28-rc.1" => matches Err(_); "prerelease with non-ASCII alphabetic characters is invalid")]
        fn parse(text: &str) -> Result<DateVersion, DateVersionError> {
            text.parse()
        }
    }

    #[cfg(test)]
    mod try_from {
        use chrono::NaiveDate;
        use dsc_lib::types::{DateVersion, DateVersionError};
        use test_case::test_case;

        #[test_case("2026-02-03".parse().unwrap() => matches Ok(_); "date with year greater than 999 is valid")]
        #[test_case("0999-02-03".parse().unwrap() => matches Err(_); "date with year less than 1000 is invalid")]
        fn naive_date(date: NaiveDate) -> Result<DateVersion, DateVersionError> {
            DateVersion::try_from(date)
        }

        #[test_case("2026-02-28" => matches Ok(_); "ISO8601 date is valid")]
        #[test_case("2026-02-28-rc" => matches Ok(_); "ISO8601 date with prerelease is valid")]
        #[test_case("2028-02-29" => matches Ok(_); "leap day in leap year is valid")]
        #[test_case("2026-02-29" => matches Err(_); "leap day in non leap year is invalid")]
        #[test_case("123-02-03" => matches Err(_); "year with less than four digits is invalid")]
        #[test_case("1234-2-03" => matches Err(_); "month with less than two digits is invalid")]
        #[test_case("1234-02-3" => matches Err(_); "day with less than two digits is invalid")]
        #[test_case("0123-02-03" => matches Err(_); "year with leading zero is invalid")]
        #[test_case("1234-2-03" => matches Err(_); "month without leading zero is invalid")]
        #[test_case("1234-02-3" => matches Err(_); "day without leading zero is invalid")]
        #[test_case("1234-00-03" => matches Err(_); "zero month is invalid")]
        #[test_case("1234-02-00" => matches Err(_); "zero day is invalid")]
        #[test_case("1234-14-21" => matches Err(_); "impossible month is invalid")]
        #[test_case("1234-01-32" => matches Err(_); "impossible january date is invalid")]
        #[test_case("1234-02-30" => matches Err(_); "impossible february date is invalid")]
        #[test_case("1234-03-32" => matches Err(_); "impossible march date is invalid")]
        #[test_case("1234-04-31" => matches Err(_); "impossible april date is invalid")]
        #[test_case("1234-05-32" => matches Err(_); "impossible may date is invalid")]
        #[test_case("1234-06-31" => matches Err(_); "impossible june date is invalid")]
        #[test_case("1234-07-32" => matches Err(_); "impossible july date is invalid")]
        #[test_case("1234-08-32" => matches Err(_); "impossible august date is invalid")]
        #[test_case("1234-09-31" => matches Err(_); "impossible september date is invalid")]
        #[test_case("1234-10-32" => matches Err(_); "impossible october date is invalid")]
        #[test_case("1234-11-31" => matches Err(_); "impossible november date is invalid")]
        #[test_case("1234-12-32" => matches Err(_); "impossible december date is invalid")]
        #[test_case("2026-02-28-rc.1" => matches Err(_); "prerelease with non-ASCII alphabetic characters is invalid")]
        fn string(text: &str) -> Result<DateVersion, DateVersionError> {
            DateVersion::try_from(text.to_string())
        }

        #[test_case("2026-02-28" => matches Ok(_); "ISO8601 date is valid")]
        #[test_case("2026-02-28-rc" => matches Ok(_); "ISO8601 date with prerelease is valid")]
        #[test_case("2028-02-29" => matches Ok(_); "leap day in leap year is valid")]
        #[test_case("2026-02-29" => matches Err(_); "leap day in non leap year is invalid")]
        #[test_case("123-02-03" => matches Err(_); "year with less than four digits is invalid")]
        #[test_case("1234-2-03" => matches Err(_); "month with less than two digits is invalid")]
        #[test_case("1234-02-3" => matches Err(_); "day with less than two digits is invalid")]
        #[test_case("0123-02-03" => matches Err(_); "year with leading zero is invalid")]
        #[test_case("1234-2-03" => matches Err(_); "month without leading zero is invalid")]
        #[test_case("1234-02-3" => matches Err(_); "day without leading zero is invalid")]
        #[test_case("1234-00-03" => matches Err(_); "zero month is invalid")]
        #[test_case("1234-02-00" => matches Err(_); "zero day is invalid")]
        #[test_case("1234-14-21" => matches Err(_); "impossible month is invalid")]
        #[test_case("1234-01-32" => matches Err(_); "impossible january date is invalid")]
        #[test_case("1234-02-30" => matches Err(_); "impossible february date is invalid")]
        #[test_case("1234-03-32" => matches Err(_); "impossible march date is invalid")]
        #[test_case("1234-04-31" => matches Err(_); "impossible april date is invalid")]
        #[test_case("1234-05-32" => matches Err(_); "impossible may date is invalid")]
        #[test_case("1234-06-31" => matches Err(_); "impossible june date is invalid")]
        #[test_case("1234-07-32" => matches Err(_); "impossible july date is invalid")]
        #[test_case("1234-08-32" => matches Err(_); "impossible august date is invalid")]
        #[test_case("1234-09-31" => matches Err(_); "impossible september date is invalid")]
        #[test_case("1234-10-32" => matches Err(_); "impossible october date is invalid")]
        #[test_case("1234-11-31" => matches Err(_); "impossible november date is invalid")]
        #[test_case("1234-12-32" => matches Err(_); "impossible december date is invalid")]
        #[test_case("2026-02-28-rc.1" => matches Err(_); "prerelease with non-ASCII alphabetic characters is invalid")]
        fn str(text: &str) -> Result<DateVersion, DateVersionError> {
            DateVersion::try_from(text)
        }
    }

    #[cfg(test)]
    mod into {
        use chrono::NaiveDate;
        use dsc_lib::types::DateVersion;
        use test_case::test_case;

        #[test_case("2026-02-03", "2026-02-03".parse().unwrap(); "date version without prerelease")]
        #[test_case("2026-02-03-rc", "2026-02-03".parse().unwrap(); "date version with prerelease")]
        fn naive_date(version: &str, expected: NaiveDate) {
            let actual: NaiveDate = DateVersion::parse(version).unwrap().into();

            pretty_assertions::assert_eq!(
                actual,
                expected
            )
        }

        #[test_case("2026-02-03", "2026-02-03".to_string(); "date version without prerelease")]
        #[test_case("2026-02-03-rc", "2026-02-03-rc".to_string(); "date version with prerelease")]
        fn string(version: &str, expected: String) {
            let actual: String = DateVersion::parse(version).unwrap().into();

            pretty_assertions::assert_eq!(
                actual,
                expected
            )
        }
    }

    #[cfg(test)]
    mod partial_eq {
        use dsc_lib::types::DateVersion;
        use test_case::test_case;

        #[test_case("2026-02-03", "2026-02-03", true; "identical date versions without prerelease")]
        #[test_case("2026-02-03-rc", "2026-02-03-rc", true; "identical date versions with prerelease")]
        #[test_case("2026-02-03", "2026-02-13", false; "date versions with different days")]
        #[test_case("2026-02-03", "2026-12-03", false; "date versions with different months")]
        #[test_case("2026-02-03", "2027-02-03", false; "date versions with different years")]
        #[test_case("2026-02-03-alpha", "2026-02-03-beta", false; "date versions with different prerelease segments")]
        fn date_version(lhs: &str, rhs: &str, should_be_equal: bool) {
            if should_be_equal {
                pretty_assertions::assert_eq!(
                    DateVersion::parse(lhs).unwrap(),
                    DateVersion::parse(rhs).unwrap()
                )
            } else {
                pretty_assertions::assert_ne!(
                    DateVersion::parse(lhs).unwrap(),
                    DateVersion::parse(rhs).unwrap()
                )
            }
        }

        #[test_case("2026-02-03", "2026-02-03", true; "date version without prerelease and identical string")]
        #[test_case("2026-02-03-rc", "2026-02-03-rc", true; "date version with prerelease and identical string")]
        #[test_case("2026-02-03", "2026-02-13", false; "date version and string with different days")]
        #[test_case("2026-02-03", "2026-12-03", false; "date version and string with different months")]
        #[test_case("2026-02-03", "2027-02-03", false; "date version and string with different years")]
        #[test_case("2026-02-03-alpha", "2026-02-03-beta", false; "date version and string with different prerelease segments")]
        #[test_case("2026-02-03", "2026.02.03", false; "date version and non-parseable string")]
        fn string(date_version_string: &str, string_slice: &str, should_be_equal: bool) {
            let date_version = DateVersion::parse(date_version_string).unwrap();
            let string = string_slice.to_string();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                date_version == string,
                should_be_equal,
                "expected comparison of {date_version} and {string} to be {should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string == date_version,
                should_be_equal,
                "expected comparison of {string} and {date_version} to be {should_be_equal}"
            );
        }

        #[test_case("2026-02-03", "2026-02-03", true; "date version without prerelease and identical string slice")]
        #[test_case("2026-02-03-rc", "2026-02-03-rc", true; "date version with prerelease and identical string slice")]
        #[test_case("2026-02-03", "2026-02-13", false; "date version and string slice with different days")]
        #[test_case("2026-02-03", "2026-12-03", false; "date version and string slice with different months")]
        #[test_case("2026-02-03", "2027-02-03", false; "date version and string slice with different years")]
        #[test_case("2026-02-03-alpha", "2026-02-03-beta", false; "date version and string slice with different prerelease segments")]
        #[test_case("2026-02-03", "2026.02.03", false; "date version and non-parseable string slice")]
        fn str(date_version_string: &str, string_slice: &str, should_be_equal: bool) {
            let date_version = DateVersion::parse(date_version_string).unwrap();

            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                date_version == string_slice,
                should_be_equal,
                "expected comparison of {date_version} and {string_slice} to be {should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string_slice == date_version,
                should_be_equal,
                "expected comparison of {string_slice} and {date_version} to be {should_be_equal}"
            );
        }
    }

    #[cfg(test)]
    mod ord {
        use std::cmp::Ordering;

        use dsc_lib::types::DateVersion;
        use test_case::test_case;

        #[test_case("2026-02-03", "2026-02-03", Ordering::Equal; "equal stable versions")]
        #[test_case("2026-02-03-rc", "2026-02-03-rc", Ordering::Equal; "equal prerelease versions")]
        #[test_case("2026-02-04", "2026-02-03", Ordering::Greater; "newer stable is greater than older stable")]
        #[test_case("2026-02-03", "2026-02-04", Ordering::Less; "older stable is less than newer stable")]
        #[test_case("2026-02-03", "2026-02-03-rc", Ordering::Greater; "stable is greater than prerelease")]
        #[test_case("2026-02-03-alpha", "2026-02-03-beta", Ordering::Less; "prerelease sorts lexicographically")]
        #[test_case("2026-02-03-PREVIEW", "2026-02-03-preview", Ordering::Less; "prerelease uppercase is less than lowercase")]
        fn date_version(lhs: &str, rhs: &str, expected_order: Ordering) {
            pretty_assertions::assert_eq!(
                DateVersion::parse(lhs)
                    .expect("parsing for lhs should not fail")
                    .partial_cmp(&DateVersion::parse(rhs).expect("parsing for rhs should not fail"))
                    .expect("comparison should always be an ordering"),
                expected_order,
                "expected '{lhs}' compared to '{rhs}' to be {expected_order:#?}"
            )
        }
    }

    #[cfg(test)]
    mod partial_ord {
        use std::cmp::Ordering;

        use chrono::NaiveDate;
        use dsc_lib::types::DateVersion;
        use test_case::test_case;

        #[test_case("2026-02-03", "2026-02-03", Some(Ordering::Equal); "equal stable version and date")]
        #[test_case("2026-02-03-rc", "2026-02-03", Some(Ordering::Equal); "prerelease versions and same date")]
        #[test_case("2026-02-04", "2026-02-03", Some(Ordering::Greater); "newer version is greater than older date")]
        #[test_case("2026-02-03", "2026-02-04", Some(Ordering::Less); "older version is less than newer date")]
        #[test_case("2026-02-03", "0999-02-03", None; "version and invalid date are not comparable")]
        fn naive_date(version_string: &str, date_string: &str, expected_order: Option<Ordering>) {
            let version: DateVersion = version_string.parse().unwrap();
            let date: NaiveDate = date_string.parse().unwrap();

            // Test comparison bidirectionally
            pretty_assertions::assert_eq!(
                version.partial_cmp(&date),
                expected_order,
                "expected comparison of {version} and {date} to be #{expected_order:#?}"
            );

            let expected_inverted_order = match expected_order {
                None => None,
                Some(o) => match o {
                    Ordering::Equal => Some(Ordering::Equal),
                    Ordering::Greater => Some(Ordering::Less),
                    Ordering::Less => Some(Ordering::Greater),
                },
            };

            pretty_assertions::assert_eq!(
                date.partial_cmp(&version),
                expected_inverted_order,
                "expected comparison of {date} and {version} to be #{expected_inverted_order:#?}"
            );
        }

        #[test_case("2026-02-03", "2026-02-03", Some(Ordering::Equal); "equal stable version and date")]
        #[test_case("2026-02-03-rc", "2026-02-03-rc", Some(Ordering::Equal); "prerelease version and same string")]
        #[test_case("2026-02-04", "2026-02-03", Some(Ordering::Greater); "newer version is greater than older string")]
        #[test_case("2026-02-03", "2026-02-04", Some(Ordering::Less); "older version is less than newer string")]
        #[test_case("2026-02-03", "0999-02-03", None; "version and invalid string are not comparable")]
        fn string(
            version_string: &str,
            string_slice: &str,
            expected_order: Option<Ordering>,
        ) {
            let version: DateVersion = version_string.parse().unwrap();
            let string = string_slice.to_string();

            // Test comparison bidirectionally
            pretty_assertions::assert_eq!(
                version.partial_cmp(&string),
                expected_order,
                "expected comparison of {version} and {string} to be #{expected_order:#?}"
            );

            let expected_inverted_order = match expected_order {
                None => None,
                Some(o) => match o {
                    Ordering::Equal => Some(Ordering::Equal),
                    Ordering::Greater => Some(Ordering::Less),
                    Ordering::Less => Some(Ordering::Greater),
                },
            };

            pretty_assertions::assert_eq!(
                string.partial_cmp(&version),
                expected_inverted_order,
                "expected comparison of {string} and {version} to be #{expected_inverted_order:#?}"
            );
        }

        #[test_case("2026-02-03", "2026-02-03", Some(Ordering::Equal); "equal stable version and date")]
        #[test_case("2026-02-03-rc", "2026-02-03-rc", Some(Ordering::Equal); "prerelease version and same string")]
        #[test_case("2026-02-04", "2026-02-03", Some(Ordering::Greater); "newer version is greater than older string")]
        #[test_case("2026-02-03", "2026-02-04", Some(Ordering::Less); "older version is less than newer string")]
        #[test_case("2026-02-03", "0999-02-03", None; "version and invalid string are not comparable")]
        fn str(
            version_string: &str,
            string_slice: &str,
            expected_order: Option<Ordering>,
        ) {
            let version: DateVersion = version_string.parse().unwrap();

            // Test comparison bidirectionally
            pretty_assertions::assert_eq!(
                version.partial_cmp(string_slice),
                expected_order,
                "expected comparison of {version} and {string_slice} to be #{expected_order:#?}"
            );

            let expected_inverted_order = match expected_order {
                None => None,
                Some(o) => match o {
                    Ordering::Equal => Some(Ordering::Equal),
                    Ordering::Greater => Some(Ordering::Less),
                    Ordering::Less => Some(Ordering::Greater),
                },
            };

            pretty_assertions::assert_eq!(
                string_slice.partial_cmp(&version),
                expected_inverted_order,
                "expected comparison of {string_slice} and {version} to be #{expected_inverted_order:#?}"
            );
        }
    }
}
