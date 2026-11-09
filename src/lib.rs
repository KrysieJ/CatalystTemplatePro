//! CatalystTemplatePro core library.
//!
//! This crate holds the data definitions and the logic that do not depend on
//! a screen: what a release is, where a fact came from, what a license
//! covers, which platform a template targets, whether a command is available
//! there, and what the tool should warn the engineer about.
//!
//! Every module follows the design recipe from *How to Design Programs*:
//! a data definition with examples of data, then for each function a
//! purpose statement, then examples written as tests, then the body.
//! See docs/DESIGN-DECISIONS.md, D5.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod availability;
pub mod data;
pub mod finding;
pub mod license;
pub mod platform;
pub mod release;
pub mod roles;
pub mod source;
