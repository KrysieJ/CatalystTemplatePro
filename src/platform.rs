//! Hardware platforms, and the target a template is built for.

use std::fmt;

use crate::license::Licenses;
use crate::release::Release;

/// A group of hardware models that share one Cisco command reference.
///
/// Models are grouped by the software they run, not by marketing name, so
/// one command catalog serves the whole family and per-model differences
/// become callouts. See docs/DESIGN-DECISIONS.md, D8.
///
/// Temporary: this fixed list will be replaced by a platform data file, so
/// that adding a device needs no code change. See D20.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Family {
    /// Catalyst 9200 series.
    Catalyst9200,
    /// Catalyst 9300 series. Includes the C9350, which runs the same image
    /// with extra features. [Engineer]
    Catalyst9300,
    /// Catalyst 9400 series.
    Catalyst9400,
    /// Catalyst 9500 series. Includes the C9550, which runs the same image
    /// with extra features. [Engineer]
    Catalyst9500,
    /// Catalyst 9600 series. Includes the C9610, which runs the same image
    /// with extra features. [Engineer]
    Catalyst9600,
    /// Catalyst 9800 series wireless controllers.
    Catalyst9800,
    /// Catalyst 8300 series edge platforms. [Unverified] whether the 8200
    /// belongs here; Cisco publishes 8200 and 8300 release notes together.
    Catalyst8300,
    /// Catalyst 8500 series edge platforms.
    Catalyst8500,
    /// IR1800 series rugged routers. Includes the IR1833 and IR1835, which
    /// share an image and command reference and differ by an I/O header.
    /// [Engineer]
    Ir1800,
    /// IE3x00 series rugged switches. Includes the IE3400 and IE3500; the
    /// "x" is a wildcard in the image name. [Engineer]
    Ie3x00,
}

impl fmt::Display for Family {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Family::Catalyst9200 => "Catalyst 9200",
            Family::Catalyst9300 => "Catalyst 9300",
            Family::Catalyst9400 => "Catalyst 9400",
            Family::Catalyst9500 => "Catalyst 9500",
            Family::Catalyst9600 => "Catalyst 9600",
            Family::Catalyst9800 => "Catalyst 9800",
            Family::Catalyst8300 => "Catalyst 8300",
            Family::Catalyst8500 => "Catalyst 8500",
            Family::Ir1800 => "IR1800",
            Family::Ie3x00 => "IE3x00",
        };
        write!(f, "{name}")
    }
}

/// One hardware model.
///
/// Example of data: a C9350 is
/// `Model { name: "C9350".to_string(), family: Family::Catalyst9300 }`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Model {
    /// The model name as Cisco writes it, such as "C9350" or "IR1835".
    pub name: String,
    /// The family whose command reference this model follows.
    pub family: Family,
}

/// What a template is being built for.
///
/// Example of data: a Catalyst 9200 going to 26.2.1 with Network Essentials
/// and DNA Essentials is a `Target` with that model, `Release::new(26, 2, 1)`,
/// and those licenses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The hardware model.
    pub model: Model,
    /// The software release the device will run.
    pub release: Release,
    /// The licenses the device holds, or `None` if the engineer has not
    /// recorded them. Unknown licenses produce reminders, never blocks.
    pub licenses: Option<Licenses>,
}

#[cfg(test)]
mod tests {
    use super::{Family, Model};

    #[test]
    fn families_print_their_names() {
        assert_eq!(Family::Catalyst9300.to_string(), "Catalyst 9300");
        assert_eq!(Family::Ir1800.to_string(), "IR1800");
        assert_eq!(Family::Ie3x00.to_string(), "IE3x00");
    }

    #[test]
    fn newer_models_share_their_family() {
        let c9350 = Model {
            name: "C9350".to_string(),
            family: Family::Catalyst9300,
        };
        let c9300 = Model {
            name: "C9300".to_string(),
            family: Family::Catalyst9300,
        };
        assert_eq!(c9350.family, c9300.family);
        assert_ne!(c9350, c9300);
    }
}
