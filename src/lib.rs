//! A simple way to run code before and after tests.
//!
//! Suppose you want to set up a tracing subscriber to display log and tracing
//! events before some tests:
//!
//! ```
//! #[cfg(test)]
//! #[wraptest::wrap_tests(before = setup_logs)]
//! mod tests {
//!     use tracing::info;
//!     use tracing_subscriber::fmt::format::FmtSpan;
//!
//!     fn setup_logs() {
//!         tracing_subscriber::fmt::fmt()
//!             .with_env_filter("debug")
//!             .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
//!             .init();
//!     }
//!
//!     #[test]    
//!     fn with_tracing() {
//!         info!("with tracing");
//!     }
//!
//!     #[tokio::test]
//!     async fn with_tracing_async() {
//!         info!("with tracing -- but async");
//!     }
//! }
//! ```
//!
//! This translates to essentially:
//!
//! ```
//! # use tracing::info;
//! #
//! #[test]
//! fn with_tracing() {
//!     fn with_tracing() {
//!         info!("with tracing");
//!     }
//!     setup_logs();
//!     with_tracing();
//! }
//!
//! #[tokio::test]
//! async fn with_tracing_async() {
//!     async fn with_tracing_async() {
//!         info!("with tracing -- but async");
//!     }
//!     setup_logs();
//!     with_tracing_async().await;
//! }
//! ```
//!
//! You can also specify `#[wraptest(after = after_fn)]` to run code after each
//! test.
//!
//! ## Prior Art
//! I got the idea of simplifying redundant test setup with macros from the
//! excellent [test-env-log][test-env-log].
//!
//! [test-env-log]: https://github.com/d-e-s-o/test-env-log
//!
#![warn(clippy::cargo)]

use proc_macro2::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    visit_mut::{self, VisitMut},
    ItemFn, ItemMod, Token,
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

#[proc_macro_error]
#[proc_macro_attribute]
pub fn wrap_tests(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let Args { before, after } = parse_macro_input!(args as Args);
    let mut module = parse_macro_input!(input as ItemMod);

    let mut visitor = ModVisitor { before, after };
    visitor.visit_item_mod_mut(&mut module);

    let out = quote! { #module };
    out.into()
}

struct ModVisitor {
    before: Option<Ident>,
    after: Option<Ident>,
}

impl VisitMut for ModVisitor {
    fn visit_item_fn_mut(&mut self, node: &mut ItemFn) {
        if self.is_test_fn(node) {
            if !node.sig.inputs.is_empty() {
                abort!(
                    node.sig.inputs,
                    "wraptest doesn't support test functions that take arguments"
                );
            }

            if node.sig.asyncness.is_some() {
                self.visit_async_test_fn(node);
            } else {
                self.visit_non_async_test_fn(node);
            }
        }

        visit_mut::visit_item_fn_mut(self, node);
    }
}

impl ModVisitor {
    fn is_test_fn(&self, node: &ItemFn) -> bool {
        node.attrs.iter().any(|attr| {
            if attr.path.is_ident("test") {
                return true;
            }

            let pairs = attr
                .path
                .segments
                .pairs()
                .map(|pair| pair.value().ident.to_string())
                .collect::<Vec<_>>();
            if pairs.len() == 2 && pairs[0] == "tokio" && pairs[1] == "test" {
                return true;
            }

            false
        })
    }

    fn visit_async_test_fn(&mut self, node: &mut ItemFn) {
        let call_before = self.call_before_quote();
        let call_after = self.call_after_quote();

        let wrapped = Self::without_attrs(node);
        let name = &wrapped.sig.ident;

        node.block.stmts = parse_quote! {
            #wrapped
            #call_before
            let result = #name().await;
            #call_after
            result
        }
    }

    fn visit_non_async_test_fn(&mut self, node: &mut ItemFn) {
        let call_before = self.call_before_quote();
        let call_after = self.call_after_quote();

        let wrapped = Self::without_attrs(node);
        let name = &wrapped.sig.ident;

        node.block.stmts = parse_quote! {
            #wrapped
            #call_before
            let result = #name();
            #call_after
            result
        }
    }

    fn without_attrs(node: &ItemFn) -> ItemFn {
        let mut node = node.clone();
        node.attrs = vec![];
        node
    }

    fn call_before_quote(&mut self) -> TokenStream {
        if let Some(before) = &self.before {
            quote! { #before(); }
        } else {
            quote! {}
        }
    }

    fn call_after_quote(&mut self) -> TokenStream {
        if let Some(after) = &self.after {
            quote! { #after(); }
        } else {
            quote! {}
        }
    }
}
