//! A simple way to run code before and after tests.
//!
//! Suppose you want to set up a tracing subscriber to display log and tracing
//! events before some tests:
//!
//! ```
//! # use tracing::info;
//! # use tracing_subscriber::fmt::format::FmtSpan;
//! # use wraptest::wraptest;
//! #
//! fn setup_logs() {
//!     tracing_subscriber::fmt::fmt()
//!         .with_env_filter("debug")
//!         .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
//!         .init();
//! }
//!
//! #[wraptest(before = setup_logs)]
//! fn with_tracing() {
//!     info!("with tracing");
//! }
//! ```
//!
//! This translates to essentially:
//!
//! ```
//! # use tracing::info;
//! # use wraptest::wraptest;
//! #
//! #[test]
//! fn with_tracing() {
//!     fn with_tracing() {
//!         info!("with tracing");
//!     }
//!     setup_logs();
//!     with_tracing();
//! }
//! ```
//!
//! Async functions and running code after your test are also supported.
//!
//!
//! ```
//! # use wraptest::wraptest;
//! #
//! fn before() {
//!     eprintln!("Called before");
//! }
//!
//! fn after() {
//!     eprintln!("Called after");
//! }
//!
//! #[wraptest(before = before, after = after)]
//! async fn it_works_async() {
//!     assert_eq!(2 + 2, 4);
//! }
//! ```
//!
//! ## Prior Art
//! I got the idea of simplifying redundant test setup with macros from the
//! excellent [test-env-log][test-env-log].
//!
//! [test-env-log]: https://github.com/d-e-s-o/test-env-log
//!
#![warn(clippy::cargo)]

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    ItemFn, Token,
};
use syn::{parse_macro_input, Ident};

struct Args {
    before: Option<Ident>,
    after: Option<Ident>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut before = None;
        let mut after = None;

        let args = Punctuated::<Arg, Token![,]>::parse_terminated(input)?;

        for pair in args.into_pairs() {
            match pair.into_value() {
                Arg::Before(ident) => before = Some(ident),
                Arg::After(ident) => after = Some(ident),
            }
        }

        Ok(Self { before, after })
    }
}

enum Arg {
    Before(Ident),
    After(Ident),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let value = input.parse::<Ident>()?;

        let arg = match name.to_string().as_str() {
            "before" => Arg::Before(value),
            "after" => Arg::After(value),
            _ => abort!(name, "Unexpected argument name"),
        };
        Ok(arg)
    }
}

// TODO: Support defining default before/after

#[proc_macro_error]
#[proc_macro_attribute]
pub fn wraptest(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let wrapped = parse_macro_input!(input as ItemFn);

    let sig = &wrapped.sig;
    let input_ident = &sig.ident;
    let is_async = sig.asyncness.is_some();

    if !sig.inputs.is_empty() {
        abort!(sig.inputs, "wraptest: Test functions cannot take arguments")
    }

    let call_input = if is_async {
        quote! {
            #input_ident().await
        }
    } else {
        quote! { #input_ident() }
    };

    let test_attr = if is_async {
        quote! { #[::tokio::test] }
    } else {
        quote! { #[::core::prelude::v1::test] }
    };

    let test_fn_prefix = if is_async {
        quote! { async }
    } else {
        quote! {}
    };

    let Args { before, after } = args;
    let call_before = if let Some(before) = before {
        quote! { #before(); }
    } else {
        quote! {}
    };
    let call_after = if let Some(after) = after {
        quote! { #after(); }
    } else {
        quote! {}
    };

    let out = quote! {
        #test_attr
        #test_fn_prefix fn #input_ident() {
            #wrapped
            #call_before;
            #call_input;
            #call_after;
        }
    };
    TokenStream::from(out)
}
