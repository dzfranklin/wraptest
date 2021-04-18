//! A simple way to run code before or after every unit test.
//!
//! The wrapper function you specify is called with each of your tests. In the
//! wrapper you do any setup you want, call the test function you were provided,
//! and then do any cleanup.
//!
//! # Examples
//!
//! ## Basic
//!
//! Suppose you want to set up a tracing subscriber to display log and tracing
//! events before some tests:
//!
//! ```
//! #[wraptest::wrap_tests(wrapper = with_logs)]
//! mod tests {
//!     use tracing::info;
//!     use tracing_subscriber::fmt::format::FmtSpan;
//!
//!     fn with_logs<T: FnOnce() -> ()>(test_fn: T) {
//!         let subscriber = tracing_subscriber::fmt::fmt()
//!            .with_env_filter("debug")
//!            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
//!            .with_test_writer()
//!            .finish();
//!         let _guard = tracing::subscriber::set_default(subscriber);
//!         test_fn();
//!     }
//!
//!     #[test]    
//!     fn with_tracing() {
//!         info!("with tracing!");
//!     }
//! }
//! ```
//!
//! ## Async
//!
//! If you have async tests (currently only [`tokio::test`] is supported) you
//! can provide an async wrapper.
//!
//! ```
//! #[wraptest::wrap_tests(async_wrapper = with_logs)]
//! mod tests {
//! #   use tracing::info;
//! #   use tracing_subscriber::fmt::format::FmtSpan;
//! #   use std::future::Future;
//! #   
//!     async fn with_logs<T, F>(test_fn: T)
//!     where
//!         T: FnOnce() -> F,
//!         F: Future<Output = ()>,
//!     {
//!         let subscriber = /* ... */
//! #           tracing_subscriber::fmt::fmt()
//! #               .with_env_filter("debug")
//! #               .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
//! #               .with_test_writer()
//! #               .finish();
//!         let _guard = tracing::subscriber::set_default(subscriber);
//!         test_fn();
//!     }
//!
//!     #[tokio::test]    
//!     async fn with_tracing() {
//!         info!("with tracing, but async!");
//!     }
//! }
//! ```
//!
//! ## Custom return type
//!
//! If you want to return something other than `()` from your tests you just
//! need to change the signature of your wrapper. Here's how you can make your
//! wrappers generic over any return type:
//!
//! ```
//! #[wraptest::wrap_tests(wrapper = with_logs, async_wrapper = with_logs_async)]
//! mod tests {
//!     # use std::{future::Future, time::Duration};
//!
//!     fn with_logs<T, R>(test_fn: T) -> R
//!     where
//!         T: FnOnce() -> R,
//!     {
//!         eprintln!("Setting up...");
//!         let result = test_fn();
//!         eprintln!("Cleaning up...");
//!         result
//!     }
//!
//!     async fn with_logs_async<T, F, R>(test_fn: T) -> R
//!     where
//!         T: FnOnce() -> F,
//!         F: Future<Output = R>,
//!     {
//!         eprintln!("Setting up...");
//!         let result = test_fn().await;
//!         eprintln!("Cleaning up...");
//!         result
//!     }
//! }
//! ```

#![warn(clippy::cargo)]

use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    visit_mut::{self, VisitMut},
    Ident, ItemFn, ItemMod, Token,
};

const USAGE: &str = "Usage is generally `#[wraptest::wrap_tests(wrapper = your_fn)]`, or
`#[wraptest::wrap_tests(wrapper = your_fn, async_wrapper = your_fn_async)]` if
you have async tests.";

struct Args {
    wrapper: Option<Ident>,
    async_wrapper: Option<Ident>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let punct = Punctuated::<Arg, Token![,]>::parse_terminated(input)?;

        let mut wrapper = None;
        let mut async_wrapper = None;

        for pair in punct.into_pairs() {
            match pair.into_value() {
                Arg::Wrapper(ident) => wrapper = Some(ident),
                Arg::AsyncWrapper(ident) => async_wrapper = Some(ident),
            }
        }

        Ok(Self {
            wrapper,
            async_wrapper,
        })
    }
}

enum Arg {
    Wrapper(Ident),
    AsyncWrapper(Ident),
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let value = input.parse::<Ident>()?;

        let arg = if name == "wrapper" {
            Self::Wrapper(value)
        } else if name == "async_wrapper" {
            Self::AsyncWrapper(value)
        } else {
            abort!(name, "wraptest: Unexpected parameter"; note = USAGE)
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
    let Args {
        wrapper,
        async_wrapper,
    } = parse_macro_input!(args as Args);
    let mut module = parse_macro_input!(input as ItemMod);

    let mut visitor = ModVisitor {
        wrapper,
        async_wrapper,
    };
    visitor.visit_item_mod_mut(&mut module);

    let out = quote! { #module };
    out.into()
}

struct ModVisitor {
    wrapper: Option<Ident>,
    async_wrapper: Option<Ident>,
}

impl VisitMut for ModVisitor {
    fn visit_item_fn_mut(&mut self, node: &mut ItemFn) {
        if self.is_test_fn(node) {
            if !node.sig.inputs.is_empty() {
                abort!(
                    node.sig.inputs,
                    "wraptest: Test functions that take arguments aren't supported";
                    note = USAGE,
                );
            }

            self.visit_test_fn(node);
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

    fn visit_test_fn(&mut self, node: &mut ItemFn) {
        let wrapped = Self::strip_attrs(node);
        let name = &wrapped.sig.ident;

        node.block.stmts = if node.sig.asyncness.is_some() {
            let async_wrapper = match &self.async_wrapper {
                Some(wrapper) => wrapper,
                None => abort!(
                    node,
                    "wraptest: Must specify `async_wrapper` to wrap async test functions";
                    note = USAGE
                ),
            };

            parse_quote! {
                #wrapped
                #async_wrapper(#name).await
            }
        } else {
            let wrapper = match &self.wrapper {
                Some(wrapper) => wrapper,
                None => abort!(
                    node,
                    "wraptest: Must specify `wrapper` to wrap async test functions";
                    note = USAGE
                ),
            };
            parse_quote! {
                #wrapped
                #wrapper(#name)
            }
        };
    }

    fn strip_attrs(node: &ItemFn) -> ItemFn {
        let mut node = node.clone();
        node.attrs = vec![];
        node
    }
}
