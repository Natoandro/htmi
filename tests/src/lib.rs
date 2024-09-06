#[cfg(test)]
mod tests {
    #[test]
    fn simple() {
        let a = htmi::render!(<div hello="world"></div>);
        assert_eq!(&a, r#"<div hello="world"></div>"#);
        eprintln!("{}", a);
    }

    #[test]
    fn expression_attribute() {
        let world = "world";
        let a = htmi::render!(<div hello=(world)></div>);
        assert_eq!(a, r#"<div hello="world"></div>"#);
        eprintln!("{}", a);
    }

    // TODO html escape
}
