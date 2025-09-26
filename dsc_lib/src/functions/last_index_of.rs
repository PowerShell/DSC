// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use crate::DscError;
use crate::configure::context::Context;
use crate::functions::{FunctionArgKind, Function, FunctionCategory, FunctionMetadata};
use rust_i18n::t;
use serde_json::Value;
use tracing::debug;

#[derive(Debug, Default)]
pub struct LastIndexOf {}

impl Function for LastIndexOf {
    fn get_metadata(&self) -> FunctionMetadata {
        FunctionMetadata {
            name: "lastIndexOf".to_string(),
            description: t!("functions.lastIndexOf.description").to_string(),
            category: vec![FunctionCategory::String],
            min_args: 2,
            max_args: 2,
            accepted_arg_ordered_types: vec![
                vec![FunctionArgKind::Array],
                vec![FunctionArgKind::String, FunctionArgKind::Number, FunctionArgKind::Array, FunctionArgKind::Object],
            ],
            remaining_arg_accepted_types: None,
            return_types: vec![FunctionArgKind::Number],
        }
    }

    fn invoke(&self, args: &[Value], _context: &Context) -> Result<Value, DscError> {
        debug!("{}", t!("functions.lastIndexOf.invoked"));

        let Some(array) = args[0].as_array() else {
            return Err(DscError::Parser(t!("functions.lastIndexOf.invalidArrayArg").to_string()));
        };

        let item_to_find = &args[1];

        if let Some(pos) = array.iter().rposition(|v| v == item_to_find) {
            let index_i64 = i64::try_from(pos).map_err(|_| {
                DscError::Parser("Array index too large to represent as integer".to_string())
            })?;
            return Ok(Value::Number(index_i64.into()));
        }

        Ok(Value::Number((-1i64).into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::configure::context::Context;
    use crate::parser::Statement;

    #[test]
    fn finds_last_occurrence_string() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[lastIndexOf(createArray('a','b','a','c'), 'a')]", &Context::new()).unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn finds_last_occurrence_number() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[lastIndexOf(createArray(10,20,30,20), 20)]", &Context::new()).unwrap();
        assert_eq!(result, 3);
    }

    #[test]
    fn not_found_returns_minus_one() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute("[lastIndexOf(createArray('x','y'), 'z')]", &Context::new()).unwrap();
        assert_eq!(result, -1);
    }

    #[test]
    fn finds_last_occurrence_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(
            "[lastIndexOf(createArray(createArray('a','b'), createArray('c','d'), createArray('a','b')), createArray('a','b'))]",
            &Context::new(),
        )
        .unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn finds_last_occurrence_nested_array() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute(
                "[lastIndexOf(createArray(createArray(1,2), createArray(3,4), createArray(1,2)), createArray(1,2))]",
                &Context::new(),
            )
            .unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn finds_last_occurrence_object() {
        let mut parser = Statement::new().unwrap();
        let result = parser.parse_and_execute(
            "[lastIndexOf(createArray(createObject('name','John'), createObject('name','Jane'), createObject('name','John')), createObject('name','John'))]",
            &Context::new(),
        )
        .unwrap();
        assert_eq!(result, 2);
    }

    #[test]
    fn finds_object_regardless_of_property_order() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute(
                "[lastIndexOf(createArray(createObject('a',1,'b',2), createObject('b',2,'a',1)), createObject('a',1,'b',2))]",
                &Context::new(),
            )
            .unwrap();
        assert_eq!(result, 1);
    }

    #[test]
    fn mismatched_types_do_not_match() {
        let mut parser = Statement::new().unwrap();
        let result = parser
            .parse_and_execute("[lastIndexOf(createArray('1','2','3'), 1)]", &Context::new())
            .unwrap();
        assert_eq!(result, -1);

        let result = parser
            .parse_and_execute("[lastIndexOf(createArray(1,2,3), '1')]", &Context::new())
            .unwrap();
        assert_eq!(result, -1);
    }
}
