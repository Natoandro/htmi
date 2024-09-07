use crate::document::{Attribute, Node};
use proc_macro::{Literal, TokenTree};
use quote::quote;

pub trait Render {
    fn render_into(self, target: &mut Vec<TokenTree>);
}

impl Render for Node {
    fn render_into(self, target: &mut Vec<TokenTree>) {
        match self {
            Node::Text(text) => {
                target.push(quote_str(&text));
            }
            Node::Element(el) => {
                target.push(quote_str(&format!("<{}", el.tag)));
                for attr in el.attributes {
                    match attr {
                        Attribute::Empty(name) => {
                            target.push(quote_str(&format!(" {}", name)));
                        }
                        Attribute::Literal(name, value) => {
                            // TODO escaping
                            target.push(quote_str(&format!(" {}={}", name, value)));
                        }
                        Attribute::Expression(name, value) => {
                            target.push(quote_str(&format!(" {}=", name)));
                            let value: syn::Expr =
                                syn::parse(value).expect("could not parse expression");
                            // TODO excape strings in value?
                            let stream: proc_macro::TokenStream =
                                quote!(& #value .to_string()).into();
                            let group = TokenTree::Group(proc_macro::Group::new(
                                proc_macro::Delimiter::Parenthesis,
                                stream.into_iter().collect(),
                            ));
                            target.push(quote_str("\""));
                            target.push(group);
                            target.push(quote_str("\""));
                        }
                    }
                }
                target.push(quote_str(">"));
                for child in el.children {
                    child.render_into(target);
                }
                target.push(quote_str(&format!("</{}>", el.tag)));
            }
        }
    }
}

fn quote_str(s: &str) -> TokenTree {
    TokenTree::Literal(Literal::string(s))
}
