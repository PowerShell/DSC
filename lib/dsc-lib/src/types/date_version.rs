// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use std::{
    fmt::Display,
    str::FromStr,
    sync::OnceLock,
};

use chrono::{Datelike, NaiveDate};
use miette::Diagnostic;
use regex::Regex;
use rust_i18n::t;
use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::schemas::dsc_repo::DscRepoSchema;

/// Defines a version as an ISO8601 formatted date string for compatibility scenarios.
///
/// DSC supports date versions for compatibility. Unless required for compatibility scenarios,
/// resources should use semantic versioning.
///
/// A date version is represented as one of two string forms, both conforming to ISO8601 for
/// dates:
///
/// - `YYYY-MM-DD`, like `2026-02-11` or `2027-11-01`. These date versions are defined without
///   the optional `prerelease` segment.
/// - `YYYY-MM-DD-PRE`, like `2026-02-11-preview` or `2027-11-01-rc`. These date versions are
///   defined with the optional `prerelease` segment.
///
/// A date version _must_ represent a valid date. Defining an invalid month, like `00` or `15`,
/// raises a parsing error. Defining an invalid day of the month, like `00` or `33`, raises a
/// parsing error. For February, `29` is only a valid day for leap years.
///
/// The year for a date version must not be defined with a leading zero. Defining a date with a
/// leading zero for the year, like `0123`, raises a parsing error.
///
/// If the date version is for a prerelease, the prerelease segment must be a string of ASCII
/// alphabetic characters (`[a-zA-Z]`).
#[derive(Debug, Clone, Hash, Serialize, Deserialize, DscRepoSchema)]
#[serde(try_from = "String", into = "String")]
#[dsc_repo_schema(base_name = "dateVersion", folder_path = "definitions")]
pub struct DateVersion(NaiveDate, Option<String>);

/// Indicates an error with parsing or converting a [`DateVersion`].
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum DateVersionError {
    /// Indicates that the input string for a date version doesn't match the validating pattern for
    /// date versions.
    ///
    /// The input string must match the regular expression defined by
    /// [`DateVersion::VALIDATING_PATTERN`] to be parsed as a date version.
    #[error("{t}", t = t!(
        "types.date_version.notMatchPattern",
        "text" => text,
        "pattern" => DateVersion::VALIDATING_PATTERN,
    ))]
    NotMatchPattern {
        /// The text input that failed to match the validating pattern.
        text: String
    },

    /// Indicates that the input string for a date version matches the validating pattern for date
    /// versions but defines an invalid date.
    ///
    /// The validating pattern isn't able to verify whether a given date is valid. For example, the
    /// date string `2026-04-31` isn't valid because the maximum number of days in April is 30.
    /// Similarly, `2028-02-29` defines a leap day in a valid leap year, but the date `2026-02-29`
    /// is invalid because 2026 isn't a leap year.
    #[error("{t}", t = t!(
        "types.date_version.invalidDate",
        "text" => text,
        "errors" => errors.iter().map(|e| e.to_string()).collect::<Vec<String>>().join(", "),
    ))]
    InvalidDate {
        /// The text input that defines an invalid date.
        ///
        /// This text is guaranteed to match the validating pattern for date versions, but it
        /// defines an invalid date for one or more reasons. The `errors` field includes more
        /// details about why the date is invalid.
        text: String,
        /// A list of specific errors that explain why the date defined by `text` is invalid.
        #[related]
        errors: Vec<DateVersionError>,
    },

    /// Indicates that the year segment of a date version defines an invalid year.
    ///
    /// This can occur for years greater than `9999` or less than `1000`.
    #[error("{t}", t = t!(
        "types.date_version.invalidYear",
        "year" => year : {:04},
    ))]
    InvalidYear{
        /// The invalid year defined in the input string.
        year: i32
    },

    /// Indicates that the month segment of a date version defines an invalid month.
    ///
    /// This can occur for months less than `1` or greater than `12`.
    #[error("{t}", t = t!(
        "types.date_version.invalidMonth",
        "month" => month : {:02},
    ))]
    InvalidMonth{
        /// The invalid month defined in the input string.
        month: u32
    },

    /// Indicates that the day segment of a date version defines an invalid leap day for February.
    ///
    /// This can occur when the month is defined as `02`, the day is defined as `29`, but the year
    /// isn't a leap year. For example, `2026-02-29` defines an invalid leap day.
    #[error("{t}", t = t!(
        "types.date_version.invalidLeapDay",
        "year" => year : {:04},
    ))]
    InvalidLeapDay {
        /// The year defined in the input string.
        year: i32
    },

    /// Indicates that the day segment of a date version defines an invalid day for the month.
    ///
    /// This can occur when the day is less than `1` or greater than the maximum number of days in
    /// the month. For example, `2026-04-31` defines an invalid day because April has only 30 days.
    #[error("{t}", t = t!(
        "types.date_version.invalidDay",
        "day" => day : {:02},
        "month" => month : {:02},
        "max_days" => max_days : {:02},
    ))]
    InvalidDay {
        /// The invalid day defined in the input string.
        day: u32,
        /// The month defined in the input string.
        month: u32,
        /// The maximum number of days in the month defined by `month`.
        max_days: u32,
    }
}

/// This static lazily defines the validating regex for [`DateVersion`]. It enables the
/// [`Regex`] instance to be constructed once, the first time it's used, and then reused on all
/// subsequent validation calls. It's kept private, since the API usage is to invoke the
/// [`DateVersion::parse()`] method to validate and parse a string into a date version.
static VALIDATING_PATTERN_REGEX: OnceLock<Regex> = OnceLock::new();

impl DateVersion {
    /// Parses string input into a [`DateVersion`], raising an error if the input isn't a valid
    /// string representation.
    ///
    /// A date version is represented as one of two string forms, both conforming to ISO8601 for
    /// dates:
    ///
    /// - `YYYY-MM-DD`, like `2026-02-11` or `2027-11-01`. These date versions are defined without
    ///   the optional `prerelease` segment.
    /// - `YYYY-MM-DD-PRE`, like `2026-02-11-preview` or `2027-11-01-rc`. These date versions are
    ///   defined with the optional `prerelease` segment.
    ///
    /// A date version _must_ represent a valid date. Defining an invalid month, like `00` or `15`,
    /// raises a parsing error. Defining an invalid day of the month, like `00` or `33`, raises a
    /// parsing error. For February, `29` is only a valid day for leap years.
    ///
    /// The year for a date version must not be defined with a leading zero. Defining a date with
    /// a leading zero for the year, like `0123`, raises a parsing error.
    ///
    /// If the date version is for a prerelease, the prerelease segment must be a string of ASCII
    /// alphabetic characters (`[a-zA-Z]`).
    ///
    /// A date version is parsed from an input string with the following regular expression:
    ///
    /// ```regex
    /// ^(?<year>[1-9][0-9]{3})-(?<month>0[1-9]|1[0-2])-(?<day>0[1-9]|[1-2][0-9]|3[0-1])(?:-(?<prerelease>[a-zA-Z]+))?$
    /// ```
    ///
    /// For more information about the pattern, see [`VALIDATING_PATTERN`].
    ///
    /// # Examples
    ///
    /// The following example shows how you can parse a date version from a string.
    ///
    /// ```rust
    /// # use dsc_lib::types::DateVersion;
    /// # use chrono::Datelike;
    /// # use pretty_assertions::assert_eq;
    /// let version = DateVersion::parse("2026-02-03").unwrap();
    ///
    /// assert_eq!(version.year(), 2026);
    /// assert_eq!(version.month(), 2);
    /// assert_eq!(version.day(), 3);
    /// assert_eq!(version.prerelease(), None);
    /// ```
    ///
    /// # Errors
    ///
    /// The following list shows example inputs that raise parse errors and includes the reasons
    /// each input fails to parse:
    ///
    /// - `2026-00-01` - The month segment must not be defined as `00`.
    /// - `2026-15-01` - The month segment is defined as `15`, which isn't a valid month.
    /// - `0123-11-01` - The year segment must not begin with a leading zero.
    /// - `2026-05-00` - The day segment must not be defined as `00`.
    /// - `2026-03-38` - The day segment must not be defined as a number greater than `31`.
    /// - `2026-07-15-rc.1` - The prerelease segment must be a string consisting of only ASCII
    ///   alphabetic characters.
    /// - `2026-02-29` - Defines the date as February 29, 2026. 2026 isn't a leap year and
    ///   February 29 isn't a valid date for a non-leap year.
    ///
    /// [`VALIDATING_PATTERN`]: DateVersion::VALIDATING_PATTERN
    pub fn parse(text: &str) -> Result<Self, DateVersionError> {
        let pattern = VALIDATING_PATTERN_REGEX.get_or_init(Self::init_pattern);
        let Some(captures) = pattern.captures(text) else {
            return Err(DateVersionError::NotMatchPattern { text: text.to_string() });
        };

        let year: i32 = captures
            .name("year")
            .expect("year is always defined")
            .as_str()
            .parse()
            .expect("year is always a valid non-zero i32");

        let month: u32 = captures
            .name("month")
            .expect("month is always defined")
            .as_str()
            .parse()
            .expect("month is always a valid non-zero u32");

        let day: u32 = captures
            .name("day")
            .expect("day is always defined")
            .as_str()
            .parse()
            .expect("day is always a valid non-zero u32");

        Self::validate_date(year, month, day, text)?;
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .expect("date is pre-validated");

        match captures.name("prerelease") {
            None => Ok(Self(date, None)),
            Some(prerelease_match) => Ok(Self(
                date,
                Some(prerelease_match.as_str().to_string())
            )),
        }
    }

    /// Returns the underlying [`NaiveDate`] representation for an instance of [`DateVersion`].
    ///
    /// # Examples
    ///
    /// The following snippet shows how a [`DateVersion`] without the optional `prerelease` segment
    /// returns the underlying naive date.
    ///
    /// ```rust
    /// # use chrono::{Datelike, NaiveDate};
    /// # use dsc_lib::types::DateVersion;
    /// # use pretty_assertions::assert_eq;
    /// let version: DateVersion = "2026-02-01".parse().unwrap();
    /// let expected: NaiveDate = "2026-02-01".parse().unwrap();
    /// assert_eq!(version.as_naive_date(), expected);
    /// ```
    ///
    /// The next snippet shows how the same [`DateVersion`] compares to another instance that
    /// does define the `prerelease` segment.
    ///
    /// ```rust
    /// # use chrono::{Datelike, NaiveDate};
    /// # use dsc_lib::types::DateVersion;
    /// # use pretty_assertions::{assert_eq, assert_ne};
    /// let stable_version: DateVersion = "2026-02-01".parse().unwrap();
    /// let preview_version: DateVersion = "2026-02-01-preview".parse().unwrap();
    /// // Comparing the versions as naive dates shows they are apparently equal
    /// assert_eq!(
    ///     stable_version.as_naive_date(),
    ///     preview_version.as_naive_date()
    /// );
    /// // Comparing the versions as date versions shows they are _not_ equal
    /// assert_ne!(
    ///     stable_version,
    ///     preview_version
    /// );
    /// ```
    pub fn as_naive_date(&self) -> NaiveDate {
        self.0
    }

    /// Indicates whether the originally parsed date version defined the `prerelease` segment.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dsc_lib::types::DateVersion;
    /// let stable_version: DateVersion = "2026-02-01".parse().unwrap();
    /// assert_eq!(
    ///     stable_version.is_prerelease(),
    ///     false
    /// );
    ///
    /// let preview_version: DateVersion = "2026-02-01-preview".parse().unwrap();
    /// assert_eq!(
    ///     preview_version.is_prerelease(),
    ///     true
    /// );
    /// ```
    pub fn is_prerelease(&self) -> bool {
        self.1.is_some()
    }

    /// Returns a reference to the prerelease segment string for the date version, if defined.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use dsc_lib::types::DateVersion;
    /// let preview_version: DateVersion = "2026-02-01-preview".parse().unwrap();
    /// assert_eq!(
    ///     preview_version.prerelease(),
    ///     Some(&"preview".to_string())
    /// );
    ///
    /// let stable_version: DateVersion = "2026-02-01".parse().unwrap();
    /// assert_eq!(
    ///     stable_version.prerelease(),
    ///     None
    /// );
    /// ```
    pub fn prerelease(&self) -> Option<&String> {
        self.1.as_ref()
    }

    /// Defines the pattern that validates the string representation of a [`DateVersion`].
    ///
    /// This pattern is used both to validate the string representation of a date version and to
    /// capture segments of the string. This pattern:
    ///
    /// 1. Forbids leading and trailing spacing characters.
    /// 1. Requires the first segment of the version to be a four-digit year, like `2026`. It
    ///    forbids the year from starting with a leading zero.
    /// 1. Requires a hyphen before the second segment. The second segment must be a two-digit
    ///    month, like `02` or `11`. The first nine months require a leading zero. Defining the
    ///    month as either zero or greater than twelve is invalid.
    /// 1. Requires a hyphen before the third segment. The third segment must be a two-digit day of
    ///    the month, like `07` or `29`. The first nine days of the month require a leading zero.
    ///    Defining the day of the month as either zero or greater than thirty-one is invalid.
    /// 1. Allows an optional fourth segment. If defined, the fourth segment must be preceded by
    ///    a hyphen. The fourth segment must define a prerelease string as one or more ASCII
    ///    alphabetic characters (`[a-zA-Z]`), like `rc` or `preview`.
    ///
    /// The full pattern is:
    ///
    /// ```regex
    /// ^(?<year>[1-9][0-9]{3})-(?<month>0[1-9]|1[0-2])-(?<day>0[1-9]|[1-2][0-9]|3[0-1])(?:-(?<prerelease>[a-zA-Z]+))?$
    /// ```
    ///
    /// The following list shows a set of input strings and whether they're valid for this pattern.
    /// If the input string is invalid, the list item explains why that input is invalid.
    ///
    /// - `2026-07-15` - valid input, defining the year as `2026`, the month as July, and the day
    ///   of the month as `15`.
    /// - `2026-07-15-preview` - valid input, defining the year as `2026`, the month as July, the
    ///   day of the month as `15`, and the prerelease segment as `preview`.
    /// - `2026-00-01` - invalid input. The month segment must not be defined as `00`.
    /// - `2026-15-01` - invalid input. The month segment is defined as `15`, which isn't a valid
    ///   month.
    /// - `0123-11-01` - invalid input. The year segment must not begin with a leading zero.
    /// - `2026-05-00` - invalid input. The day segment must not be defined as `00`.
    /// - `2026-03-38` - invalid input. The day segment must not be defined as a number greater
    ///   than `31`.
    /// - `2026-07-15-rc.1` - invalid input. The prerelease segment must be a string consisting of
    ///   only ASCII alphabetic characters.
    /// - `2026-02-29` - valid pattern, invalid input. While this construction is valid for the
    ///   regular expression, defining this value for a [`DateVersion`] fails parsing. It defines
    ///   the date as February 29, 2026. 2026 isn't a leap year and February 29 isn't a valid date
    ///   for a non-leap year.
    pub const VALIDATING_PATTERN: &str = const_str::concat!(
        "^",                  // Anchor to start of string
        "(?<year>",           // Open named capture group for year segment
            "[1-9]",          //   First numeral in year must be greater than 0
            "[0-9]{3}",       //   Remaining numerals in year can be any digit
        ")",                  // Close named capture group
        "-",                  // Require a hyphen before the month segment
        "(?<month>",          // Open named capture group for month segment
            "0[1-9]",         //   The first nine months must have a leading 0
            "|",              //   or
            "1[0-2]",         //   The last three months must be 10, 11, or 12
        ")",                  // Close named capture group for month segment
        "-",                  // Require a hyphen before the day segment
        "(?<day>",            // Open named capture group for the day segment
            "0[1-9]",         //   The first 9 days of a month must have a leading 0
            "|",              //   or
            "[1-2][0-9]",     //   The day is between 10 and 29
            "|",              //   or
            "3[0-1]",         //   The day is 30 or 31
        ")",                  // Close named capture group for the day segment
        r"(?:",               // Open non-capture group for optional prerelease substring
            "-",              //   Require a hyphen before the segment
            "(?<prerelease>", //   Open named capture group for the prerelease segment
            "[a-zA-Z]+",      //   Require the segment to contain only alphabetic ASCII
            ")",              //   Close named capture group for the prerelease segment
        ")?",                 // Close non-capture group for optional prerelease substring
        "$",                  // Anchor to end of string
    );

    /// Returns the [`Regex`] for [`VALIDATING_PATTERN`].
    ///
    /// This private method is used to initialize the [`VALIDATING_PATTERN_REGEX`] private
    /// static to reduce the number of times the regular expression is compiled from the pattern
    /// string.
    ///
    /// [`VALIDATING_PATTERN`]: Self::VALIDATING_PATTERN
    fn init_pattern() -> Regex {
        Regex::new(Self::VALIDATING_PATTERN).expect("pattern is valid")
    }

    /// Validates the numerically parsed values for the string representation of a [`DateVersion`].
    ///
    /// The [`VALIDATING_PATTERN`] alone isn't able to verify whether a given date is valid. For
    /// example, the date string `2026-04-31` isn't valid because the maximum number of days in
    /// April is `30`. Similarly, `2028-02-29` defines a leap day in a valid leap year, but the
    /// date `2026-02-29` is invalid because 2026 isn't a leap year.
    ///
    /// This validation function enables the type to validate the input for a date version beyond
    /// what a regular expression can quickly and performantly parse while capturing the various
    /// segments.
    ///
    /// [`VALIDATING_PATTERN`]: Self::VALIDATING_PATTERN
    fn validate_date(year: i32, month: u32, day: u32, text: &str) -> Result<(), DateVersionError> {
        let max_days_in_month = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31u32,
            4 | 6 | 9 | 11 => 30u32,
            2 => {
                //  Gregorian leap year rule: divisible by 4, except centuries not divisible by 400.
                if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
                    29u32
                } else {
                    28u32
                }
            },
            _ => unreachable!()
        };
        let mut errors: Vec<DateVersionError> = vec![];

        if year > 9999 || year < 1000 {
            errors.push(DateVersionError::InvalidYear { year });
        }

        if month > 12 {
            errors.push(DateVersionError::InvalidMonth { month });
        }

        if day > max_days_in_month {
            if month == 2 && day == 29 {
                errors.push(DateVersionError::InvalidLeapDay { year });
            } else {
                errors.push(DateVersionError::InvalidDay {
                    day,
                    month,
                    max_days: max_days_in_month
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(DateVersionError::InvalidDate { text: text.to_string(), errors })
        }
    }
}

impl JsonSchema for DateVersion {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        Self::default_schema_id_uri().into()
    }
    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": Self::default_schema_id_uri(),
            "title": t!("schemas.definitions.dateVersion.title"),
            "description": t!("schemas.definitions.dateVersion.description"),
            "markdownDescription": t!("schemas.definitions.dateVersion.markdownDescription"),
            "type": "string",
            "pattern": Self::VALIDATING_PATTERN,
            "patternErrorMessage": t!("schemas.definitions.dateVersion.patternErrorMessage"),
        })
    }
}

impl Datelike for DateVersion {
    fn day(&self) -> u32 {
        self.as_ref().day()
    }
    fn day0(&self) -> u32 {
        self.as_ref().day0()
    }
    fn year(&self) -> i32 {
        self.as_ref().year()
    }
    fn month(&self) -> u32 {
        self.as_ref().month()
    }
    fn month0(&self) -> u32 {
        self.as_ref().month0()
    }
    fn ordinal(&self) -> u32 {
        self.as_ref().ordinal()
    }
    fn ordinal0(&self) -> u32 {
        self.as_ref().ordinal0()
    }
    fn weekday(&self) -> chrono::Weekday {
        self.as_ref().weekday()
    }
    fn iso_week(&self) -> chrono::IsoWeek {
        self.as_ref().iso_week()
    }
    fn with_year(&self, year: i32) -> Option<Self> {
        match self.as_ref().with_year(year) {
            None => None,
            Some(new_date) => Some(Self(new_date, self.1.clone()))
        }
    }
    fn with_month(&self, month: u32) -> Option<Self> {
        match self.as_ref().with_month(month) {
            None => None,
            Some(new_date) => Some(Self(new_date, self.1.clone()))
        }
    }
    fn with_month0(&self, month0: u32) -> Option<Self> {
        match self.as_ref().with_month0(month0) {
            None => None,
            Some(new_date) => Some(Self(new_date, self.1.clone()))
        }
    }
    fn with_day(&self, day: u32) -> Option<Self> {
        match self.as_ref().with_day(day) {
            None => None,
            Some(new_date) => Some(Self(new_date, self.1.clone()))
        }
    }
    fn with_day0(&self, day0: u32) -> Option<Self> {
        match self.as_ref().with_day0(day0) {
            None => None,
            Some(new_date) => Some(Self(new_date, self.1.clone()))
        }
    }
    fn with_ordinal(&self, ordinal: u32) -> Option<Self> {
        match self.as_ref().with_ordinal(ordinal) {
            None => None,
            Some(new_date) => Some(Self(new_date, self.1.clone()))
        }
    }
    fn with_ordinal0(&self, ordinal0: u32) -> Option<Self> {
        match self.as_ref().with_ordinal0(ordinal0) {
            None => None,
            Some(new_date) => Some(Self(new_date, self.1.clone()))
        }
    }
}

impl Display for DateVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(prerelease) = &self.1 {
            write!(f, "{}-{}", self.0.format("%Y-%m-%d"), prerelease)
        } else {
            write!(f, "{}", self.0.format("%Y-%m-%d"))
        }
    }
}

// Reference a date version as a naive date
impl AsRef<NaiveDate> for DateVersion {
    fn as_ref(&self) -> &NaiveDate {
        &self.0
    }
}

impl FromStr for DateVersion {
    type Err = DateVersionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<&str> for DateVersion {
    type Error = DateVersionError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl TryFrom<String> for DateVersion {
    type Error = DateVersionError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value.as_str())
    }
}

impl From<DateVersion> for String {
    fn from(value: DateVersion) -> Self {
        value.to_string()
    }
}

impl TryFrom<NaiveDate> for DateVersion {
    type Error = DateVersionError;
    fn try_from(value: NaiveDate) -> Result<Self, Self::Error> {
        Self::validate_date(value.year(), value.month(), value.day(), value.to_string().as_str())?;

        Ok(Self(value, None))
    }
}

impl From<DateVersion> for NaiveDate {
    fn from(value: DateVersion) -> Self {
        value.0
    }
}

impl Eq for DateVersion {}

impl PartialEq for DateVersion {
    fn eq(&self, other: &Self) -> bool {
        let self_is_prerelease = self.is_prerelease();
        let other_is_prerelease = other.is_prerelease();
        let both_are_prerelease = self_is_prerelease && other_is_prerelease;
        let neither_are_prerelease = !self_is_prerelease && !other_is_prerelease;

        if both_are_prerelease {
            self.0.eq(&other.0) && self.1.eq(&other.1)
        } else if neither_are_prerelease {
            self.0.eq(&other.0)
        } else {
            false
        }
    }
}

impl PartialEq<NaiveDate> for DateVersion {
    fn eq(&self, other: &NaiveDate) -> bool {
        self.0.eq(other)
    }
}

impl PartialEq<DateVersion> for NaiveDate {
    fn eq(&self, other: &DateVersion) -> bool {
        self.eq(&other.0)
    }
}

impl PartialEq<String> for DateVersion {
    fn eq(&self, other: &String) -> bool {
        match Self::parse(other.as_str()) {
            Ok(other_version) => self.eq(&other_version),
            Err(_) => false,
        }
    }
}

impl PartialEq<DateVersion> for String {
    fn eq(&self, other: &DateVersion) -> bool {
        match DateVersion::parse(self.as_str()) {
            Ok(version) => version.eq(other),
            Err(_) => false
        }
    }
}

impl PartialEq<str> for DateVersion {
    fn eq(&self, other: &str) -> bool {
        match Self::parse(other) {
            Ok(other_version) => self.eq(&other_version),
            Err(_) => false,
        }
    }
}

impl PartialEq<DateVersion> for str {
    fn eq(&self, other: &DateVersion) -> bool {
        match DateVersion::parse(self) {
            Ok(version) => version.eq(other),
            Err(_) => false
        }
    }
}

impl PartialEq<&str> for DateVersion {
    fn eq(&self, other: &&str) -> bool {
        match Self::parse(*other) {
            Ok(other_version) => self.eq(&other_version),
            Err(_) => false,
        }
    }
}

impl PartialEq<DateVersion> for &str {
    fn eq(&self, other: &DateVersion) -> bool {
        match DateVersion::parse(*self) {
            Ok(version) => version.eq(other),
            Err(_) => false
        }
    }
}

impl Ord for DateVersion {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_is_prerelease = self.is_prerelease();
        let other_is_prerelease = other.is_prerelease();
        let both_are_prerelease = self_is_prerelease && other_is_prerelease;

        if self.0 == other.0 {
            if both_are_prerelease {
                self.1.as_ref().cmp(&other.1.as_ref())
            } else if self_is_prerelease {
                std::cmp::Ordering::Less
            } else if other_is_prerelease {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        } else {
            self.0.cmp(&other.0)
        }
    }
}

impl PartialOrd for DateVersion {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialOrd<NaiveDate> for DateVersion {
    fn partial_cmp(&self, other: &NaiveDate) -> Option<std::cmp::Ordering> {
        match Self::try_from(*other) {
            Ok(_) => self.0.partial_cmp(other),
            Err(_) => None,
        }
    }
}

impl PartialOrd<DateVersion> for NaiveDate {
    fn partial_cmp(&self, other: &DateVersion) -> Option<std::cmp::Ordering> {
        match DateVersion::try_from(*self) {
            Ok(_) => self.partial_cmp(&other.0),
            Err(_) => None,
        }
    }
}

impl PartialOrd<String> for DateVersion {
    fn partial_cmp(&self, other: &String) -> Option<std::cmp::Ordering> {
        match Self::parse(other.as_str()) {
            Ok(other_version) => self.partial_cmp(&other_version),
            Err(_) => None,
        }
    }
}

impl PartialOrd<DateVersion> for String {
    fn partial_cmp(&self, other: &DateVersion) -> Option<std::cmp::Ordering> {
        match DateVersion::parse(self.as_str()) {
            Ok(version) => version.partial_cmp(other),
            Err(_) => None,
        }
    }
}

impl PartialOrd<str> for DateVersion {
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        match Self::parse(other) {
            Ok(other_version) => self.partial_cmp(&other_version),
            Err(_) => None,
        }
    }
}

impl PartialOrd<DateVersion> for str {
    fn partial_cmp(&self, other: &DateVersion) -> Option<std::cmp::Ordering> {
        match DateVersion::parse(self) {
            Ok(version) => version.partial_cmp(other),
            Err(_) => None,
        }
    }
}
