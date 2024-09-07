pub fn escape_attribute(value: &str) -> String {
    html_escape::encode_double_quoted_attribute(value).to_string()
}
