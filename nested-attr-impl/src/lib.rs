//! Implementation for nested definition attr.

use ::convert_case::{Case, Casing};
use ::proc_macro2::TokenStream;
use ::quote::ToTokens;
use ::syn::{
    AttrStyle, Attribute, Generics, Ident, Token, Type, Visibility, braced,
    parse::{Parse, Parser},
    punctuated::{Pair, Punctuated},
    token::Brace,
};

/// Implementation of nested attribute macro
pub fn nested(item: TokenStream) -> TokenStream {
    parse_nested
        .parse2(item)
        .unwrap_or_else(::syn::Error::into_compile_error)
}

/// Syn parser for nested struct.
fn parse_nested(input: ::syn::parse::ParseStream) -> ::syn::Result<TokenStream> {
    let mut inner_attrs = input.call(Attribute::parse_inner)?;
    for attr in &mut inner_attrs {
        attr.style = AttrStyle::Outer;
    }
    let global_attrs = inner_attrs;

    let nested_struct = input.parse()?;
    let mut tokens = TokenStream::default();
    NestedStruct::write_split(&nested_struct, &mut tokens, &global_attrs);
    Ok(tokens)
}

/// Identity of nested struct, either just a type name or a field name and a type name.
enum NestedStructIdent {
    /// Identity is just a type name.
    Ident(Ident),
    /// Identity is a field and type name.
    FieldTyIdent {
        /// Name of field in parent.
        field: Ident,
        /// Colon token.
        colon_token: Token![:],
        /// Type attributes.
        attrs: Vec<Attribute>,
        /// Name of type.
        ty_ident: Ident,
    },
}

impl Parse for NestedStructIdent {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let first = input.parse()?;

        if input.peek(Token![:]) {
            let colon_token = input.parse()?;
            let attrs = input.call(Attribute::parse_outer)?;
            let ty_ident = input.parse()?;
            Ok(Self::FieldTyIdent {
                field: first,
                colon_token,
                attrs,
                ty_ident,
            })
        } else {
            Ok(Self::Ident(first))
        }
    }
}

/// Nested struct definition.
struct NestedStruct {
    /// struct attributes.
    attrs: Vec<Attribute>,
    /// struct visibility.
    vis: Visibility,
    /// struct token.
    struct_token: Token![struct],
    /// Identity of struct.
    ident: NestedStructIdent,
    /// Generics and where clause of struct.
    generics: Generics,
    /// Fields of struct.
    fields: NestedStructFields,
}

impl NestedStruct {
    /// Parse other than attrs and vis.
    fn parse_partial(
        input: ::syn::parse::ParseStream,
        attrs: Vec<Attribute>,
        vis: Visibility,
    ) -> ::syn::Result<Self> {
        let struct_token = input.parse()?;
        let ident = input.parse()?;
        let generics = input.parse()?;
        let fields = input.parse()?;
        let mut where_clause = None;

        if input.peek(Token![where]) {
            where_clause = Some(input.parse()?);
        }

        Ok(Self {
            attrs,
            vis,
            struct_token,
            ident,
            generics: Generics {
                where_clause,
                ..generics
            },
            fields,
        })
    }

    /// Split nesting and write to token stream.
    fn write_split(&self, tokens: &mut TokenStream, global_attrs: &[Attribute]) {
        let Self {
            attrs,
            vis,
            struct_token,
            ident,
            generics,
            fields,
        } = self;
        let (attrs, ident) = match ident {
            NestedStructIdent::Ident(ident) => (attrs, ident),
            NestedStructIdent::FieldTyIdent {
                attrs, ty_ident, ..
            } => (attrs, ty_ident),
        };
        for attr in global_attrs {
            attr.to_tokens(tokens);
        }
        for attr in attrs {
            attr.to_tokens(tokens);
        }

        vis.to_tokens(tokens);
        struct_token.to_tokens(tokens);
        ident.to_tokens(tokens);
        generics.to_tokens(tokens);
        generics.where_clause.to_tokens(tokens);

        let NestedStructFields::Named { brace_token, named } = fields else {
            ::syn::token::Semi {
                spans: [ident.span()],
            }
            .to_tokens(tokens);
            return;
        };

        let mut split = Vec::new();

        brace_token.surround(tokens, |tokens| {
            for (named_field, punct) in named.pairs().map(Pair::into_tuple) {
                match named_field {
                    NamedField::Struct(
                        nested @ NestedStruct {
                            attrs,
                            vis,
                            ident,
                            generics,
                            ..
                        },
                    ) => {
                        split.push(nested);
                        let (_, generics, _) = generics.split_for_impl();
                        let generics = generics.as_turbofish();
                        let field;
                        let colon_token;
                        let (field, colon_token, ty_ident) = match ident {
                            NestedStructIdent::Ident(ty_ident) => {
                                field = Ident::new(
                                    &ty_ident.to_string().to_case(Case::Snake),
                                    ty_ident.span(),
                                );
                                colon_token = ::syn::token::Colon {
                                    spans: [ty_ident.span()],
                                };

                                for attr in attrs {
                                    if attr.path().is_ident("doc") {
                                        attr.to_tokens(tokens);
                                    }
                                }

                                (&field, &colon_token, ty_ident)
                            }
                            NestedStructIdent::FieldTyIdent {
                                field,
                                colon_token,
                                ty_ident,
                                ..
                            } => {
                                for attr in attrs {
                                    attr.to_tokens(tokens);
                                }
                                (field, colon_token, ty_ident)
                            }
                        };
                        vis.to_tokens(tokens);
                        field.to_tokens(tokens);
                        colon_token.to_tokens(tokens);
                        ty_ident.to_tokens(tokens);
                        generics.to_tokens(tokens);
                    }
                    NamedField::NameTy {
                        attrs,
                        vis,
                        ident,
                        colon_token,
                        ty,
                    } => {
                        for attr in attrs {
                            attr.to_tokens(tokens);
                        }
                        vis.to_tokens(tokens);
                        ident.to_tokens(tokens);
                        colon_token.to_tokens(tokens);
                        ty.to_tokens(tokens);
                    }
                }
                punct.to_tokens(tokens);
            }
        });

        for nested in split {
            nested.write_split(tokens, global_attrs);
        }
    }
}

impl Parse for NestedStruct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        Self::parse_partial(input, attrs, vis)
    }
}

/// Fields of a nested struct.
enum NestedStructFields {
    /// Struct is a unit struct.
    Unit,
    /// Struct has named fields.
    Named {
        /// Brace '{' tokens surrounding fields.
        brace_token: Brace,
        /// Fields
        named: Punctuated<NamedField, Token![,]>,
    },
}

impl Parse for NestedStructFields {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Brace) {
            let buffer;
            let brace_token = braced!(buffer in input);
            let named = Punctuated::parse_terminated(&buffer)?;

            Ok(Self::Named { brace_token, named })
        } else {
            Ok(Self::Unit)
        }
    }
}

/// A named field in a nested struct.
enum NamedField {
    /// Field is a furhter nested struct.
    Struct(NestedStruct),
    /// Field is a regular struct field.
    NameTy {
        /// Field attributes.
        attrs: Vec<Attribute>,
        /// Field visibility.
        vis: Visibility,
        /// Field ident.
        ident: Ident,
        /// Colon token.
        colon_token: Token![:],
        /// Field type.
        ty: Type,
    },
}

impl Parse for NamedField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;

        if input.peek(Token![struct]) {
            NestedStruct::parse_partial(input, attrs, vis).map(Self::Struct)
        } else {
            let ident = input.parse()?;
            let colon_token = input.parse()?;
            let ty = input.parse()?;
            Ok(Self::NameTy {
                attrs,
                vis,
                ident,
                colon_token,
                ty,
            })
        }
    }
}
