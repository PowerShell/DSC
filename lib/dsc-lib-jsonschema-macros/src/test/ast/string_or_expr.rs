// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

use darling::FromMeta;

use crate::ast::StringOrExpr;

#[test] fn test_string_literal() {
    let code = syn::parse_quote!("literal");
    let parsed = StringOrExpr::from_value(&code);
    
    assert_eq!(parsed.unwrap(), StringOrExpr::String("literal".to_string()));
}

#[test] fn test_expr_fn() {
    let code = syn::parse_quote!(get_value());
    let parsed = StringOrExpr::from_expr(&code);

    assert_eq!(parsed.unwrap(), StringOrExpr::Expr(Box::new(code)));
}

#[test] fn test_expr_macro() {
    let code = syn::parse_quote!(get_value!());
    let parsed = StringOrExpr::from_expr(&code);

    assert_eq!(parsed.unwrap(), StringOrExpr::Expr(Box::new(code)));
}
