//! Findings: what the tool tells the engineer about a template.
//!
//! A finding is advice. It never blocks an export. The engineer drives the
//! template and decides what to do. See docs/DESIGN-DECISIONS.md, D11.

use crate::source::Source;

/// What kind of problem a finding reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FindingKind {
    /// The command or feature is not supported on the target platform and
    /// release.
    Platform,
    /// The command or feature needs a license the target does not hold, or
    /// whose holding is not recorded.
    License,
    /// The template does not fit the Catalyst Center workflow it is meant
    /// for, such as using bind variables in a PnP template.
    Workflow,
    /// The template breaks a Catalyst Center template-engine rule.
    SyntaxRule,
    /// Two sources of truth disagree. The tool shows both and does not pick.
    SourceConflict,
    /// No source confirms whether the command or feature works on the
    /// target. See D21.
    Unverified,
}

/// One piece of advice about a template.
///
/// Example of data: a BGP block on a Network Essentials switch, where the
/// data sheet lists BGP under Network Advantage, gives a finding with kind
/// `License`, subject "router bgp", a message naming the required license,
/// and the data sheet as its source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// What kind of problem this is.
    pub kind: FindingKind,
    /// What the finding is about: a command, block, or component name.
    pub subject: String,
    /// The advice, in plain words.
    pub message: String,
    /// The source behind the advice, if there is one. A finding of kind
    /// `Unverified` has none, because the lack of a source is the point.
    pub source: Option<Source>,
}
