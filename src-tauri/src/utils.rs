pub fn remove_whitespace(input: &str) -> String {
    input.chars().filter(|c| !c.is_whitespace()).collect()
}
