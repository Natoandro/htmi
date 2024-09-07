use proc_macro::TokenStream;

#[derive(Debug)]
pub enum Node {
    Text(String), // TODO support expressions
    Element(Element),
}

#[derive(Debug)]
pub enum AttributeValue {
    Empty,
    Literal(String),
    Expression(TokenStream),
}

#[derive(Debug)]
pub struct Attribute {
    pub name: String,
    pub value: AttributeValue,
}

impl Attribute {
    pub fn new(name: String, value: AttributeValue) -> Self {
        Self { name, value }
    }

    pub fn empty(name: String) -> Self {
        Self {
            name,
            value: AttributeValue::Empty,
        }
    }
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
