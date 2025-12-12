// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// Simplifies passing either a literal string or an expression that evaluates to a string for the
/// annotation fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum StringOrExpr {
    Expr(Box<syn::Expr>),
    String(String),
}

impl darling::FromMeta for StringOrExpr {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        Ok(Self::Expr(Box::new(expr.clone())))
    }
    fn from_string(value: &str) -> darling::Result<Self> {
        Ok(Self::String(value.to_string()))
    }
    fn from_value(value: &syn::Lit) -> darling::Result<Self> {
        match value {
            syn::Lit::Str(v) => Ok(Self::String(v.value())),
            _ => Err(darling::Error::unexpected_lit_type(value)),
        }
    }
}

impl quote::ToTokens for StringOrExpr {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Expr(expr) => expr.to_tokens(tokens),
            Self::String(str) => str.to_tokens(tokens),
        }
    }
}
