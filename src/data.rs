//! Errors shared by every data file the tool loads: the platform list now,
//! the command catalog later. See docs/DESIGN-DECISIONS.md, D20 and D23.

use std::fmt;

/// Why a data file was refused.
///
/// The tool refuses the whole file rather than loading part of it, so a
/// mistake can never quietly drop an entry.
///
/// Example of data: a model that names a family nobody defined is
/// `DataError::UnknownReference { what: "model", id: "C9999", field: "family", target: "catalyst-9999" }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataError {
    /// The file is not valid TOML, or an entry has a missing, extra, or
    /// misspelled field, or a value of the wrong kind. The message comes
    /// from the TOML reader and names the line.
    Syntax(String),
    /// The file says it uses a layout this build does not understand.
    WrongSchemaVersion {
        /// The version the file states.
        found: u32,
        /// The version this build reads.
        expected: u32,
    },
    /// Two entries share a name or id that must be unique.
    Duplicate {
        /// The kind of entry, such as "family" or "model".
        what: &'static str,
        /// The repeated name or id.
        id: String,
    },
    /// An entry refers to something that does not exist in the file.
    UnknownReference {
        /// The kind of entry making the reference.
        what: &'static str,
        /// The name or id of that entry.
        id: String,
        /// The field holding the reference.
        field: &'static str,
        /// The name or id it refers to, which does not exist.
        target: String,
    },
    /// A value is present but cannot be used, such as an empty name, a
    /// date that does not exist, or a release that is not a release.
    BadValue {
        /// The kind of entry.
        what: &'static str,
        /// The name or id of that entry.
        id: String,
        /// The field with the bad value.
        field: &'static str,
        /// The value as written in the file.
        value: String,
    },
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::Syntax(message) => write!(f, "The file could not be read: {message}"),
            DataError::WrongSchemaVersion { found, expected } => write!(
                f,
                "The file uses layout version {found}; this build reads version {expected}."
            ),
            DataError::Duplicate { what, id } => {
                write!(f, "Two {what} entries share the name or id \"{id}\".")
            }
            DataError::UnknownReference {
                what,
                id,
                field,
                target,
            } => write!(
                f,
                "The {what} \"{id}\" has {field} = \"{target}\", which does not exist in the file."
            ),
            DataError::BadValue {
                what,
                id,
                field,
                value,
            } => write!(
                f,
                "The {what} \"{id}\" has a {field} that cannot be used: \"{value}\"."
            ),
        }
    }
}

impl std::error::Error for DataError {}

#[cfg(test)]
mod tests {
    use super::DataError;

    #[test]
    fn messages_name_the_entry_and_the_problem() {
        let error = DataError::UnknownReference {
            what: "model",
            id: "C9999".to_string(),
            field: "family",
            target: "catalyst-9999".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "The model \"C9999\" has family = \"catalyst-9999\", which does not exist in the file."
        );
    }
}
