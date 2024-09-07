use proc_macro::{Literal, TokenStream, TokenTree};
use quote::quote;
use std::iter::Peekable;
use syn::{parse, Expr};

#[derive(Debug)]
enum Node {
    Text(String), // TODO support expressions
    Element(Element),
}

impl Node {
    fn quote_in(self, target: &mut Vec<TokenTree>) {
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
                            let value: Expr = parse(value).expect("could not parse expression");
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
                    child.quote_in(target);
                }
                target.push(quote_str(&format!("</{}>", el.tag)));
            }
        }
    }
}

fn quote_str(s: &str) -> TokenTree {
    TokenTree::Literal(Literal::string(s))
}

#[derive(Debug)]
enum Attribute {
    Empty(String),
    Literal(String, String),
    Expression(String, TokenStream),
}

#[derive(Debug)]
struct Element {
    tag: String, // empty for fragments
    attributes: Vec<Attribute>,
    children: Vec<Node>,
}

impl Element {
    fn new(tag: String) -> Self {
        Self {
            tag,
            attributes: vec![],
            children: vec![],
        }
    }

    fn fragment() -> Self {
        Self::new(String::new())
    }
}

#[proc_macro]
pub fn render(input: TokenStream) -> TokenStream {
    let root = Parser::parse(input);
    let mut trees = vec![];
    root.quote_in(&mut trees);
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

enum Target {
    Empty,
    NewElement,
    Attributes,
}

#[derive(Debug)]
enum ElementError {
    UnexpectedToken,
}

struct Parser {
    stack: Vec<Element>,
    stream: Peekable<proc_macro::token_stream::IntoIter>,
}

struct AttributeError;

impl Parser {
    fn parse(ts: TokenStream) -> Node {
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
