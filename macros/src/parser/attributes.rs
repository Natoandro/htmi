use proc_macro::TokenTree;

use super::Parser;
use crate::document::Attribute;

pub struct AttributeError;

impl Parser {
    pub fn try_attribute(&mut self) -> Result<Option<Attribute>, AttributeError> {
        match self.stream.peek() {
            Some(TokenTree::Ident(ident)) => {
                let attr_name = ident.to_string();
                self.stream.next();
                match self.stream.next() {
                    Some(TokenTree::Punct(p)) if p.as_char() == '=' => match self.stream.peek() {
                        Some(TokenTree::Literal(lit)) => {
                            let attr_value = lit.to_string();
                            self.stream.next();
                            Ok(Some(Attribute::Literal(attr_name, attr_value)))
                        }
                        Some(TokenTree::Group(_)) => match self.stream.next() {
                            Some(TokenTree::Group(group)) => {
                                let attr_value = group.stream();
                                Ok(Some(Attribute::Expression(attr_name, attr_value)))
                            }
                            _ => unreachable!(),
                        },
                        _ => Err(AttributeError),
                    },
                    Some(TokenTree::Punct(p)) if p.as_char() == '>' || p.as_char() == '/' => {
                        // TODO: PERF hint next element
                        Ok(Some(Attribute::Empty(attr_name)))
                    }
                    Some(TokenTree::Ident(_)) => {
                        // TODO: PERF hint next attribute
                        Ok(Some(Attribute::Empty(attr_name)))
                    }
                    _ => Err(AttributeError),
                }
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '>' || p.as_char() == '/' => Ok(None),
            _ => Err(AttributeError),
        }
    }
}
