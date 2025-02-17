use super::ParseError;
use serde::{Deserialize, Serialize};
use std::fmt;
use validator::ValidateEmail;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriberEmail<'a>(&'a str);

impl<'a> AsRef<str> for SubscriberEmail<'a> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'a> fmt::Display for SubscriberEmail<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'a> SubscriberEmail<'a> {
    pub fn parse(name: &str) -> Result<SubscriberEmail, ParseError> {
        if name.validate_email() {
            Ok(SubscriberEmail(name))
        } else {
            return Err(ParseError::BadEmail);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SubscriberEmail;
    use claim::{assert_err, assert_ok};

    #[test]
    fn valid_email_parsed_successfully() {
        let email = "luka_tim@gmail.com";
        assert_ok!(SubscriberEmail::parse(&email));
    }

    #[test]
    fn invalid_email_rejected() {
        let email = "luka_timgmail.com";
        assert_err!(SubscriberEmail::parse(&email));
    }

    #[test]
    fn a_email_longer_than_256_is_rejected() {
        let email = "a".repeat(257);
        assert_err!(SubscriberEmail::parse(&email));
    }

    #[test]
    fn only_whitespace_email_rejected() {
        let email = " ";
        assert_err!(SubscriberEmail::parse(email));
    }
}
