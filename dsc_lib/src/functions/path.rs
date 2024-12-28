// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{AcceptedArgKind, Function};
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Default)]
pub struct Path {}

/// Implements the `path` function.
/// Accepts a variable number of arguments, each of which is a string.
/// Returns a string that is the concatenation of the arguments, separated by the platform's path separator.
impl Function for Path {
    fn min_args(&self) -> usize {
        2
    }

    fn max_args(&self) -> usize {
        usize::MAX
    }

    fn accepted_arg_types(&self) -> Vec<AcceptedArgKind> {
        vec![AcceptedArgKind::String]
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("Executing path function with args: {:?}", args);

        let mut path = PathBuf::new();
        for arg in args {
            if let Value::String(s) = arg {
                path.push(s);
            } else {
                return Err(DscError::Parser("Arguments must all be strings".to_string()));
            }
        }

        Ok(Value::String(path.to_string_lossy().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    const SEPARATOR: char = std::path::MAIN_SEPARATOR;

    #[test]
    fn start_with_drive_letter() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[path('C:\\','test')]", &Context::new()).unwrap();

        #[cfg(target_os = "windows")]
        assert_eq!(result, format!("C:{SEPARATOR}test"));

        #[cfg(not(target_os = "windows"))]
        assert_eq!(result, format!("C:\\{SEPARATOR}test"));
    }

    #[test]
    fn drive_letter_in_middle() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[path('a','C:\\','test')]", &Context::new()).unwrap();

        // if any part of the path is absolute, it replaces it instead of appending on Windows
        #[cfg(target_os = "windows")]
        assert_eq!(result, format!("C:{SEPARATOR}test"));

        // non-Windows, the colon is a valid character in a path
        #[cfg(not(target_os = "windows"))]
        assert_eq!(result, format!("a{SEPARATOR}C:\\{SEPARATOR}test"));
    }

    #[test]
    fn multiple_drive_letters() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[path('C:\\','D:\\','test')]", &Context::new()).unwrap();

        // if any part of the path is absolute, it replaces it instead of appending on Windows
        #[cfg(target_os = "windows")]
        assert_eq!(result, format!("D:\\test"));

        // non-Windows, the colon is a valid character in a path
        #[cfg(not(target_os = "windows"))]
        assert_eq!(result, format!("C:\\{SEPARATOR}D:\\{SEPARATOR}test"));
    }

    #[test]
    fn relative_path() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[path('a','..','b')]", &Context::new()).unwrap();
        assert_eq!(result, format!("a{SEPARATOR}..{SEPARATOR}b"));
    }

    #[test]
    fn path_segement_with_separator() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(format!("[path('a','b{SEPARATOR}c')]").as_str(), &Context::new()).unwrap();
        assert_eq!(result, format!("a{SEPARATOR}b{SEPARATOR}c"));
    }

    #[test]
    fn unix_absolute_path() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[path('/','a','b')]", &Context::new()).unwrap();
        assert_eq!(result, format!("/a{SEPARATOR}b"));
    }

    #[test]
    fn two_args() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[path('a','b')]", &Context::new()).unwrap();
        assert_eq!(result, format!("a{SEPARATOR}b"));
    }

    #[test]
    fn three_args() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[path('a','b','c')]", &Context::new()).unwrap();
        assert_eq!(result, format!("a{SEPARATOR}b{SEPARATOR}c"));
    }
}
