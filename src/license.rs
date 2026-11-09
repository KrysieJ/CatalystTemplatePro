//! License levels and what each one covers.
//!
//! Scope: these levels are the Catalyst 9000 switch levels from Cisco's 9200
//! configuration guide (see docs/DESIGN-DECISIONS.md, D10, reference [9]):
//! base licenses Network Essentials and Network Advantage, and add-on
//! licenses DNA Essentials and DNA Advantage. That guide states Network
//! Advantage includes Network Essentials, and DNA Advantage includes DNA
//! Essentials. [Verified]
//!
//! [Unverified] License models for routers (Catalyst 8300/8500, IR1800),
//! IE3x00 switches, and the 9800 were not checked. They will need their own
//! levels once a Cisco source is found.
//!
//! [Unverified] Whether an add-on level depends on a particular base level
//! is not encoded here. A device's base and add-on are checked separately.

use std::fmt;

/// A base network license level on a Catalyst 9000 switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BaseLicense {
    /// Network Essentials.
    NetworkEssentials,
    /// Network Advantage. Includes everything in Network Essentials.
    NetworkAdvantage,
}

/// An add-on license level on a Catalyst 9000 switch.
///
/// Cisco's guide calls these DNA Essentials and DNA Advantage. The engineer
/// also refers to the same tiers as Catalyst Essentials and Catalyst
/// Advantage. [Engineer]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddOnLicense {
    /// DNA (Catalyst) Essentials.
    DnaEssentials,
    /// DNA (Catalyst) Advantage. Includes everything in DNA Essentials.
    DnaAdvantage,
}

impl BaseLicense {
    /// Purpose: whether a device holding this level meets a requirement
    /// for `needed`.
    pub fn covers(self, needed: BaseLicense) -> bool {
        match (self, needed) {
            (BaseLicense::NetworkAdvantage, _) => true,
            (BaseLicense::NetworkEssentials, BaseLicense::NetworkEssentials) => true,
            (BaseLicense::NetworkEssentials, BaseLicense::NetworkAdvantage) => false,
        }
    }
}

impl AddOnLicense {
    /// Purpose: whether a device holding this add-on meets a requirement
    /// for `needed`.
    pub fn covers(self, needed: AddOnLicense) -> bool {
        match (self, needed) {
            (AddOnLicense::DnaAdvantage, _) => true,
            (AddOnLicense::DnaEssentials, AddOnLicense::DnaEssentials) => true,
            (AddOnLicense::DnaEssentials, AddOnLicense::DnaAdvantage) => false,
        }
    }
}

impl fmt::Display for BaseLicense {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BaseLicense::NetworkEssentials => write!(f, "Network Essentials"),
            BaseLicense::NetworkAdvantage => write!(f, "Network Advantage"),
        }
    }
}

impl fmt::Display for AddOnLicense {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddOnLicense::DnaEssentials => write!(f, "DNA Essentials"),
            AddOnLicense::DnaAdvantage => write!(f, "DNA Advantage"),
        }
    }
}

/// The licenses a target device holds.
///
/// Example of data: a switch bought with Network Essentials and the DNA
/// Essentials add-on is
/// `Licenses { base: BaseLicense::NetworkEssentials, add_on: Some(AddOnLicense::DnaEssentials) }`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Licenses {
    /// The base license level.
    pub base: BaseLicense,
    /// The add-on level, if the device has one.
    pub add_on: Option<AddOnLicense>,
}

/// What a feature needs in order to work.
///
/// Data definition: one of
/// - a base level only,
/// - an add-on level only,
/// - both a base level and an add-on level.
///
/// There is no "nothing required" case on purpose. A feature with no
/// license requirement simply has no `LicenseRequirement`.
///
/// Example of data: a feature the data sheet lists under Network Advantage
/// is `LicenseRequirement::Base(BaseLicense::NetworkAdvantage)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LicenseRequirement {
    /// Needs this base level or higher.
    Base(BaseLicense),
    /// Needs this add-on level or higher.
    AddOn(AddOnLicense),
    /// Needs both this base level and this add-on level, or higher.
    Both(BaseLicense, AddOnLicense),
}

impl LicenseRequirement {
    /// Purpose: whether the licenses a device holds meet this requirement.
    pub fn is_met_by(&self, held: &Licenses) -> bool {
        match *self {
            LicenseRequirement::Base(needed) => held.base.covers(needed),
            LicenseRequirement::AddOn(needed) => {
                held.add_on.is_some_and(|have| have.covers(needed))
            }
            LicenseRequirement::Both(needed_base, needed_add_on) => {
                held.base.covers(needed_base)
                    && held.add_on.is_some_and(|have| have.covers(needed_add_on))
            }
        }
    }
}

impl fmt::Display for LicenseRequirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LicenseRequirement::Base(base) => write!(f, "{base}"),
            LicenseRequirement::AddOn(add_on) => write!(f, "{add_on}"),
            LicenseRequirement::Both(base, add_on) => write!(f, "{base} + {add_on}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AddOnLicense, BaseLicense, LicenseRequirement, Licenses};

    #[test]
    fn base_levels_cover_themselves_and_below() {
        assert!(BaseLicense::NetworkEssentials.covers(BaseLicense::NetworkEssentials));
        assert!(BaseLicense::NetworkAdvantage.covers(BaseLicense::NetworkAdvantage));
        assert!(BaseLicense::NetworkAdvantage.covers(BaseLicense::NetworkEssentials));
        assert!(!BaseLicense::NetworkEssentials.covers(BaseLicense::NetworkAdvantage));
    }

    #[test]
    fn add_on_levels_cover_themselves_and_below() {
        assert!(AddOnLicense::DnaEssentials.covers(AddOnLicense::DnaEssentials));
        assert!(AddOnLicense::DnaAdvantage.covers(AddOnLicense::DnaAdvantage));
        assert!(AddOnLicense::DnaAdvantage.covers(AddOnLicense::DnaEssentials));
        assert!(!AddOnLicense::DnaEssentials.covers(AddOnLicense::DnaAdvantage));
    }

    #[test]
    fn base_requirement_checks_only_the_base() {
        let needs_advantage = LicenseRequirement::Base(BaseLicense::NetworkAdvantage);
        let essentials_switch = Licenses {
            base: BaseLicense::NetworkEssentials,
            add_on: Some(AddOnLicense::DnaAdvantage),
        };
        let advantage_switch = Licenses {
            base: BaseLicense::NetworkAdvantage,
            add_on: None,
        };
        assert!(!needs_advantage.is_met_by(&essentials_switch));
        assert!(needs_advantage.is_met_by(&advantage_switch));
    }

    #[test]
    fn add_on_requirement_fails_when_there_is_no_add_on() {
        let needs_dna = LicenseRequirement::AddOn(AddOnLicense::DnaEssentials);
        let no_add_on = Licenses {
            base: BaseLicense::NetworkAdvantage,
            add_on: None,
        };
        let with_add_on = Licenses {
            base: BaseLicense::NetworkEssentials,
            add_on: Some(AddOnLicense::DnaEssentials),
        };
        assert!(!needs_dna.is_met_by(&no_add_on));
        assert!(needs_dna.is_met_by(&with_add_on));
    }

    #[test]
    fn both_requirement_needs_both() {
        let needs_both =
            LicenseRequirement::Both(BaseLicense::NetworkAdvantage, AddOnLicense::DnaAdvantage);
        let only_base = Licenses {
            base: BaseLicense::NetworkAdvantage,
            add_on: Some(AddOnLicense::DnaEssentials),
        };
        let only_add_on = Licenses {
            base: BaseLicense::NetworkEssentials,
            add_on: Some(AddOnLicense::DnaAdvantage),
        };
        let everything = Licenses {
            base: BaseLicense::NetworkAdvantage,
            add_on: Some(AddOnLicense::DnaAdvantage),
        };
        assert!(!needs_both.is_met_by(&only_base));
        assert!(!needs_both.is_met_by(&only_add_on));
        assert!(needs_both.is_met_by(&everything));
    }

    #[test]
    fn requirements_print_in_plain_words() {
        assert_eq!(
            LicenseRequirement::Base(BaseLicense::NetworkAdvantage).to_string(),
            "Network Advantage"
        );
        assert_eq!(
            LicenseRequirement::AddOn(AddOnLicense::DnaEssentials).to_string(),
            "DNA Essentials"
        );
        assert_eq!(
            LicenseRequirement::Both(BaseLicense::NetworkAdvantage, AddOnLicense::DnaAdvantage)
                .to_string(),
            "Network Advantage + DNA Advantage"
        );
    }
}
