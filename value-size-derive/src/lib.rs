use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse_quote,
    punctuated::Punctuated,
    token::{Comma, Plus},
    Data, DataEnum, DataStruct, DeriveInput, Fields, TypeParam, Variant, WherePredicate,
};

struct MaybeVariant {
    ident: Option<Ident>,
    fields: Fields,
}

#[proc_macro_derive(Size)]
pub fn derive_size(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input: DeriveInput = syn::parse_macro_input!(input);
    let ident = input.ident;
    let variants = match input.data {
        Data::Struct(DataStruct { fields, .. }) => vec![MaybeVariant {
            ident: None,
            fields,
        }],
        Data::Enum(DataEnum { variants, .. }) => variants
            .into_iter()
            .map(|Variant { ident, fields, .. }| MaybeVariant {
                ident: Some(ident),
                fields,
            })
            .collect(),
        Data::Union(_) => panic!("unions are for people, not data!"),
    };

    let mut indirect_size_match_clauses = TokenStream::new();
    for variant in variants {
        let path = if let Some(variant) = variant.ident {
            quote! { #ident::#variant }
        } else {
            quote! { #ident }
        };
        let (fields_pat, field_names) = match variant.fields {
            Fields::Named(fields) => {
                let names = fields
                    .named
                    .into_iter()
                    .map(|f| f.ident.unwrap())
                    .collect::<Punctuated<Ident, Comma>>();
                (quote! { { #names } }, names)
            }
            Fields::Unnamed(fields) => {
                let names = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format_ident!("field{i}"))
                    .collect::<Punctuated<Ident, Comma>>();
                (quote! { ( #names )}, names)
            }
            Fields::Unit => (quote! {}, Punctuated::new()),
        };
        let field_sizes = field_names
            .into_iter()
            .map(|f| quote! { value_size::Size::indirect_size(#f) })
            .collect::<Punctuated<_, Plus>>();
        indirect_size_match_clauses.extend(quote! { #path #fields_pat => #field_sizes, });
    }
    let artificial_where_predicates = input
        .generics
        .type_params()
        .map(
            |TypeParam {
                 ident: tp_ident, ..
             }| parse_quote! { #tp_ident: value_size::Size },
        )
        .collect::<Vec<WherePredicate>>();
    input
        .generics
        .make_where_clause()
        .predicates
        .extend(artificial_where_predicates);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics value_size::Size for #ident #ty_generics #where_clause {
            fn indirect_size(&self) -> usize {
                match self {
                    #indirect_size_match_clauses
                }
            }
        }
    }
    .into()
}
