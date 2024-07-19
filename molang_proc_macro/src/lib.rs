use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Data, DeriveInput, Ident, Type};

#[proc_macro_derive(MolangStruct)]
pub fn molang_struct_macro(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    molang_struct(input.into()).into()
}

fn molang_struct(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse2(input).unwrap();

    let name = input.ident;

    let fields = match input.data {
        Data::Struct(st) => st.fields,
        _ => return quote! { compile_error!("Only supported for structs") }.into(),
    };

    let field_idents: Vec<Ident> = fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect();

    let field_types: Vec<Type> = fields
        .iter()
        .map(|field| field.ty.clone())
        .collect();

    quote! {
        impl molang::ToMolangValue for #name {
            fn to_value(self) -> molang::Value {
                let mut fields = std::collections::HashMap::new();
                #(fields.insert(stringify!(#field_idents).to_string(), self.#field_idents.to_value());)*
                molang::Value::Struct(fields)
            }
        }

        impl molang::FromMolangValue for #name {
            fn from_value(v: molang::Value) -> Result<Self, molang::MolangError> {
                match v {
                    molang::Value::Struct(mut st) => {
                        Ok(#name { #( #field_idents : 
                            match st.remove(&stringify!(#field_idents).to_string()) {
                                Some(x) => #field_types::from_value(x)?,
                                None => return Err(molang::MolangError::TypeError(stringify!(#field_types).to_string(), "None".to_string()))
                            },
                        )* })
                    },
                    a => Err(molang::MolangError::TypeError("Struct".to_string(), format!("{a:?}")))
                }
            }
        }
    }.into()
}

#[test]
fn my_test() {
    println!(
        "{}",
        molang_struct(quote! {
            struct testing {
                a: f32,
            }
        })
    );
    panic!("abc");
}