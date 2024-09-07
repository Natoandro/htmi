use crate::{parser::Parser, render::Render as _};
use proc_macro::{TokenStream, TokenTree};
use quote::quote;

mod document;
mod parser;
mod render;

#[proc_macro]
pub fn render(input: TokenStream) -> TokenStream {
    let root = Parser::parse(input);
    let mut trees = vec![];
    root.render_into(&mut trees);
    let group: TokenStream = TokenTree::Group(proc_macro::Group::new(
        proc_macro::Delimiter::Bracket,
        trees
            .into_iter()
            .flat_map(|tree| {
                [
                    tree,
                    TokenTree::Punct(proc_macro::Punct::new(',', proc_macro::Spacing::Alone)),
                ]
            })
            .collect(),
    ))
    .into();
    let group: proc_macro2::TokenStream = group.into();
    let stream: proc_macro2::TokenStream = quote! { #group .join("") }.into();

    #[cfg(debug_assertions)]
    eprintln!("{}", stream);

    stream.into()
}
