use crate::document::Element;
use proc_macro::TokenTree;

use super::{children::ChildError, Parser};

pub enum CloseTagError {
    UnexpectedToken,
}

#[derive(Debug)]
pub enum ElementError {
    UnexpectedToken,
}

impl Parser {
    // Node::Element or Node::Fragment
    pub fn try_element(&mut self) -> Option<Element> {
        match self.stream.peek() {
            Some(TokenTree::Punct(p)) if p.as_char() == '<' => {
                self.stream.next();
                if let Some(tag_name) = self.try_tag_name() {
                    let mut el = Element::new(tag_name);
                    loop {
                        match self.try_attribute() {
                            Ok(Some(attr)) => {
                                el.attributes.push(attr);
                            }
                            Ok(None) => break,
                            Err(_) => panic!("Unexpected token"),
                        }
                    }

                    self.stack.push(el);
                    Some(self.end_current_element().expect("could not end element"))
                } else {
                    if let Some(TokenTree::Punct(p)) = self.stream.next() {
                        if p.as_char() == '>' {
                            self.stream.next();
                            self.stack.push(Element::fragment());
                            self.end_fragment();
                            return Some(self.stack.pop().expect("inconsistent state for stack"));
                        }
                        panic!("Unexpected token");
                    }
                    None
                }
            }
            _ => None,
        }
    }

    pub fn try_tag_name(&mut self) -> Option<String> {
        match self.stream.peek() {
            Some(TokenTree::Ident(ident)) => {
                let tag_name = ident.to_string();
                self.stream.next();
                Some(tag_name)
            }
            _ => None,
        }
    }

    pub fn try_close_tag(&mut self) -> Result<String, CloseTagError> {
        match self.stream.next() {
            Some(TokenTree::Ident(ident)) => {
                let tag_name = ident.to_string();
                if let Some(TokenTree::Punct(p)) = self.stream.next() {
                    if p.as_char() == '>' {
                        Ok(tag_name)
                    } else {
                        Err(CloseTagError::UnexpectedToken)
                    }
                } else {
                    Err(CloseTagError::UnexpectedToken)
                }
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '>' => {
                // end of fragment
                Ok(String::new())
            }
            _ => Err(CloseTagError::UnexpectedToken),
        }
    }

    pub fn end_current_element(&mut self) -> Result<Element, ElementError> {
        match self.stream.next() {
            Some(TokenTree::Punct(p)) if p.as_char() == '>' => loop {
                match self.try_child() {
                    Ok(child) => {
                        self.stack
                            .last_mut()
                            .expect("inconsistent state on stack")
                            .children
                            .push(child);
                        continue;
                    }
                    Err(ChildError::ClosingTag(tag_name)) => {
                        let el = self.stack.pop().expect("inconsistent state on stack");
                        if el.tag != tag_name {
                            panic!("Mismatched closing tag");
                        }
                        return Ok(el);
                    }
                    Err(ChildError::UnexpectedToken) => panic!("Unexpected token"),
                }
            },
            Some(TokenTree::Punct(p)) if p.as_char() == '/' => match self.stream.next() {
                Some(TokenTree::Punct(p)) if p.as_char() == '>' => {
                    self.stream.next();
                    let el = self
                        .stack
                        .pop()
                        .expect("inconsistent state on stack: no current element");
                    return Ok(el);
                }
                _ => panic!("Unexpected token: expected '>'"),
            },
            _ => panic!("Unexpected token"),
        }
    }

    pub fn end_fragment(&mut self) {
        loop {
            match self.try_child() {
                Ok(child) => {
                    self.stack
                        .last_mut()
                        .expect("inconsistent state on stack")
                        .children
                        .push(child);
                }
                Err(ChildError::ClosingTag(tag_name)) => {
                    let el = self.stack.pop().expect("inconsistent state on stack");
                    if el.tag != tag_name {
                        panic!("Mismatched closing tag");
                    }
                    break;
                }
                Err(ChildError::UnexpectedToken) => panic!("Unexpected token"),
            }
        }
    }
}
