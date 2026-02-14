//! Proc-macro attributes for NewRow-Print! taint specification.
//!
//! These macros do not perform full data-flow analysis themselves;
//! instead, they mark critical items and perform cheap syntactic
//! checks that surface as compiler errors when obvious violations
//! occur (e.g., unsafe fn on a critical type).
//!
//! A separate static analyzer can consume the marker metadata
//! via `cargo check --message-format json` if deeper analysis
//! is needed.

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, AttributeArgs, Item, ItemFn, ItemMod, ItemType, Meta, NestedMeta,
};

/// #[nr_taint_critical]
///
/// Marks a type alias or item as policy-critical.
/// For now this is a pure marker; deeper checks are done
/// by the analyzer that reads the compiled metadata.
#[proc_macro_attribute]
pub fn nr_taint_critical(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(args as AttributeArgs);
    let item = parse_macro_input!(input as Item);

    // Inject a doc flag so the analyzer can discover this easily.
    let expanded = match item {
        Item::Type(ItemType { attrs, vis, type_token, ident, generics, eq_token, ty, semi_token }) => {
            let mut attrs = attrs;
            attrs.push(syn::parse_quote!(#[doc(hidden)]));
            quote! {
                #(#attrs)*
                #vis #type_token #ident #generics #eq_token #ty #semi_token
            }
        }
        other => {
            quote! {
                #[doc(hidden)]
                #other
            }
        }
    };

    expanded.into()
}

/// #[nr_taint_trusted_writer]
///
/// Marks a function as an allowed writer of critical types.
/// Enforces a small syntactic rule: the function itself cannot be `unsafe`.
#[proc_macro_attribute]
pub fn nr_taint_trusted_writer(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(args as AttributeArgs);
    let item = parse_macro_input!(input as Item);

    match item {
        Item::Fn(ref fn_item) => {
            if fn_item.sig.unsafety.is_some() {
                let ident = &fn_item.sig.ident;
                let err = syn::Error::new_spanned(
                    &fn_item.sig,
                    format!(
                        "nr_taint_trusted_writer: trusted writer `{}` must not be `unsafe`",
                        ident
                    ),
                );
                return err.to_compile_error().into();
            }
        }
        _ => {
            let err = syn::Error::new_spanned(
                item.to_token_stream(),
                "#[nr_taint_trusted_writer] may only be applied to functions",
            );
            return err.to_compile_error().into();
        }
    }

    // For now, act as a pure marker. The analyzer can pick up the
    // attribute via the macro path in metadata.
    let tokens = quote! { #item };
    tokens.into()
}

/// #[nr_taint_trusted_reader]
///
/// Marks a module as a read-only consumer of critical types.
/// Syntactic guard: must be used on modules, not functions.
#[proc_macro_attribute]
pub fn nr_taint_trusted_reader(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(args as AttributeArgs);
    let item = parse_macro_input!(input as Item);

    match item {
        Item::Mod(ItemMod { .. }) => {
            let tokens = quote! { #item };
            tokens.into()
        }
        _ => {
            let err = syn::Error::new_spanned(
                item.to_token_stream(),
                "#[nr_taint_trusted_reader] may only be applied to modules",
            );
            err.to_compile_error().into()
        }
    }
}

/// #[nr_taint_diag_join]
///
/// Marks the single diagnostic join point where tainted evidence
/// (Tree-of-Life, Neuroprint, envelopes, AutoChurch) may be joined
/// into `nosaferalternative`.
///
/// Syntactic guards:
/// - Must be applied to a function.
/// - Must not be `unsafe`.
#[proc_macro_attribute]
pub fn nr_taint_diag_join(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = parse_macro_input!(args as AttributeArgs);
    let item = parse_macro_input!(input as Item);

    match item {
        Item::Fn(ref fn_item) => {
            if fn_item.sig.unsafety.is_some() {
                let ident = &fn_item.sig.ident;
                let err = syn::Error::new_spanned(
                    &fn_item.sig,
                    format!(
                        "nr_taint_diag_join: diagnostic join point `{}` must not be `unsafe`",
                        ident
                    ),
                );
                return err.to_compile_error().into();
            }
            // Could add further syntactic checks here (e.g., return type),
            // but deeper semantic checks should live in the analyzer.
            let tokens = quote! { #fn_item };
            tokens.into()
        }
        _ => {
            let err = syn::Error::new_spanned(
                item.to_token_stream(),
                "#[nr_taint_diag_join] may only be applied to functions",
            );
            err.to_compile_error().into()
        }
    }
}
