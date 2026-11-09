//! Where a fact came from, so every claim the tool makes can be checked.

use std::collections::BTreeMap;
use std::fmt;

use serde::Deserialize;

use crate::data::DataError;
use crate::release::Release;

/// A calendar date, used to record when a source was read.
///
/// Data definition: a year, a month from 1 to 12, and a day that exists in
/// that month. Kept as plain numbers so the crate needs no date library.
///
/// Example of data: 8 November 2026 is `Date::new(2026, 11, 8)`.
///
/// The fields are private so that a date can only be made through `new`
/// or `parse`, which refuse dates that do not exist (such as 31 April).
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

    /// Purpose: read a date written as YYYY-MM-DD, such as "2026-11-08".
    /// Gives back `None` for any other shape or for a date that does not
    /// exist.
    pub fn parse(text: &str) -> Option<Date> {
        let parts: Vec<&str> = text.trim().split('-').collect();
        if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
            return None;
        }
        if !parts
            .iter()
            .all(|part| part.chars().all(|c| c.is_ascii_digit()))
        {
            return None;
        }
        let year = parts[0].parse::<u16>().ok()?;
        let month = parts[1].parse::<u8>().ok()?;
        let day = parts[2].parse::<u8>().ok()?;
        Date::new(year, month, day)
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
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
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
/// In data files each kind is written in lowercase with hyphens, such as
/// `kind = "command-reference"` or `kind = "engineer-statement"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
/// its title as Cisco gives it, no single release, and that retrieval date.
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

/// A `[[source]]` entry exactly as written in a data file, before checking.
///
/// Example in a file:
/// ```toml
/// [[source]]
/// id = "cisco-iosxe26-release-notes"
/// kind = "release-notes"
/// title = "Cisco IOS XE 26 release notes list"
/// location = "https://www.cisco.com/..."
/// retrieved = "2026-11-08"
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RawSource {
    pub(crate) id: String,
    pub(crate) kind: SourceKind,
    pub(crate) title: String,
    #[serde(default)]
    pub(crate) release: Option<String>,
    #[serde(default)]
    pub(crate) location: Option<String>,
    pub(crate) retrieved: String,
}

/// The sources listed in one data file, looked up by their id.
///
/// Entries elsewhere in the file name a source by id instead of repeating
/// it, so one document can back many claims.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SourceTable {
    by_id: BTreeMap<String, Source>,
}

impl SourceTable {
    /// Purpose: check raw `[[source]]` entries and build the table, or
    /// refuse with the first problem found.
    pub(crate) fn from_raw(raw: Vec<RawSource>) -> Result<SourceTable, DataError> {
        let mut by_id = BTreeMap::new();
        for entry in raw {
            if entry.id.trim().is_empty() {
                return Err(DataError::BadValue {
                    what: "source",
                    id: entry.id.clone(),
                    field: "id",
                    value: entry.id,
                });
            }
            if entry.title.trim().is_empty() {
                return Err(DataError::BadValue {
                    what: "source",
                    id: entry.id,
                    field: "title",
                    value: entry.title,
                });
            }
            if by_id.contains_key(&entry.id) {
                return Err(DataError::Duplicate {
                    what: "source",
                    id: entry.id,
                });
            }
            let retrieved = match Date::parse(&entry.retrieved) {
                Some(date) => date,
                None => {
                    return Err(DataError::BadValue {
                        what: "source",
                        id: entry.id,
                        field: "retrieved",
                        value: entry.retrieved,
                    })
                }
            };
            let release = match entry.release {
                Some(text) => match Release::parse(&text) {
                    Ok(release) => Some(release),
                    Err(_) => {
                        return Err(DataError::BadValue {
                            what: "source",
                            id: entry.id,
                            field: "release",
                            value: text,
                        })
                    }
                },
                None => None,
            };
            let source = Source {
                kind: entry.kind,
                title: entry.title,
                release,
                location: entry.location,
                retrieved,
            };
            by_id.insert(entry.id, source);
        }
        Ok(SourceTable { by_id })
    }

    /// Purpose: find a source by its id.
    pub fn get(&self, id: &str) -> Option<&Source> {
        self.by_id.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::{Date, RawSource, Source, SourceKind, SourceTable};
    use crate::data::DataError;
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
    fn reads_a_written_date() {
        assert_eq!(Date::parse("2026-11-08"), Date::new(2026, 11, 8));
        assert_eq!(Date::parse(" 2026-11-08 "), Date::new(2026, 11, 8));
    }

    #[test]
    fn refuses_written_dates_in_other_shapes() {
        assert_eq!(Date::parse("2026-11-8"), None);
        assert_eq!(Date::parse("11/08/2026"), None);
        assert_eq!(Date::parse("2026-11"), None);
        assert_eq!(Date::parse("2026-+1-08"), None);
        assert_eq!(Date::parse("2026-02-30"), None);
        assert_eq!(Date::parse(""), None);
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

    /// A synthetic raw source for tests. It is not a real document.
    fn raw(id: &str, retrieved: &str, release: Option<&str>) -> RawSource {
        RawSource {
            id: id.to_string(),
            kind: SourceKind::ReleaseNotes,
            title: "Example release notes (synthetic test fixture)".to_string(),
            release: release.map(str::to_string),
            location: None,
            retrieved: retrieved.to_string(),
        }
    }

    #[test]
    fn source_table_finds_entries_by_id() {
        let table = SourceTable::from_raw(vec![raw("a", "2026-11-08", Some("26.1.2"))])
            .expect("a valid source");
        let found = table.get("a").expect("source a is in the table");
        assert_eq!(found.release, Some(Release::new(26, 1, 2)));
        assert!(table.get("b").is_none());
    }

    #[test]
    fn source_table_refuses_duplicate_ids() {
        let result = SourceTable::from_raw(vec![
            raw("a", "2026-11-08", None),
            raw("a", "2026-11-08", None),
        ]);
        assert_eq!(
            result,
            Err(DataError::Duplicate {
                what: "source",
                id: "a".to_string()
            })
        );
    }

    #[test]
    fn source_table_refuses_dates_that_do_not_exist() {
        let result = SourceTable::from_raw(vec![raw("a", "2026-02-30", None)]);
        assert_eq!(
            result,
            Err(DataError::BadValue {
                what: "source",
                id: "a".to_string(),
                field: "retrieved",
                value: "2026-02-30".to_string()
            })
        );
    }

    #[test]
    fn source_table_refuses_releases_that_are_not_releases() {
        let result = SourceTable::from_raw(vec![raw("a", "2026-11-08", Some("26.x.x"))]);
        assert_eq!(
            result,
            Err(DataError::BadValue {
                what: "source",
                id: "a".to_string(),
                field: "release",
                value: "26.x.x".to_string()
            })
        );
    }

    #[test]
    fn source_table_refuses_an_empty_id() {
        let result = SourceTable::from_raw(vec![raw(" ", "2026-11-08", None)]);
        assert!(matches!(
            result,
            Err(DataError::BadValue { field: "id", .. })
        ));
    }
}
