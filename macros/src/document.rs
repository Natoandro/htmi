use proc_macro::TokenStream;

#[derive(Debug)]
pub enum Node {
    Text(String), // TODO support expressions
    Element(Element),
}

#[derive(Debug)]
pub enum Attribute {
    Empty(String),
    Literal(String, String),
    Expression(String, TokenStream),
}

#[derive(Debug)]
pub struct Element {
    pub tag: String, // empty for fragments
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
}

impl Element {
    pub fn new(tag: String) -> Self {
        Self {
            tag,
            attributes: vec![],
            children: vec![],
        }
    }

    pub fn fragment() -> Self {
        Self::new(String::new())
    }
}
