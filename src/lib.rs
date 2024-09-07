pub use htmi_macros::render;

pub mod utils;

#[cfg(test)]
mod tests {
    use crate as htmi;

    #[test]
    fn simple() {
        assert_eq!(
            &htmi::render!(<div hello="world"></div>),
            r#"<div hello="world"></div>"#
        );

        // non-string
        assert_eq!(
            &htmi::render!(<div hello=12></div>),
            r#"<div hello="12"></div>"#
        );
    }

    #[test]
    fn expression_attribute() {
        let world = "world";
        assert_eq!(
            htmi::render!(<div hello=world></div>),
            r#"<div hello="world"></div>"#
        );
        assert_eq!(
            htmi::render!(<div hello=(world)></div>),
            r#"<div hello="world"></div>"#
        );
        assert_eq!(
            htmi::render!(<div hello={world}></div>),
            r#"<div hello="world"></div>"#
        );
        assert_eq!(
            htmi::render!(<div hello=(1+2)></div>),
            r#"<div hello="3"></div>"#
        );
        assert_eq!(
            htmi::render!(<div hello={1+2}></div>),
            r#"<div hello="3"></div>"#
        );
        assert_eq!(
            htmi::render!(<div hello={let a = 12; a + 1}></div>),
            r#"<div hello="13"></div>"#
        );
    }

    // TODO html escape
}
