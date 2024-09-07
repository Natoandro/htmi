use crate::document::{Element, Node};
use proc_macro::{TokenStream, TokenTree};
use std::iter::Peekable;

mod attributes;
mod children;
mod element;

pub struct Parser {
    stack: Vec<Element>,
    stream: Peekable<proc_macro::token_stream::IntoIter>,
}

#[derive(Debug)]
enum ElementError {
    UnexpectedToken,
}

impl Parser {
    pub fn parse(ts: TokenStream) -> Node {
        let mut parser = Parser {
            stack: vec![],
            stream: ts.into_iter().peekable(),
        };

        parser.start()
    }

    // TODO return Result; no panic
    fn start(&mut self) -> Node {
        if let Some(text) = self.try_text() {
            Node::Text(text)
        } else {
            if let Some(el) = self.try_element() {
                Node::Element(el)
            } else {
                panic!("Unexpected token");
            }
        }
    }

    fn try_text(&mut self) -> Option<String> {
        match self.stream.peek() {
            Some(TokenTree::Literal(_)) => {
                let lit = self.stream.next().unwrap();
                match lit {
                    TokenTree::Literal(lit) => Some(lit.to_string()),
                    _ => unreachable!(),
                }
            }
            _ => None,
        }
    }
}
