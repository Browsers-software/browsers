pub fn ellipsize(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        return text.to_string();
    }
    text[0..max_length - 1].to_string() + "…"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipsize_shorter() {
        assert_eq!(ellipsize("some text", 8), "some te…");
    }

    #[test]
    fn test_ellipsize_enough() {
        assert_eq!(ellipsize("some text", 9), "some text");
    }
}
