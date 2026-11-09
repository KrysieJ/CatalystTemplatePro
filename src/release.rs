//! Software release numbers, such as IOS XE 26.1.2.

use std::fmt;

/// A Cisco software release number.
///
/// Data definition: three whole numbers (major, minor, rebuild) and an
/// optional lowercase letter that Cisco sometimes appends to the rebuild.
///
/// Examples of data:
/// - IOS XE 26.1.2 is `Release::new(26, 1, 2)`.
/// - IOS XE 17.13.1a is `Release::parse("17.13.1a")`.
///
/// Releases compare by number, not as text: 26.1.1 comes before 26.1.2,
/// which comes before 26.2.1, and 17.9.1 comes before 17.18.1. A lettered
/// rebuild (17.13.1a) comes after the plain one (17.13.1). That last rule
/// follows the usual Cisco respin convention and is [Unverified].
///
/// The fields are private so that a release can only be made through
/// `new` or `parse`, which keeps every release well formed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Release {
    // Field order matters: the derived ordering compares major first,
    // then minor, then rebuild, then the letter.
    major: u16,
    minor: u16,
    rebuild: u16,
    suffix: Option<char>,
}

/// Why a piece of text could not be read as a release number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseError {
    /// The text does not have exactly three parts separated by dots.
    WrongShape,
    /// A part is empty, is not a whole number, or is larger than 65535.
    BadNumber,
}

impl Release {
    /// Purpose: build a release from its three numbers, with no letter.
    pub fn new(major: u16, minor: u16, rebuild: u16) -> Release {
        Release {
            major,
            minor,
            rebuild,
            suffix: None,
        }
    }

    /// Purpose: read a release from text such as "26.1.2" or "17.13.1a".
    ///
    /// Spaces around the text are ignored. Anything that is not exactly
    /// three dot-separated whole numbers, with an optional lowercase letter
    /// on the last one, is refused with the reason.
    pub fn parse(text: &str) -> Result<Release, ReleaseError> {
        let parts: Vec<&str> = text.trim().split('.').collect();
        if parts.len() != 3 {
            return Err(ReleaseError::WrongShape);
        }
        let major = parse_number(parts[0])?;
        let minor = parse_number(parts[1])?;
        let (rebuild_text, suffix) = split_suffix(parts[2]);
        let rebuild = parse_number(rebuild_text)?;
        Ok(Release {
            major,
            minor,
            rebuild,
            suffix,
        })
    }
}

/// Purpose: read one part of a release as a whole number.
fn parse_number(text: &str) -> Result<u16, ReleaseError> {
    if text.is_empty() || !text.chars().all(|c| c.is_ascii_digit()) {
        return Err(ReleaseError::BadNumber);
    }
    text.parse::<u16>().map_err(|_| ReleaseError::BadNumber)
}

/// Purpose: separate a trailing lowercase letter from the rebuild part.
/// "1a" becomes ("1", Some('a')). "1" becomes ("1", None).
fn split_suffix(text: &str) -> (&str, Option<char>) {
    match text.chars().next_back() {
        // A lowercase ASCII letter is one byte long, so cutting one byte
        // off the end is safe here.
        Some(letter) if letter.is_ascii_lowercase() => (&text[..text.len() - 1], Some(letter)),
        _ => (text, None),
    }
}

impl fmt::Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.rebuild)?;
        if let Some(letter) = self.suffix {
            write!(f, "{letter}")?;
        }
        Ok(())
    }
}

impl fmt::Display for ReleaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReleaseError::WrongShape => write!(f, "a release must look like 26.1.2"),
            ReleaseError::BadNumber => {
                write!(f, "each part of a release must be a whole number")
            }
        }
    }
}

impl std::error::Error for ReleaseError {}

#[cfg(test)]
mod tests {
    use super::{Release, ReleaseError};

    #[test]
    fn parses_a_plain_release() {
        assert_eq!(Release::parse("26.1.2"), Ok(Release::new(26, 1, 2)));
    }

    #[test]
    fn ignores_surrounding_spaces() {
        assert_eq!(Release::parse("  26.2.1 "), Ok(Release::new(26, 2, 1)));
    }

    #[test]
    fn parses_a_lettered_release_and_prints_it_back() {
        let release = Release::parse("17.13.1a").expect("17.13.1a is a valid release");
        assert_eq!(release.to_string(), "17.13.1a");
    }

    #[test]
    fn prints_a_plain_release() {
        assert_eq!(Release::new(26, 1, 1).to_string(), "26.1.1");
    }

    #[test]
    fn orders_releases_the_way_cisco_numbers_them() {
        assert!(Release::new(26, 1, 1) < Release::new(26, 1, 2));
        assert!(Release::new(26, 1, 2) < Release::new(26, 2, 1));
        assert!(Release::new(17, 18, 1) < Release::new(26, 1, 1));
        let lettered = Release::parse("17.13.1a").expect("17.13.1a is a valid release");
        assert!(Release::new(17, 13, 1) < lettered);
    }

    #[test]
    fn compares_numbers_not_text() {
        // As text, "17.9.1" sorts after "17.18.1". As releases it comes first.
        assert!(Release::new(17, 9, 1) < Release::new(17, 18, 1));
    }

    #[test]
    fn refuses_the_wrong_number_of_parts() {
        assert_eq!(Release::parse("26.1"), Err(ReleaseError::WrongShape));
        assert_eq!(Release::parse("26.1.2.3"), Err(ReleaseError::WrongShape));
        assert_eq!(Release::parse(""), Err(ReleaseError::WrongShape));
    }

    #[test]
    fn refuses_parts_that_are_not_numbers() {
        assert_eq!(Release::parse("26.x.1"), Err(ReleaseError::BadNumber));
        assert_eq!(Release::parse("26..1"), Err(ReleaseError::BadNumber));
        assert_eq!(Release::parse("-1.1.1"), Err(ReleaseError::BadNumber));
        assert_eq!(Release::parse("70000.1.1"), Err(ReleaseError::BadNumber));
    }

    #[test]
    fn refuses_bad_letters() {
        // Uppercase is not a Cisco suffix.
        assert_eq!(Release::parse("26.1.A"), Err(ReleaseError::BadNumber));
        // A letter with no number in front of it.
        assert_eq!(Release::parse("26.1.a"), Err(ReleaseError::BadNumber));
        // Two letters.
        assert_eq!(Release::parse("26.1.ab"), Err(ReleaseError::BadNumber));
    }
}
