// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Utility macro for counting identifiers, needed for constant array
/// definitions. Not intended for direct use.
///
/// This macro counts the number of identifiers passed to it. It is primarily
/// used for defining constant arrays where the length needs to be determined
/// at compile time.
///
/// Used by [`define_string_enum_field!`].
///
/// [`define_string_enum_field!`]: super::define_string_enum_field!
macro_rules! count_idents {
    ($($ident:ident),*) => {
        <[()]>::len(&[$($crate::settings::fields::macros::utility::count_idents!(@replace $ident)),*])
    };
    (@replace $ident:ident) => { () };
}

pub(crate) use count_idents;

/// Utility macro for generating a comma-separated list of literals enclosed in
/// backticks. Not intended for direct use.
///
/// This macro is primarily used for defining the `VARIANT_LIST` and
/// `VARIANT_LAST` constants in the [`define_variant_consts!`] macro.
///
/// Used by [`define_variant_consts!`] in [`define_string_enum_field!`].
///
/// [`define_string_enum_field!`]: super::define_string_enum_field!
macro_rules! backtick_list {
    ($last:literal) => {
        concat!("`", $last, "`")
    };
    ($first:literal, $($rest:literal),+) => {
        concat!(
            "`",
            $first,
            "`, ",
            $crate::settings::fields::macros::utility::backtick_list!($($rest),+)
        )
    };
}

pub(crate) use backtick_list;

/// Defines constants for the variants of a string enum field. Not intended for
/// direct use.
///
/// This macro generates two private (crate public) constants:
///
/// - `VARIANT_LIST`: A comma-separated list of all variant names except the
///   last one, formatted with backticks.
/// - `VARIANT_LAST`: The last variant name, formatted with backticks.
///
/// Used by [`define_string_enum_field!`].
///
/// [`define_string_enum_field!`]: super::define_string_enum_field!
macro_rules! define_variant_consts {
    ($($item:literal),+ $(,)?) => {
        $crate::settings::fields::macros::utility::define_variant_consts!(@split [] $($item),+);
    };

    // Only one identifier remains, so it is the final variant.
    (@split [$($preceding:literal),+] $last:literal) => {
        /// A comma-separated list of all variant names except the last one,
        /// formatted with backticks. Only intended for error messages.
        pub(crate) const VARIANT_LIST: &'static str =
            $crate::settings::fields::macros::utility::backtick_list!($($preceding),+);

        /// The last variant name, formatted with backticks. Only intended for
        /// error messages.
        pub(crate) const VARIANT_LAST: &'static str =
            $crate::settings::fields::macros::utility::backtick_list!($last);
    };

    // Move identifiers into the accumulator until one remains.
    (@split [$($preceding:literal),*] $next:literal, $($rest:literal),+) => {
        $crate::settings::fields::macros::utility::define_variant_consts!(@split [$($preceding,)* $next] $($rest),+);
    };
}

pub(crate) use define_variant_consts;
