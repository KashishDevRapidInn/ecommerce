// An extension trait to provide the `graphemes` method on `String` and `&str`
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct UserName(String);

impl UserName {
    pub fn parse(s: String) -> std::result::Result<UserName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|c| forbidden_characters.contains(&c));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            Err(format!("{} is not a valid subscriber name.", s))
        } else {
            Ok(Self(s))
        }
    }
}
impl AsRef<str> for UserName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

use regex::Regex;

#[derive(Debug)]
pub struct UserEmail(String);

impl UserEmail {
    pub fn parse(s: String) -> std::result::Result<UserEmail, String> {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if email_regex.is_match(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid email address.", s))
        }
    }
}

impl AsRef<str> for UserEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
