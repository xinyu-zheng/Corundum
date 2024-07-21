extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use syn::parse::{Parse, ParseStream};

struct MacroInput {
    pub field_type: syn::Type,
    pub field_name: String,
    pub field_count: u64,
}
impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field_type = input.parse::<syn::Type>()?;
        let _comma = input.parse::<syn::token::Comma>()?;
        let field_name = input.parse::<syn::LitStr>()?;
        let _comma = input.parse::<syn::token::Comma>()?;
        let count = input.parse::<syn::LitInt>()?;
        Ok(MacroInput {
            field_type: field_type,
            field_name: field_name.value(),
            field_count: count.base10_parse().unwrap(),
        })
    }
}

#[proc_macro_attribute]
pub fn derivefields(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(attr as MacroInput);
    let mut found_struct = false;
    item.into_iter()
        .map(|r| match &r {
            &proc_macro::TokenTree::Ident(ref ident) if ident.to_string() == "struct" => {
                found_struct = true;
                r
            }
            &proc_macro::TokenTree::Group(ref group)
                if group.delimiter() == proc_macro::Delimiter::Brace && found_struct == true =>
            {
                let mut stream = proc_macro::TokenStream::new();
                stream.extend(
                    (1..input.field_count)
                        .fold(vec![], |mut state: Vec<proc_macro::TokenStream>, i| {
                            let field_name_str = format!("{}{}", input.field_name, i);
                            let field_name = Ident::new(&field_name_str, Span::call_site());
                            let field_type = input.field_type.clone();
                            state.push(
                                quote!(pub #field_name: #field_type,
                                )
                                .into(),
                            );
                            state
                        })
                        .into_iter(),
                );
                stream.extend(group.stream());
                proc_macro::TokenTree::Group(proc_macro::Group::new(
                    proc_macro::Delimiter::Brace,
                    stream,
                ))
            }
            _ => r,
        })
        .collect()
}
