use proc_macro::TokenTree;

use crate::document::Node;

use super::Parser;

pub enum ChildError {
    UnexpectedToken,
    ClosingTag(String),
}

impl Parser {
    pub fn try_child(&mut self) -> Result<Node, ChildError> {
        if let Some(text) = self.try_text() {
            Ok(Node::Text(text))
        } else {
            match self.stream.next() {
                // next element or closing tag
                Some(TokenTree::Punct(p)) if p.as_char() == '<' => match self.stream.peek() {
                    Some(TokenTree::Punct(p)) if p.as_char() == '/' => {
                        self.stream.next();
                        if let Ok(tag_name) = self.try_close_tag() {
                            Err(ChildError::ClosingTag(tag_name))
                        } else {
                            panic!("Unexpected token");
                        }
                    }
                    _ => {
                        if let Some(el) = self.try_element() {
                            Ok(Node::Element(el))
                        } else {
                            panic!("Unexpected token");
                        }
                    }
                },
                _ => panic!("Unexpected token"),
            }
        }
    }
}
