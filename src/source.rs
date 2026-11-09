//! Where a fact came from, so every claim the tool makes can be checked.

use std::fmt;

use crate::release::Release;

/// A calendar date, used to record when a source was read.
///
/// Data definition: a year, a month from 1 to 12, and a day that exists in
/// that month. Kept as plain numbers so the crate needs no date library.
///
/// Example of data: 8 November 2026 is `Date::new(2026, 11, 8)`.
///
/// The fields are private so that a date can only be made through `new`,
/// which refuses dates that do not exist (such as 31 April).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

impl Date {
    /// Purpose: build a date, or give back `None` if the month or day does
    /// not exist.
    pub fn new(year: u16, month: u8, day: u8) -> Option<Date> {
        if !(1..=12).contains(&month) {
            return None;
        }
        if day == 0 || day > days_in_month(year, month) {
            return None;
        }
        Some(Date { year, month, day })
    }
}

/// Purpose: how many days a month has in a given year.
/// Only called with months 1 to 12.
fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Purpose: whether a year has 29 February (Gregorian calendar rule).
fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

impl fmt::Display for Date {
    /// Prints the date as YYYY-MM-DD.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// The kind of document or statement a fact came from.
///
/// Which kind answers which question is set in docs/DESIGN-DECISIONS.md, D6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceKind {
    /// A Cisco command reference. Authority for command syntax.
    CommandReference,
    /// A Cisco configuration guide.
    ConfigurationGuide,
    /// Cisco release notes.
    ReleaseNotes,
    /// A Cisco data sheet. Often the place that ties features to licenses.
    DataSheet,
    /// A Cisco YANG model. Authority for configuration structure only.
    YangModel,
    /// The CATC-TEMPLATE-RAG repository. Authority for template rules.
    TemplateRag,
    /// A statement from an engineer with hands-on experience. Treated as a
    /// source of truth until that engineer withdraws it.
    EngineerStatement,
}

/// One source for one fact.
///
/// Example of data: the Catalyst 9300 command reference for IOS XE 26,
/// read on 8 November 2026, is a `Source` with kind `CommandReference`,
/// title "Command Reference, Cisco IOS XE 26.x.x (Catalyst 9300 Switches)",
/// no single release, and that retrieval date.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// What kind of document or statement this is.
    pub kind: SourceKind,
    /// The title, exactly as the publisher gives it. For an engineer's
    /// statement, a short summary of what was stated.
    pub title: String,
    /// The software release the source covers, if it covers only one.
    pub release: Option<Release>,
    /// Where in the source to look: a section, a chapter, or a web address.
    pub location: Option<String>,
    /// The date the source was read.
    pub retrieved: Date,
}

impl Source {
    /// Purpose: one line describing the source, for findings and reports.
    pub fn describe(&self) -> String {
        let mut text = self.title.clone();
        if let Some(release) = &self.release {
            text.push_str(&format!(", release {release}"));
        }
        if let Some(location) = &self.location {
            text.push_str(&format!(", {location}"));
        }
        text.push_str(&format!(", retrieved {}", self.retrieved));
        text
    }
}

#[cfg(test)]
mod tests {
    use super::{Date, Source, SourceKind};
    use crate::release::Release;

    #[test]
    fn accepts_a_real_date_and_prints_it() {
        let date = Date::new(2026, 11, 8).expect("8 November 2026 exists");
        assert_eq!(date.to_string(), "2026-11-08");
    }

    #[test]
    fn refuses_months_that_do_not_exist() {
        assert_eq!(Date::new(2026, 0, 1), None);
        assert_eq!(Date::new(2026, 13, 1), None);
    }

    #[test]
    fn refuses_days_that_do_not_exist() {
        assert_eq!(Date::new(2026, 1, 0), None);
        assert_eq!(Date::new(2026, 4, 31), None);
        assert_eq!(Date::new(2026, 1, 32), None);
    }

    #[test]
    fn knows_about_leap_years() {
        assert_eq!(Date::new(2026, 2, 29), None);
        assert!(Date::new(2028, 2, 29).is_some());
        // Century years are leap years only when divisible by 400.
        assert_eq!(Date::new(1900, 2, 29), None);
        assert!(Date::new(2000, 2, 29).is_some());
    }

    #[test]
    fn orders_dates_by_calendar() {
        let earlier = Date::new(2026, 4, 10).expect("valid date");
        let later = Date::new(2026, 11, 8).expect("valid date");
        assert!(earlier < later);
    }

    #[test]
    fn describes_a_source_with_only_a_title_and_date() {
        let source = Source {
            kind: SourceKind::CommandReference,
            title: "Command Reference, Cisco IOS XE 26.x.x (Catalyst 9300 Switches)".to_string(),
            release: None,
            location: None,
            retrieved: Date::new(2026, 11, 8).expect("valid date"),
        };
        assert_eq!(
            source.describe(),
            "Command Reference, Cisco IOS XE 26.x.x (Catalyst 9300 Switches), retrieved 2026-11-08"
        );
    }

    #[test]
    fn describes_a_source_with_every_detail() {
        // Synthetic fixture: the section name is made up for the test.
        let source = Source {
            kind: SourceKind::ConfigurationGuide,
            title: "Example Configuration Guide".to_string(),
            release: Some(Release::new(26, 1, 2)),
            location: Some("Chapter: Example".to_string()),
            retrieved: Date::new(2026, 11, 8).expect("valid date"),
        };
        assert_eq!(
            source.describe(),
            "Example Configuration Guide, release 26.1.2, Chapter: Example, retrieved 2026-11-08"
        );
    }
}
