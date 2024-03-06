use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberUsername(String);

impl SubscriberUsername {
    pub fn parse(s: String) -> Result<Self, String> {
        // Do not allow empty or whitespace username
        let is_empty_or_whitespace = s.trim().is_empty();

        // Do not allow username whose length exceed 256 chars
        let exceed_allowed_length = s.graphemes(true).count() > 256;

        // Do not allow special chars
        let special_chars = ['(', ')', '/', '"', '<', '>', '{', '}', '\\'];

        let contains_special_chars = s.chars().any(|c| special_chars.contains(&c));

        if !(is_empty_or_whitespace || exceed_allowed_length || contains_special_chars) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid subscriber name.", s))
        }
    }
}

impl AsRef<str> for SubscriberUsername {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};

    use crate::domain::subscriber_username::SubscriberUsername;
    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a".repeat(256);
        assert_ok!(SubscriberUsername::parse(name));
    }
    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(SubscriberUsername::parse(name));
    }
    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert_err!(SubscriberUsername::parse(name));
    }
    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberUsername::parse(name));
    }
    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberUsername::parse(name));
        }
    }
    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        assert_ok!(SubscriberUsername::parse(name));
    }
}
