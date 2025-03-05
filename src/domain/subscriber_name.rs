use std::fmt;

use super::ParseError;

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SubscriberName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Default, Clone)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(name: &str) -> Result<SubscriberName, ParseError> {
        let is_too_long = name.len() > 256;

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contais_forbidden_chars = name.chars().any(|c| forbidden_characters.contains(&c));

        let is_empty_or_whitespace = name.trim().is_empty();
        if is_empty_or_whitespace
            || is_too_long
            || is_empty_or_whitespace
            || contais_forbidden_chars
        {
            tracing::error!("Name validation failed for string `{}`", name);
            return Err(ParseError::BadName);
        }

        tracing::info!("Successfully parsed name: `{}`", name);
        Ok(SubscriberName(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberName;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_long_name_is_valid() {
        let name = "a".repeat(256);
        assert_ok!(SubscriberName::parse(&name));
    }

    #[test]
    fn a_name_longer_than_256_is_rejected() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(&name));
    }

    #[test]
    fn only_whitespace_name_rejected() {
        let name = " ";
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn valid_name_parsed_successfully() {
        assert_ok!(SubscriberName::parse("Luka Tim"));
    }
}
