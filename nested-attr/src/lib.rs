//! Nested struct definitions.

use ::proc_macro::TokenStream;

/// Macro to declare nested struct definitions.
#[proc_macro]
pub fn nested(item: TokenStream) -> TokenStream {
    ::nested_attr_impl::nested(item.into()).into()
}
