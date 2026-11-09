//! Whether a command or feature is available on one platform and release.

use crate::finding::{Finding, FindingKind};
use crate::license::{LicenseRequirement, Licenses};
use crate::source::Source;

/// What is known about a command or feature on one platform and release.
///
/// Data definition: one of
/// - `Supported`: a source says it works, possibly needing a license;
/// - `NotSupported`: a source says it does not work;
/// - `Unverified`: no source says either way.
///
/// A claim of support or non-support cannot be made without a source: the
/// compiler will refuse it. "We don't know" is a normal answer, not a gap.
/// See docs/DESIGN-DECISIONS.md, D9 and D19.
///
/// Examples of data:
/// - A feature a data sheet lists under Network Advantage is `Supported`
///   with `license: Some(LicenseRequirement::Base(BaseLicense::NetworkAdvantage))`
///   and that data sheet as its source.
/// - A command nobody has checked on the IE3x00 yet is `Unverified` with
///   `note: None`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Availability {
    /// A source says it works.
    Supported {
        /// The license it needs, if any. Per D10, when a source ties a
        /// feature to a license, every command in the feature inherits it.
        license: Option<LicenseRequirement>,
        /// The source that says it works.
        source: Source,
    },
    /// A source says it does not work.
    NotSupported {
        /// The source that says it does not work.
        source: Source,
    },
    /// No source says either way.
    Unverified {
        /// Anything worth knowing while it is unverified, such as where
        /// someone has already looked.
        note: Option<String>,
    },
}

impl Availability {
    /// Purpose: the findings to show the engineer when `subject` (a command,
    /// block, or component name) is used on a target holding `held`
    /// licenses. `held` is `None` when the licenses are not recorded.
    ///
    /// Gives back an empty list when there is nothing to say. Never blocks
    /// anything; it only advises.
    pub fn review(&self, subject: &str, held: Option<&Licenses>) -> Vec<Finding> {
        match self {
            Availability::Supported {
                license: None,
                source: _,
            } => Vec::new(),
            Availability::Supported {
                license: Some(needed),
                source,
            } => match held {
                Some(licenses) if needed.is_met_by(licenses) => Vec::new(),
                Some(_) => vec![Finding {
                    kind: FindingKind::License,
                    subject: subject.to_string(),
                    message: format!(
                        "Requires {needed}. The target's recorded licenses do not include it."
                    ),
                    source: Some(source.clone()),
                }],
                None => vec![Finding {
                    kind: FindingKind::License,
                    subject: subject.to_string(),
                    message: format!(
                        "Requires {needed}. The target's licenses are not recorded."
                    ),
                    source: Some(source.clone()),
                }],
            },
            Availability::NotSupported { source } => vec![Finding {
                kind: FindingKind::Platform,
                subject: subject.to_string(),
                message: "Not supported on the target platform and release.".to_string(),
                source: Some(source.clone()),
            }],
            Availability::Unverified { note } => {
                let message = match note {
                    Some(text) => format!("No source confirms this works on the target. {text}"),
                    None => "No source confirms this works on the target.".to_string(),
                };
                vec![Finding {
                    kind: FindingKind::Unverified,
                    subject: subject.to_string(),
                    message,
                    source: None,
                }]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Availability;
    use crate::finding::FindingKind;
    use crate::license::{AddOnLicense, BaseLicense, LicenseRequirement, Licenses};
    use crate::source::{Date, Source, SourceKind};

    /// A synthetic source for tests. It is not a real document.
    fn example_source() -> Source {
        Source {
            kind: SourceKind::DataSheet,
            title: "Example data sheet (synthetic test fixture)".to_string(),
            release: None,
            location: None,
            retrieved: Date::new(2026, 11, 8).expect("valid date"),
        }
    }

    fn essentials_switch() -> Licenses {
        Licenses {
            base: BaseLicense::NetworkEssentials,
            add_on: Some(AddOnLicense::DnaEssentials),
        }
    }

    fn advantage_switch() -> Licenses {
        Licenses {
            base: BaseLicense::NetworkAdvantage,
            add_on: Some(AddOnLicense::DnaAdvantage),
        }
    }

    fn needs_advantage() -> Availability {
        Availability::Supported {
            license: Some(LicenseRequirement::Base(BaseLicense::NetworkAdvantage)),
            source: example_source(),
        }
    }

    #[test]
    fn supported_with_no_license_says_nothing() {
        let availability = Availability::Supported {
            license: None,
            source: example_source(),
        };
        assert!(availability.review("hostname", Some(&essentials_switch())).is_empty());
        assert!(availability.review("hostname", None).is_empty());
    }

    #[test]
    fn supported_with_the_license_held_says_nothing() {
        let findings = needs_advantage().review("router bgp", Some(&advantage_switch()));
        assert!(findings.is_empty());
    }

    #[test]
    fn missing_license_gives_one_license_finding_with_its_source() {
        let findings = needs_advantage().review("router bgp", Some(&essentials_switch()));
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::License);
        assert_eq!(findings[0].subject, "router bgp");
        assert_eq!(
            findings[0].message,
            "Requires Network Advantage. The target's recorded licenses do not include it."
        );
        assert_eq!(findings[0].source, Some(example_source()));
    }

    #[test]
    fn unrecorded_licenses_give_a_reminder() {
        let findings = needs_advantage().review("router bgp", None);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::License);
        assert_eq!(
            findings[0].message,
            "Requires Network Advantage. The target's licenses are not recorded."
        );
    }

    #[test]
    fn not_supported_gives_a_platform_finding() {
        let availability = Availability::NotSupported {
            source: example_source(),
        };
        let findings = availability.review("example command", Some(&advantage_switch()));
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::Platform);
        assert_eq!(findings[0].source, Some(example_source()));
    }

    #[test]
    fn unverified_gives_a_finding_with_no_source() {
        let availability = Availability::Unverified { note: None };
        let findings = availability.review("example command", Some(&advantage_switch()));
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, FindingKind::Unverified);
        assert_eq!(findings[0].source, None);
        assert_eq!(
            findings[0].message,
            "No source confirms this works on the target."
        );
    }

    #[test]
    fn unverified_carries_its_note() {
        let availability = Availability::Unverified {
            note: Some("Checked release notes only.".to_string()),
        };
        let findings = availability.review("example command", None);
        assert_eq!(
            findings[0].message,
            "No source confirms this works on the target. Checked release notes only."
        );
    }
}
