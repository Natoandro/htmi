use crate::document::{Attribute, Element, Node};
use proc_macro::{TokenStream, TokenTree};
use std::iter::Peekable;

pub struct Parser {
    stack: Vec<Element>,
    stream: Peekable<proc_macro::token_stream::IntoIter>,
}

#[derive(Debug)]
enum ElementError {
    UnexpectedToken,
}

struct AttributeError;

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

    // Node::Element or Node::Fragment
    fn try_element(&mut self) -> Option<Element> {
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

    fn try_tag_name(&mut self) -> Option<String> {
        match self.stream.peek() {
            Some(TokenTree::Ident(ident)) => {
                let tag_name = ident.to_string();
                self.stream.next();
                Some(tag_name)
            }
            _ => None,
        }
    }

    fn try_attribute(&mut self) -> Result<Option<Attribute>, AttributeError> {
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

    fn end_current_element(&mut self) -> Result<Element, ElementError> {
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

    fn end_fragment(&mut self) {
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

    fn try_child(&mut self) -> Result<Node, ChildError> {
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

    fn try_close_tag(&mut self) -> Result<String, CloseTagError> {
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
}

enum CloseTagError {
    UnexpectedToken,
}

enum ChildError {
    UnexpectedToken,
    ClosingTag(String),
}
