use crate::document::{Attribute, AttributeValue, Node};
use proc_macro::{Delimiter, Group, Literal, TokenStream, TokenTree};
use quote::{quote, ToTokens as _};

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
                    match attr.value {
                        AttributeValue::Empty => {
                            target.push(quote_str(&format!(" {}", attr.name)));
                        }
                        AttributeValue::Literal(value) => {
                            // TODO perf: buffer?
                            target.push(quote_str(&format!(
                                " {}=\"{}\"",
                                attr.name,
                                html_escape::encode_double_quoted_attribute(&value)
                            )));
                        }
                        AttributeValue::Expression(value) => {
                            target.push(quote_str(&format!(" {}=", attr.name)));
                            let value: TokenStream = TokenTree::Group(Group::new(
                                proc_macro::Delimiter::Brace,
                                value.into_iter().collect(),
                            ))
                            .into();
                            let value: syn::Expr =
                                syn::parse(value).expect("could not parse expression");
                            // TODO pref: skip to_string for string values...
                            let stream: proc_macro::TokenStream =
                                quote!(&htmi::utils::escape_attribute(&#value.to_string())).into();
                            let group = TokenTree::Group(Group::new(
                                proc_macro::Delimiter::None,
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
