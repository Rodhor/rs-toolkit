extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};

#[proc_macro_derive(FuzzyFromStr)]
pub fn fuzzy_from_str(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let variants = match input.data {
        Data::Enum(data) => data.variants,
        _ => panic!("FuzzyDeserialize can only be used on Enums!"),
    };

    let match_arms = variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let variant_str = variant_ident.to_string();

        let normalized_target = variant_str.to_lowercase().replace(['-', ' ', '_'], "");

        quote! {
            #normalized_target => Ok(Self::#variant_ident),
        }
    });

    let expanded = quote! {
        impl std::str::FromStr for #name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let cleaned = s.to_lowercase()
                    .replace(['-', ' ', '_'], "");

                match cleaned.as_str() {
                    #(#match_arms)*
                    _ => Err(format!("Invalid option '{}' for enum '{}'", s, stringify!(#name))),
                }
            }
        }
    };
    TokenStream::from(expanded)
}
