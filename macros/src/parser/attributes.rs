use super::Parser;
use crate::document::{Attribute, AttributeValue};
use proc_macro::{Delimiter, TokenStream, TokenTree};

pub struct AttributeError;

impl Parser {
    pub fn try_attribute(&mut self) -> Result<Option<Attribute>, AttributeError> {
        match self.stream.peek() {
            Some(TokenTree::Ident(ident)) => {
                let attr_name = ident.to_string();
                self.stream.next();
                match self.stream.next() {
                    Some(TokenTree::Punct(p)) if p.as_char() == '=' => {
                        match self.try_attribute_value() {
                            Ok(value) => Ok(Some(Attribute::new(attr_name, value))),
                            Err(_) => Err(AttributeError),
                        }
                    }
                    Some(TokenTree::Punct(p)) if p.as_char() == '>' || p.as_char() == '/' => {
                        // TODO: PERF hint next element
                        Ok(Some(Attribute::empty(attr_name)))
                    }
                    Some(TokenTree::Ident(_)) => {
                        // TODO: PERF hint next attribute
                        Ok(Some(Attribute::empty(attr_name)))
                    }
                    _ => Err(AttributeError),
                }
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '>' || p.as_char() == '/' => Ok(None),
            _ => Err(AttributeError),
        }
    }

    fn try_attribute_value(&mut self) -> Result<AttributeValue, AttributeError> {
        match self.stream.peek() {
            Some(TokenTree::Ident(_)) => {
                let ident = self.stream.next().unwrap();
                Ok(AttributeValue::Expression(TokenStream::from(ident)))
            }
            Some(TokenTree::Literal(_)) => {
                let lit: syn::Lit = syn::parse(self.stream.next().unwrap().into()).unwrap();
                let value = match lit {
                    syn::Lit::Str(s) => s.value(),
                    syn::Lit::Char(c) => [c.value()].iter().collect(),
                    syn::Lit::Int(i) => i.base10_digits().to_string(),
                    syn::Lit::Float(f) => f.to_string(),
                    syn::Lit::Bool(b) => b.value().to_string(),
                    _ => return Err(AttributeError),
                };

                Ok(AttributeValue::Literal(value))
            }
            Some(TokenTree::Group(group))
                if matches!(group.delimiter(), Delimiter::Parenthesis | Delimiter::Brace) =>
            {
                let value = group.stream();
                self.stream.next().unwrap();
                Ok(AttributeValue::Expression(value))
            }
            _ => Err(AttributeError),
        }
    }
}
