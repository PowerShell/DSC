// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

//! Defines macros used by the module.

/// Panics with a translated message.
macro_rules! panic_t {
    ($($all:tt)*) => {
        panic!("{}", crate::_rust_i18n_t!($($all)*))
    };
}

/// Asserts an expression evaluates to true or panics with a translated message.
macro_rules! assert_t {
    ($expr:expr, $($tail:tt)*) => {
        assert!($expr, "{}", crate::_rust_i18n_t!($($tail)*))
    };
}
