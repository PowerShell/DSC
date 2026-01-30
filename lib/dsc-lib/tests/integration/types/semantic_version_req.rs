// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

#[cfg(test)]
mod methods {
    #[cfg(test)]
    mod parse {
        use dsc_lib::{dscerror::DscError, types::SemanticVersionReq};
        use test_case::test_case;

        #[test_case("1" => matches Ok(_); "major is valid")]
        #[test_case("1.2" => matches Ok(_); "major.minor is valid")]
        #[test_case("1.2.3" => matches Ok(_); "major.minor.patch is valid")]
        #[test_case("1.2.3-pre" => matches Ok(_); "major.minor.patch-pre is valid")]
        #[test_case("1-pre" => matches Err(_); "major-pre is invalid")]
        #[test_case("1.2-pre" => matches Err(_); "major.minor-pre is invalid")]
        #[test_case("1.2.3+build" => matches Err(_); "major.minor.patch+build is invalid")]
        #[test_case("1.2.3-pre+build" => matches Err(_); "major.minor.patch-pre+build is invalid")]
        #[test_case("a" => matches Err(_); "invalid_char is invalid")]
        #[test_case("1.b" => matches Err(_); "major.invalid_char is invalid")]
        #[test_case("1.2.c" => matches Err(_); "major.minor.invalid_char is invalid")]
        fn literal_version(requirement_string: &str) -> Result<SemanticVersionReq, DscError> {
            SemanticVersionReq::parse(requirement_string)
        }

        #[test_case("1.*" => matches Ok(_); "major.wildcard is valid")]
        #[test_case("1.*.*" => matches Ok(_); "major.wildcard.wildcard is valid")]
        #[test_case("1.2.*" => matches Ok(_); "major.minor.wildcard is valid")]
        #[test_case("1.*.3" => matches Err(_); "major.wildcard.patch is invalid")]
        #[test_case("1.2.*-pre" => matches Err(_); "major.minor.wildcard-pre is invalid")]
        #[test_case("1.*.*-pre" => matches Err(_); "major.wildcard.wildcard-pre is invalid")]
        #[test_case("1.2.3-*" => matches Err(_); "major.minor.patch-wildcard is invalid")]
        #[test_case("1.2.3-pre.*" => matches Err(_); "major.minor.patch-pre.wildcard is invalid")]
        fn wildcard_version(requirement_string: &str) -> Result<SemanticVersionReq, DscError> {
            SemanticVersionReq::parse(requirement_string)
        }

        #[test_case("1.2.3" => matches Ok(_); "implicit operator is valid")]
        #[test_case("^ 1.2.3" => matches Ok(_); "caret operator is valid")]
        #[test_case("~ 1.2.3" => matches Ok(_); "tilde operator is valid")]
        #[test_case("= 1.2.3" => matches Ok(_); "exact operator is valid")]
        #[test_case("> 1.2.3" => matches Ok(_); "greater than operator is valid")]
        #[test_case(">= 1.2.3" => matches Ok(_); "greater than or equal to operator is valid")]
        #[test_case("< 1.2.3" => matches Ok(_); "less than operator is valid")]
        #[test_case("<= 1.2.3" => matches Ok(_); "less than or equal to operator is valid")]
        #[test_case("== 1.2.3" => matches Err(_); "invalid operator is invalid")]
        fn operators(requirement_string: &str) -> Result<SemanticVersionReq, DscError> {
            SemanticVersionReq::parse(requirement_string)
        }

        #[test_case("1.2.3, < 1.5" => matches Ok(_); "pair with separating comma is valid")]
        #[test_case("1, 1.2, 1.2.3" => matches Ok(_); "triple with separating comma is valid")]
        #[test_case("<= 1, >= 2" => matches Ok(_); "incompatible pair is valid")]
        #[test_case(", 1, 1.2" => matches Err(_); "leading comma is invalid")]
        #[test_case("1, 1.2," => matches Err(_); "trailing comma is invalid")]
        #[test_case("1 1.2" => matches Err(_); "omitted separating comma is invalid")]
        #[test_case("1.*, < 1.3.*" => matches Ok(_); "multiple comparators with wildcard is valid")]
        fn multiple_comparators(requirement_string: &str) -> Result<SemanticVersionReq, DscError> {
            SemanticVersionReq::parse(requirement_string)
        }

        #[test_case("^1.2" => matches Ok(_); "operator and version without spacing is valid")]
        #[test_case("^   1.2" => matches Ok(_); "operator and version with extra spacing is valid")]
        #[test_case("  ^ 1.2" => matches Ok(_); "leading space is valid")]
        #[test_case("^ 1.2  " => matches Ok(_); "trailing space is valid")]
        #[test_case("^1.2,<1.5" => matches Ok(_); "pair of comparators without spacing is valid")]
        #[test_case("  ^  1.2  ,  <  1.5  " => matches Ok(_); "pair of comparators with extra spacing is valid")]
        fn spacing(requirement_string: &str) -> Result<SemanticVersionReq, DscError> {
            SemanticVersionReq::parse(requirement_string)
        }
    }

    #[cfg(test)]
    mod matches {
        use dsc_lib::types::SemanticVersionReq;
        use test_case::test_case;

        fn check(requirement: &str, versions: Vec<&str>, should_match: bool) {
            let req = SemanticVersionReq::parse(requirement).unwrap();
            let expected = if should_match { "match" } else { "not match" };
            for version in versions {
                pretty_assertions::assert_eq!(
                    req.matches(&version.parse().unwrap()),
                    should_match,
                    "expected version '{version}' to {expected} requirement '{requirement}'"
                );
            }
        }

        #[test_case("1", vec!["1.0.0", "1.2.0", "1.3.0"], true; "matching major")]
        #[test_case("1", vec!["0.1.0", "2.0.0", "1.2.3-rc.1"], false; "not matching major")]
        #[test_case("1.2", vec!["1.2.0", "1.2.3", "1.3.0"], true; "matching major.minor")]
        #[test_case("1.2", vec!["1.0.0", "2.0.0", "1.2.3-rc.1"], false; "not matching major.minor")]
        #[test_case("1.2.3", vec!["1.2.3", "1.2.4", "1.3.0"], true; "matching major.minor.patch")]
        #[test_case("1.2.3", vec!["1.2.0", "2.0.0", "1.2.3-rc.1"], false; "not matching major.minor.patch")]
        #[test_case("1.2.3-rc.2", vec!["1.2.3", "1.3.0", "1.2.3-rc.2", "1.2.3-rc.3"], true; "matching major.minor.patch-pre")]
        #[test_case("1.2.3-rc.2", vec!["1.2.0", "2.0.0", "1.2.3-rc.1", "1.3.0-rc.2"], false; "not matching major.minor.patch-pre")]
        fn implicit(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }

        #[test_case("^1", vec!["1.0.0", "1.2.0", "1.3.0"], true; "matching major")]
        #[test_case("^1", vec!["0.1.0", "2.0.0", "1.2.3-rc.1"], false; "not matching major")]
        #[test_case("^1.*", vec!["1.0.0", "1.2.0", "1.3.0"], true; "matching major.wildcard")]
        #[test_case("^1.*", vec!["0.1.0", "2.0.0", "1.2.3-rc.1"], false; "not matching major.wildcard")]
        #[test_case("^1.*.*", vec!["1.0.0", "1.2.0", "1.3.0"], true; "matching major.wildcard.wildcard")]
        #[test_case("^1.*.*", vec!["0.1.0", "2.0.0", "1.2.3-rc.1"], false; "not matching major.wildcard.wildcard")]
        #[test_case("^1.2", vec!["1.2.0", "1.2.3", "1.3.0"], true; "matching major.minor")]
        #[test_case("^1.2", vec!["1.0.0", "2.0.0", "1.2.3-rc.1"], false; "not matching major.minor")]
        #[test_case("^1.2.*", vec!["1.2.0", "1.2.3", "1.3.0"], true; "matching major.minor.wildcard")]
        #[test_case("^1.2.*", vec!["1.0.0", "2.0.0", "1.2.3-rc.1"], false; "not matching major.minor.wildcard")]
        #[test_case("^1.2.3", vec!["1.2.3", "1.2.4", "1.3.0"], true; "matching major.minor.patch")]
        #[test_case("^1.2.3", vec!["1.2.0", "2.0.0", "1.2.3-rc.1"], false; "not matching major.minor.patch")]
        #[test_case("^1.2.3-rc.2", vec!["1.2.3", "1.3.0", "1.2.3-rc.2", "1.2.3-rc.3"], true; "matching major.minor.patch-pre")]
        #[test_case("^1.2.3-rc.2", vec!["1.2.0", "2.0.0", "1.2.3-rc.1", "1.3.0-rc.2"], false; "not matching major.minor.patch-pre")]
        fn caret(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }

        #[test_case("~1", vec!["1.0.0", "1.2.0", "1.3.0"], true; "matching major")]
        #[test_case("~1", vec!["0.1.0", "2.0.0", "1.2.3-rc.1"], false; "not matching major")]
        #[test_case("~1.*", vec!["1.0.0", "1.2.0", "1.3.0"], true; "matching major.wildcard")]
        #[test_case("~1.*", vec!["0.1.0", "2.0.0", "1.2.3-rc.1"], false; "not matching major.wildcard")]
        #[test_case("~1.*.*", vec!["1.0.0", "1.2.0", "1.3.0"], true; "matching major.wildcard.wildcard")]
        #[test_case("~1.*.*", vec!["0.1.0", "2.0.0", "1.2.3-rc.1"], false; "not matching major.wildcard.wildcard")]
        #[test_case("~1.2", vec!["1.2.0", "1.2.3"], true; "matching major.minor")]
        #[test_case("~1.2", vec!["1.0.0", "1.3.0", "1.2.3-rc.1"], false; "not matching major.minor")]
        #[test_case("~1.2.*", vec!["1.2.0", "1.2.3"], true; "matching major.minor.wildcard")]
        #[test_case("~1.2.*", vec!["1.0.0", "1.3.0", "1.2.3-rc.1"], false; "not matching major.minor.wildcard")]
        #[test_case("~1.2.3", vec!["1.2.3", "1.2.9"], true; "matching major.minor.patch")]
        #[test_case("~1.2.3", vec!["1.2.0", "1.3.0", "1.2.3-rc.1"], false; "not matching major.minor.patch")]
        #[test_case("~1.2.3-rc.2", vec!["1.2.3", "1.2.9", "1.2.3-rc.2", "1.2.3-rc.3"], true; "matching major.minor.patch-pre")]
        #[test_case("~1.2.3-rc.2", vec!["1.2.0", "1.3.0", "1.2.3-rc.1"], false; "not matching major.minor.patch-pre")]
        fn tilde(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }

        #[test_case("<1", vec!["0.1.0"], true; "matching major")]
        #[test_case("<1", vec!["1.0.0", "1.2.3", "0.1.0-rc.1"], false; "not matching major")]
        #[test_case("<1.*", vec!["0.1.0"], true; "matching major.wildcard")]
        #[test_case("<1.*", vec!["1.0.0", "1.2.3", "0.1.0-rc.1"], false; "not matching major.wildcard")]
        #[test_case("<1.*.*", vec!["0.1.0"], true; "matching major.wildcard.wildcard")]
        #[test_case("<1.*.*", vec!["1.0.0", "1.2.3", "0.1.0-rc.1"], false; "not matching major.wildcard.wildcard")]
        #[test_case("<1.2", vec!["0.1.0", "1.0.0", "1.1.1"], true; "matching major.minor")]
        #[test_case("<1.2", vec!["1.2.0", "1.2.3", "1.3.0", "1.2.0-rc.1"], false; "not matching major.minor")]
        #[test_case("<1.2.3", vec!["0.1.0", "1.0.0", "1.2.0"], true; "matching major.minor.patch")]
        #[test_case("<1.2.3", vec!["1.2.3", "1.3.0", "1.2.3-rc.1"], false; "not matching major.minor.patch")]
        #[test_case("<1.2.3-rc.2", vec!["0.1.0", "1.2.0", "1.2.3-rc.1"], true; "matching major.minor.patch-pre")]
        #[test_case("<1.2.3-rc.2", vec!["1.2.3", "1.3.0", "1.0.0-rc.1", "1.2.3-rc.2"], false; "not matching major.minor.patch-pre")]
        fn less_than(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }

        #[test_case("<=1", vec!["0.1.0", "1.0.0", "1.2.3"], true; "matching major")]
        #[test_case("<=1", vec!["2.0.0", "0.1.0-rc.1", "1.0.0-rc.1"], false; "not matching major")]
        #[test_case("<=1.*", vec!["0.1.0", "1.0.0", "1.2.3"], true; "matching major.wildcard")]
        #[test_case("<=1.*", vec!["2.0.0", "0.1.0-rc.1", "1.0.0-rc.1"], false; "not matching major.wildcard")]
        #[test_case("<=1.*.*", vec!["0.1.0", "1.0.0", "1.2.3"], true; "matching major.wildcard.wildcard")]
        #[test_case("<=1.*.*", vec!["2.0.0", "0.1.0-rc.1", "1.0.0-rc.1"], false; "not matching major.wildcard.wildcard")]
        #[test_case("<=1.2", vec!["0.1.0", "1.0.0", "1.2.0", "1.2.3"], true; "matching major.minor")]
        #[test_case("<=1.2", vec!["1.3.0", "1.0.0-rc.1", "1.2.0-rc.1"], false; "not matching major.minor")]
        #[test_case("<=1.2.*", vec!["0.1.0", "1.0.0", "1.2.0", "1.2.3"], true; "matching major.minor.wildcard")]
        #[test_case("<=1.2.*", vec!["1.3.0", "1.0.0-rc.1", "1.2.0-rc.1"], false; "not matching major.minor.wildcard")]
        #[test_case("<=1.2.3", vec!["0.1.0", "1.0.0", "1.2.3"], true; "matching major.minor.patch")]
        #[test_case("<=1.2.3", vec!["1.2.4", "1.3.0", "1.2.0-rc.1", "1.2.3-rc.1"], false; "not matching major.minor.patch")]
        #[test_case("<=1.2.3-rc.2", vec!["0.1.0", "1.2.3-rc.1", "1.2.3-rc.2"], true; "matching major.minor.patch-pre")]
        #[test_case("<=1.2.3-rc.2", vec!["1.2.3", "1.3.0", "1.0.0-rc.1", "1.2.3-rc.3"], false; "not matching major.minor.patch-pre")]
        fn less_than_or_equal_to(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }

        #[test_case("=1", vec!["1.0.0", "1.2.3"], true; "matching major")]
        #[test_case("=1", vec!["0.1.0", "2.0.0", "1.0.0-rc.2"], false; "not matching major")]
        #[test_case("=1.*", vec!["1.0.0", "1.2.3"], true; "matching major.wildcard")]
        #[test_case("=1.*", vec!["0.1.0", "2.0.0", "1.0.0-rc.2"], false; "not matching major.wildcard")]
        #[test_case("=1.*.*", vec!["1.0.0", "1.2.3"], true; "matching major.wildcard.wildcard")]
        #[test_case("=1.*.*", vec!["0.1.0", "2.0.0", "1.0.0-rc.2"], false; "not matching major.wildcard.wildcard")]
        #[test_case("=1.2", vec!["1.2.0", "1.2.3"], true; "matching major.minor")]
        #[test_case("=1.2", vec!["1.0.0", "1.3.0", "1.2.3-rc.2"], false; "not matching major.minor")]
        #[test_case("=1.2.*", vec!["1.2.0", "1.2.3"], true; "matching major.minor.wildcard")]
        #[test_case("=1.2.*", vec!["1.0.0", "1.3.0", "1.2.3-rc.2"], false; "not matching major.minor.wildcard")]
        #[test_case("=1.2.3", vec!["1.2.3"], true; "matching major.minor.patch")]
        #[test_case("=1.2.3", vec!["1.2.0", "1.3.0", "1.2.3-rc.2"], false; "not matching major.minor.patch")]
        #[test_case("=1.2.3-rc.2", vec!["1.2.3-rc.2"], true; "matching major.minor.patch-pre")]
        #[test_case("=1.2.3-rc.2", vec!["1.2.3", "1.3.0", "1.2.3-rc.1"], false; "not matching major.minor.patch-pre")]
        fn equal_to(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }

        #[test_case(">1", vec!["2.0.0", "2.3.4"], true; "matching major")]
        #[test_case(">1", vec!["1.0.0", "1.2.3", "2.0.0-rc.2"], false; "not matching major")]
        #[test_case(">1.*", vec!["2.0.0", "2.3.4"], true; "matching major.wildcard")]
        #[test_case(">1.*", vec!["1.0.0", "1.2.3", "2.0.0-rc.2"], false; "not matching major.wildcard")]
        #[test_case(">1.*.*", vec!["2.0.0", "2.3.4"], true; "matching major.wildcard.wildcard")]
        #[test_case(">1.*.*", vec!["1.0.0", "1.2.3", "2.0.0-rc.2"], false; "not matching major.wildcard.wildcard")]
        #[test_case(">1.2", vec!["1.3.0", "2.0.0"], true; "matching major.minor")]
        #[test_case(">1.2", vec!["1.0.0", "1.2.3", "2.0.0-rc.2"], false; "not matching major.minor")]
        #[test_case(">1.2.*", vec!["1.3.0", "2.0.0"], true; "matching major.minor.wildcard")]
        #[test_case(">1.2.*", vec!["1.0.0", "1.2.3", "2.0.0-rc.2"], false; "not matching major.minor.wildcard")]
        #[test_case(">1.2.3", vec!["1.2.4", "2.0.0"], true; "matching major.minor.patch")]
        #[test_case(">1.2.3", vec!["1.2.3", "2.0.0-rc.2"], false; "not matching major.minor.patch")]
        #[test_case(">1.2.3-rc.2", vec!["1.2.3","2.0.0", "1.2.3-rc.3"], true; "matching major.minor.patch-pre")]
        #[test_case(">1.2.3-rc.2", vec!["1.2.0", "1.2.3-rc.1", "2.0.0-rc.2"], false; "not matching major.minor.patch-pre")]
        fn greater_than(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }

        #[test_case(">=1", vec!["1.0.0", "1.2.3"], true; "matching major")]
        #[test_case(">=1", vec!["0.1.0", "1.2.3-rc.2"], false; "not matching major")]
        #[test_case(">=1.*", vec!["1.0.0", "1.2.3"], true; "matching major.wildcard")]
        #[test_case(">=1.*", vec!["0.1.0", "1.2.3-rc.2"], false; "not matching major.wildcard")]
        #[test_case(">=1.*.*", vec!["1.0.0", "1.2.3"], true; "matching major.wildcard.wildcard")]
        #[test_case(">=1.*.*", vec!["0.1.0", "1.2.3-rc.2"], false; "not matching major.wildcard.wildcard")]
        #[test_case(">=1.2", vec!["1.2.0", "1.2.3"], true; "matching major.minor")]
        #[test_case(">=1.2", vec!["1.1.1", "1.2.3-rc.2"], false; "not matching major.minor")]
        #[test_case(">=1.2.*", vec!["1.2.0", "1.2.3"], true; "matching major.minor.wildcard")]
        #[test_case(">=1.2.*", vec!["1.1.1", "1.2.3-rc.2"], false; "not matching major.minor.wildcard")]
        #[test_case(">=1.2.3", vec!["1.2.3", "1.3.0"], true; "matching major.minor.patch")]
        #[test_case(">=1.2.3", vec!["1.2.2", "1.2.3-rc.2", "2.0.0-rc.2"], false; "not matching major.minor.patch")]
        #[test_case(">=1.2.3-rc.2", vec!["1.2.3", "2.0.0", "1.2.3-rc.2", "1.2.3-rc.3"], true; "matching major.minor.patch-pre")]
        #[test_case(">=1.2.3-rc.2", vec!["1.2.0", "1.2.3-rc.1", "2.0.0-rc.2"], false; "not matching major.minor.patch-pre")]
        fn greater_than_or_equal_to(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }

        #[test_case(">= 2.0.0", vec!["2.0.0", "2.1.0", "3.0.0"], true; "matching major.minor.patch")]
        #[test_case(">= 2.0.0", vec!["1.2.3", "2.0.0-0", "2.0.0-alpha.1", "2.1.0-beta.2", "3.0.0-rc.1"], false; "not matching major.minor.patch")]
        #[test_case(">= 2.0.0-alpha", vec!["2.0.0", "2.1.0", "3.0.0", "2.0.0-alpha","2.0.0-alpha.1", "2.0.0-beta.1"], true; "matching major.minor.patch-alpha")]
        #[test_case(">= 2.0.0-alpha", vec!["1.2.3", "2.0.0-0", "3.0.0-alpha.1"], false; "not matching major.minor.patch-alpha")]
        #[test_case(">= 2.0.0-beta.2", vec!["2.0.0", "2.1.0", "3.0.0", "2.0.0-beta.2", "2.0.0-beta.3", "2.0.0-rc.1"], true; "matching major.minor.patch-beta.2")]
        #[test_case(">= 2.0.0-beta.2", vec!["1.2.3", "2.0.0-alpha", "2.0.0-beta.1", "3.0.0-rc.1"], false; "not matching major.minor.patch-beta.2")]
        #[test_case(">= 2.0.0-0", vec!["2.0.0", "2.1.0", "3.0.0", "2.0.0-0", "2.0.0-1", "2.0.0-alpha", "2.0.0-beta.1"], true; "matching major.minor.patch-0")]
        #[test_case(">= 2.0.0-0", vec!["1.2.3", "3.0.0-rc.1"], false; "not matching major.minor.patch-0")]
        fn prerelease(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }

        #[test_case("1.*", vec!["1.0.0", "1.2.3"], true; "matches major.wildcard")]
        #[test_case("1.*", vec!["0.1.0", "2.0.0", "1.2.3-rc.1"], false; "not matches major.wildcard")]
        #[test_case("1.*.*", vec!["1.0.0", "1.2.3"], true; "matches major.wildcard.wildcard")]
        #[test_case("1.*.*", vec!["0.1.0", "2.0.0", "1.2.3-rc.1"], false; "not matches major.wildcard.wildcard")]
        #[test_case("1.2.*", vec!["1.2.0", "1.2.3"], true; "matches major.minor.wildcard")]
        #[test_case("1.2.*", vec!["1.1.1", "1.3.0", "1.2.3-rc.1"], false; "not matches major.minor.wildcard")]
        fn wildcard(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }

        #[test_case(">=1.2, <1.4.0", vec!["1.2.0", "1.2.3", "1.3.0"], true; "matching multiple compatible requirements")]
        #[test_case(">=1.2, <1.4.0", vec!["1.1.0", "1.4.0", "1.3.0-rc.1"], false; "not matching multiple compatible requirements")]
        #[test_case("<=1.2, >1.4.0", vec!["1.0.0", "1.2.3", "1.3.0", "1.4.0", "2.0.0", "2.3.4-rc.1"], false; "never matching multiple incompatible requirements")]
        fn multiple(requirement: &str, versions: Vec<&str>, should_match: bool) {
            check(requirement, versions, should_match);
        }
    }
}

#[cfg(test)]
mod patterns {
    use dsc_lib::types::SemanticVersionReq;
    use test_case::test_case;

    #[test_case(SemanticVersionReq::PRERELEASE_PATTERN; "PRERELEASE_PATTERN")]
    #[test_case(SemanticVersionReq::WILDCARD_VERSION_PATTERN; "WILDCARD_VERSION_PATTERN")]
    #[test_case(SemanticVersionReq::LITERAL_VERSION_PATTERN; "LITERAL_VERSION_PATTERN")]
    #[test_case(SemanticVersionReq::COMPARATOR_PATTERN; "COMPARATOR_PATTERN")]
    #[test_case(SemanticVersionReq::OPERATOR_PATTERN; "OPERATOR_PATTERN")]
    #[test_case(SemanticVersionReq::WILDCARD_SYMBOL_PATTERN; "WILDCARD_SYMBOL_PATTERN")]
    fn partial_pattern_compiles(pattern: &str) {
        regex::Regex::new(pattern).unwrap();
    }

    #[test_case("1"; "major")]
    #[test_case("1.2"; "major.minor")]
    #[test_case("1.2.3"; "major.minor.patch")]
    #[test_case("1.*"; "major.wildcard_asterisk")]
    #[test_case("1.2.*"; "major.minor.wildcard_asterisk")]
    #[test_case("1.2.3-alpha"; "major.minor.patch-prerelease")]
    #[test_case("^1"; "caret operator")]
    #[test_case("~1"; "tilde operator")]
    #[test_case("=1"; "equals operator")]
    #[test_case(">1"; "greater than operator")]
    #[test_case("<1"; "less than operator")]
    #[test_case(">=1"; "greater than or equal to operator")]
    #[test_case("<=1"; "less than or equal to operator")]
    #[test_case("~1,1.2.3,<2"; "multiple comparators without spacing")]
    #[test_case("~ 1 , 1.2.3 , < 2"; "multiple comparators with extra spacing")]
    fn validating_pattern(requirement: &str) {
        let pattern = SemanticVersionReq::VALIDATING_PATTERN;
        let r = regex::Regex::new(pattern).unwrap();

        if r.is_match(requirement) {
            match semver::VersionReq::parse(requirement) {
                Ok(_) => {}
                Err(e) => panic!("failed to parse '{requirement}': {e}"),
            }
        } else {
            panic!("Expected '{requirement}' to be valid requirement pattern but didn't match regex '{pattern}'")
        }
    }
}

#[cfg(test)]
mod schema {
    use std::sync::LazyLock;

    use dsc_lib::types::SemanticVersionReq;
    use dsc_lib_jsonschema::schema_utility_extensions::SchemaUtilityExtensions;
    use jsonschema::Validator;
    use regex::Regex;
    use schemars::{schema_for, Schema};
    use serde_json::{json, Value};
    use test_case::test_case;

    static SCHEMA: LazyLock<Schema> = LazyLock::new(|| schema_for!(SemanticVersionReq));
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
            .expect(format!("expected keyword '{keyword}' to be defined").as_str());

        assert!(
            !(&*KEYWORD_PATTERN).is_match(value),
            "Expected keyword '{keyword}' to be defined in translation, but was set to i18n key '{value}'"
        );
    }

    #[test_case(&json!("1") => true; "major is valid")]
    #[test_case(&json!("1.2") => true; "major.minor is valid")]
    #[test_case(&json!("1.2.3") => true; "major.minor.patch is valid")]
    #[test_case(&json!("1.2.3-pre") => true; "major.minor.patch-pre is valid")]
    #[test_case(&json!("1.*") => true; "major.wildcard is valid")]
    #[test_case(&json!("1.2.*") => true; "major.minor.wildcard is valid")]
    #[test_case(&json!("^1") => true; "caret operator is valid")]
    #[test_case(&json!("~1") => true; "tilde operator is valid")]
    #[test_case(&json!("=1") => true; "equals operator is valid")]
    #[test_case(&json!(">1") => true; "greater than operator is valid")]
    #[test_case(&json!("<1") => true; "less than operator is valid")]
    #[test_case(&json!(">=1") => true; "greater than or equal to operator is valid")]
    #[test_case(&json!("<=1") => true; "less than or equal to operator is valid")]
    #[test_case(&json!("~1,1.2.3,<2") => true; "multiple comparators without spacing is valid")]
    #[test_case(&json!("~ 1 , 1.2.3 , < 2") => true; "multiple comparators with extra spacing is valid")]
    #[test_case(&json!("1.2.3+build") => false; "major.minor.patch+build is invalid")]
    #[test_case(&json!("1.2.3-pre+build") => false; "major.minor.patch-pre+build is invalid")]
    #[test_case(&json!("!3.0.0") => false; "unknown operator is invalid")]
    #[test_case(&json!("3.0.0.0") => false; "non-semantic version is invalid")]
    #[test_case(&json!("1.a") => false; "version with alphabetic segment is invalid")]
    #[test_case(&json!("*.2") => false; "wildcard.major is invalid")]
    #[test_case(&json!("1.*.3") => false; "major.wildcard.patch is invalid")]
    #[test_case(&json!("1.2.3-*") => false; "major.minor.patch-wildcard is invalid")]
    #[test_case(&json!("1.2.3-pre.*") => false; "major.minor.patch-pre.wildcard is invalid")]
    #[test_case(&json!(">=1.2,") => false; "comma without following comparator is invalid")]
    #[test_case(&json!(">=1.2 < 1.4") => false; "multiple comparators without separating comma is invalid")]
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
    use dsc_lib::types::SemanticVersionReq;
    use serde_json::{json, Value};
    use test_case::test_case;

    #[test_case("^1.2.3"; "single comparator req string serializes to string")]
    #[test_case("^1.2.3, <1.4"; "multi comparator req serializes to string")]
    fn serializing(requirement: &str) {
        let actual = serde_json::to_string(
            &SemanticVersionReq::parse(requirement).expect("parse should never fail"),
        )
        .expect("serialization should never fail");
        let expected = format!(r#""{requirement}""#);

        pretty_assertions::assert_eq!(actual, expected);
    }

    #[test_case(json!("1.2.3") => matches Ok(_); "valid req string value succeeds")]
    #[test_case(json!("a.b") => matches Err(_); "invalid req string value fails")]
    #[test_case(json!(true) => matches Err(_); "boolean value is invalid")]
    #[test_case(json!(1) => matches Err(_); "integer value is invalid")]
    #[test_case(json!(1.2) => matches Err(_); "float value is invalid")]
    #[test_case(json!({"req": "1.2.3"}) => matches Err(_); "object value is invalid")]
    #[test_case(json!(["1.2.3"]) => matches Err(_); "array value is invalid")]
    #[test_case(serde_json::Value::Null => matches Err(_); "null value is invalid")]
    fn deserializing(value: Value) -> Result<SemanticVersionReq, serde_json::Error> {
        serde_json::from_value::<SemanticVersionReq>(value)
    }

    #[test_case("^1", true; "major with explicit operator round trips")]
    #[test_case("1", false; "major with implicit operator does not round trip")]
    #[test_case("^1.2", true; "major.minor with explicit operator round trips")]
    #[test_case("1.2", false; "major.minor with implicit operator does not round trip")]
    #[test_case("^1.2.3", true; "major.minor.patch with explicit operator round trips")]
    #[test_case("1.2.3", false; "major.minor.patch with implicit operator does not round trip")]
    #[test_case("^1.2.3-pre", true; "major.minor.patch-pre with explicit operator round trips")]
    #[test_case("1.2.3-pre", false; "major.minor.patch-pre with implicit operator does not round trip")]
    #[test_case("^1.*", false; "major.wildcard with explicit operator does not round trip")]
    #[test_case("1.*", true; "major.wildcard with implicit operator round trips")]
    #[test_case("^1.*.*", false; "major.wildcard.wildcard with explicit operator does not round trip")]
    #[test_case("1.*.*", false; "major.wildcard.wildcard with implicit operator does not round trip")]
    #[test_case("^1.2.*", false; "major.minor.wildcard version with explicit operator round trips")]
    #[test_case("1.2.*", true; "major.minor.wildcard version with implicit operator round trips")]
    #[test_case("  ^1.2.3", false; "requirement with leading spaces does not round trip")]
    #[test_case("^1.2.3  ", false; "requirement with trailing spaces does not round trip")]
    #[test_case("^1.2.3, <1.5", true; "multi-comparators with single space after comma round trips")]
    #[test_case("^1.2.3,<1.5", false; "multi-comparators without space after comma does not round trip")]
    #[test_case("^1.2.3 , <1.5", false; "multi-comparators with space before and after comma does not round trip")]
    #[test_case("^1.2.3,  <1.5", false; "multi-comparators with multiple spaces after comma does not round trip")]
    fn round_tripping(requirement: &str, should_round_trip: bool) {
        let json_value = json!(requirement);
        // let json_string = json_value.clone().to_string();
        let serialized: SemanticVersionReq = serde_json::from_value(json_value.clone()).unwrap();
        let deserialized = serde_json::to_value(&serialized).unwrap();

        if should_round_trip {
            pretty_assertions::assert_eq!(
                json_value,
                deserialized,
                "expected requirement '{requirement}' to roundtrip without munging"
            );
        } else {
            pretty_assertions::assert_ne!(
                json_value,
                deserialized,
                "expected requirement '{requirement}' serialize as a munged string"
            );
        }
    }
}

#[cfg(test)]
mod traits {
    #[cfg(test)]
    mod default {
        use dsc_lib::types::SemanticVersionReq;

        #[test]
        fn default() {
            pretty_assertions::assert_eq!(
                SemanticVersionReq::default().as_ref(),
                &semver::VersionReq::default(),
            )
        }
    }

    #[cfg(test)]
    mod display {
        use dsc_lib::types::SemanticVersionReq;
        use test_case::test_case;

        #[test_case("1.2", "^1.2"; "valid req with single comparator")]
        #[test_case("1.2, < 1.4", "^1.2, <1.4"; "valid req with multiple comparators")]
        #[test_case("1.*", "1.*"; "valid req with a wildcard")]
        fn format(requirement: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                format!("req: '{}'", SemanticVersionReq::parse(requirement).unwrap()),
                format!("req: '{}'", expected)
            )
        }

        #[test_case("1.2", "^1.2"; "valid req with single comparator")]
        #[test_case("1.2, < 1.4", "^1.2, <1.4"; "valid req with multiple comparators")]
        #[test_case("1.*", "1.*"; "valid req with a wildcard")]
        fn to_string(requirement: &str, expected: &str) {
            pretty_assertions::assert_eq!(
                SemanticVersionReq::parse(requirement).unwrap().to_string(),
                expected.to_string()
            )
        }
    }

    #[cfg(test)]
    mod from {
        use dsc_lib::types::{SemanticVersion, SemanticVersionReq};

        #[test]
        fn semver_version_req() {
            let semver_req = semver::VersionReq::parse("1.2.3").unwrap();
            SemanticVersionReq::from(semver_req).matches(&SemanticVersion::new(1, 2, 3));
        }
    }

    #[cfg(test)]
    mod from_str {
        use dsc_lib::{dscerror::DscError, types::SemanticVersionReq};
        use test_case::test_case;

        // Minimal test suite, since full parsing tests are on the associated `parse` function.
        #[test_case("1.2.3" => matches Ok(_); "valid requirement returns ok")]
        #[test_case("!1.2.3" => matches Err(_); "invalid requirement returns err")]
        fn parse(input: &str) -> Result<SemanticVersionReq, DscError> {
            input.parse()
        }
    }

    #[cfg(test)]
    mod try_from {
        use dsc_lib::{dscerror::DscError, types::SemanticVersionReq};
        use test_case::test_case;

        // Minimal test suite, since full parsing tests are on the associated `parse` function.
        #[test_case("1.2.3" => matches Ok(_); "valid requirement returns ok")]
        #[test_case("!1.2.3" => matches Err(_); "invalid requirement returns err")]
        fn string(input: &str) -> Result<SemanticVersionReq, DscError> {
            SemanticVersionReq::try_from(input.to_string())
        }

        // Minimal test suite, since full parsing tests are on the associated `parse` function.
        #[test_case("1.2.3" => matches Ok(_); "valid requirement returns ok")]
        #[test_case("!1.2.3" => matches Err(_); "invalid requirement returns err")]
        fn string_slice(input: &str) -> Result<SemanticVersionReq, DscError> {
            SemanticVersionReq::try_from(input)
        }
    }

    // While technically we implemented the traits as `From<TypeVersion> for <T>`, it's easier to
    // reason about what we're converting _into_ - otherwise the functions would have names like
    // `type_version_for_semver_version`. When you implement `From`, you automatically implement
    // `Into` for the reversing of the type pair.
    #[cfg(test)]
    mod into {
        use dsc_lib::types::SemanticVersionReq;

        #[test]
        fn semver_version_req() {
            let _: semver::VersionReq = SemanticVersionReq::parse("1.2").unwrap().into();
        }

        #[test]
        fn string() {
            let _: String = SemanticVersionReq::parse("1.2").unwrap().into();
        }
    }

    #[cfg(test)]
    mod partial_eq {
        use dsc_lib::types::SemanticVersionReq;
        use test_case::test_case;

        #[test_case("1.2", "1.2", true; "identical requirements")]
        #[test_case("1.2", "^ 1.2", true; "equivalent requirements")]
        #[test_case("^1.2", "~1.2", false; "differing operator requirements")]
        #[test_case("1.2", "3.4", false; "differing version requirements")]
        #[test_case("1.2", "1.2, <3.4", false; "single and multi version requirements")]
        fn semantic_version_req(lhs: &str, rhs: &str, should_be_equal: bool) {
            if should_be_equal {
                pretty_assertions::assert_eq!(
                    SemanticVersionReq::parse(lhs).unwrap(),
                    SemanticVersionReq::parse(rhs).unwrap()
                )
            } else {
                pretty_assertions::assert_ne!(
                    SemanticVersionReq::parse(lhs).unwrap(),
                    SemanticVersionReq::parse(rhs).unwrap()
                )
            }
        }

        #[test_case("1.2", "1.2", true; "identical requirements")]
        #[test_case("1.2", "^ 1.2", true; "equivalent requirements")]
        #[test_case("^1.2", "~1.2", false; "differing operator requirements")]
        #[test_case("1.2", "3.4", false; "differing version requirements")]
        #[test_case("1.2", "1.2, <3.4", false; "single and multi version requirements")]
        fn semver_version_req(
            semantic_version_req_string: &str,
            semver_version_req_string: &str,
            should_be_equal: bool,
        ) {
            let semantic_version_req =
                SemanticVersionReq::parse(semantic_version_req_string).unwrap();
            let semver_version_req = semver::VersionReq::parse(semver_version_req_string).unwrap();
            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                semantic_version_req == semver_version_req,
                should_be_equal,
                "expected comparison of {semantic_version_req} and {semver_version_req} to be {should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                semver_version_req == semantic_version_req,
                should_be_equal,
                "expected comparison of {semver_version_req} and {semantic_version_req} to be {should_be_equal}"
            );
        }

        #[test_case("1.2", "1.2", true; "identical requirements")]
        #[test_case("1.2", "^ 1.2", true; "equivalent requirements")]
        #[test_case("^1.2", "~1.2", false; "differing operator requirements")]
        #[test_case("1.2", "3.4", false; "differing version requirements")]
        #[test_case("1.2", "1.2, <3.4", false; "single and multi version requirements")]
        #[test_case("1.2", "invalid", false; "requirement and arbitrary string")]
        fn string(semantic_version_req_string: &str, string_slice: &str, should_be_equal: bool) {
            let semantic_version_req =
                SemanticVersionReq::parse(semantic_version_req_string).unwrap();
            let string = string_slice.to_string();
            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                semantic_version_req == string,
                should_be_equal,
                "expected comparison of {semantic_version_req} and {string} to be {should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string == semantic_version_req,
                should_be_equal,
                "expected comparison of {string} and {semantic_version_req} to be {should_be_equal}"
            );
        }

        #[test_case("1.2", "1.2", true; "identical requirements")]
        #[test_case("1.2", "^ 1.2", true; "equivalent requirements")]
        #[test_case("^1.2", "~1.2", false; "differing operator requirements")]
        #[test_case("1.2", "3.4", false; "differing version requirements")]
        #[test_case("1.2", "1.2, <3.4", false; "single and multi version requirements")]
        #[test_case("1.2", "invalid", false; "requirement and arbitrary string")]
        fn str(semantic_version_req_string: &str, string_slice: &str, should_be_equal: bool) {
            let semantic_version_req =
                SemanticVersionReq::parse(semantic_version_req_string).unwrap();
            // Test equivalency bidirectionally
            pretty_assertions::assert_eq!(
                semantic_version_req == string_slice,
                should_be_equal,
                "expected comparison of {semantic_version_req} and {string_slice} to be {should_be_equal}"
            );

            pretty_assertions::assert_eq!(
                string_slice == semantic_version_req,
                should_be_equal,
                "expected comparison of {string_slice} and {semantic_version_req} to be {should_be_equal}"
            );
        }
    }
}
