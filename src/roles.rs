//! Roles: the one role Catalyst Center gives a device, and the role tags
//! that say what the device actually does. See docs/DESIGN-DECISIONS.md, D24.

use std::fmt;

use serde::Deserialize;

use crate::data::DataError;

/// The role tags that ship inside the program.
pub const BUILT_IN_ROLE_TAGS: &str = include_str!("../data/roles.toml");

/// The role tag file layout this build understands.
const SCHEMA_VERSION: u32 = 1;

/// The role Catalyst Center assigns a device. Exactly one per device.
///
/// Catalyst Center uses this role for things such as telemetry and QoS, so
/// it is set to whatever Catalyst Center needs for the device. It is never
/// worked out from the role tags: a site router that is functionally
/// border, distribution and access may still need to be Access here.
/// [Engineer]
///
/// In data files: `"border-router"`, `"core"`, `"distribution"`, `"access"`.
///
/// [Unverified] Whether Catalyst Center offers any role beyond these four.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CatalystCenterRole {
    /// Border Router.
    BorderRouter,
    /// Core.
    Core,
    /// Distribution.
    Distribution,
    /// Access.
    Access,
}

impl fmt::Display for CatalystCenterRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            CatalystCenterRole::BorderRouter => "Border Router",
            CatalystCenterRole::Core => "Core",
            CatalystCenterRole::Distribution => "Distribution",
            CatalystCenterRole::Access => "Access",
        };
        write!(f, "{name}")
    }
}

/// A role tag: one thing a device does.
///
/// A device can carry any number of tags. Components attach to tags, and a
/// device gets every component its tags call for.
///
/// Example of data: the tag with id "ce", name "CE / Site Router".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleTag {
    /// A short, unique id: lowercase letters, digits and hyphens.
    pub id: String,
    /// The tag's name as people say it.
    pub name: String,
    /// What a device with this tag does.
    pub description: Option<String>,
}

/// Every role tag, checked and ready to use.
///
/// The field is private so that a list can only be made by loading and
/// checking a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleTagList {
    tags: Vec<RoleTag>,
}

/// The role tag file exactly as written, before checking.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRoleFile {
    schema_version: u32,
    #[serde(default)]
    tag: Vec<RawRoleTag>,
}

/// A `[[tag]]` entry exactly as written, before checking.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRoleTag {
    id: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
}

impl RoleTagList {
    /// Purpose: load the role tags that ship inside the program.
    pub fn built_in() -> Result<RoleTagList, DataError> {
        RoleTagList::from_toml(BUILT_IN_ROLE_TAGS)
    }

    /// Purpose: read and check a role tag file. Gives back the whole list,
    /// or refuses the whole file with the first problem found.
    pub fn from_toml(text: &str) -> Result<RoleTagList, DataError> {
        let raw: RawRoleFile =
            toml::from_str(text).map_err(|error| DataError::Syntax(error.to_string()))?;
        if raw.schema_version != SCHEMA_VERSION {
            return Err(DataError::WrongSchemaVersion {
                found: raw.schema_version,
                expected: SCHEMA_VERSION,
            });
        }
        let mut tags: Vec<RoleTag> = Vec::new();
        for entry in raw.tag {
            if !is_valid_tag_id(&entry.id) {
                return Err(DataError::BadValue {
                    what: "tag",
                    id: entry.id.clone(),
                    field: "id",
                    value: entry.id,
                });
            }
            if entry.name.trim().is_empty() {
                return Err(DataError::BadValue {
                    what: "tag",
                    id: entry.id,
                    field: "name",
                    value: entry.name,
                });
            }
            if tags.iter().any(|tag| tag.id == entry.id) {
                return Err(DataError::Duplicate {
                    what: "tag",
                    id: entry.id,
                });
            }
            tags.push(RoleTag {
                id: entry.id,
                name: entry.name,
                description: entry.description,
            });
        }
        Ok(RoleTagList { tags })
    }

    /// Purpose: every tag, in the order the file lists them.
    pub fn tags(&self) -> &[RoleTag] {
        &self.tags
    }

    /// Purpose: find a tag by its id.
    pub fn get(&self, id: &str) -> Option<&RoleTag> {
        self.tags.iter().find(|tag| tag.id == id)
    }

    /// Purpose: which of the given tag ids are not in this list, in the
    /// order given. Used to warn about a device carrying a tag that has
    /// been renamed or removed.
    pub fn unknown<'a>(&self, ids: &'a [String]) -> Vec<&'a str> {
        ids.iter()
            .map(|id| id.as_str())
            .filter(|id| self.get(id).is_none())
            .collect()
    }
}

/// Purpose: whether text is a usable tag id: not empty, and only lowercase
/// letters, digits and hyphens.
fn is_valid_tag_id(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[cfg(test)]
mod tests {
    use super::{CatalystCenterRole, RoleTagList};
    use crate::data::DataError;

    const MINIMAL: &str = r#"
schema_version = 1

[[tag]]
id = "access"
name = "Access"
"#;

    #[test]
    fn catalyst_center_roles_print_their_names() {
        assert_eq!(
            CatalystCenterRole::BorderRouter.to_string(),
            "Border Router"
        );
        assert_eq!(CatalystCenterRole::Access.to_string(), "Access");
    }

    #[test]
    fn built_in_tags_load() {
        let list = RoleTagList::built_in().expect("data/roles.toml must load");
        for id in [
            "border",
            "core",
            "p-router",
            "distribution",
            "pe",
            "ce",
            "access",
        ] {
            assert!(list.get(id).is_some(), "missing built-in tag {id}");
        }
    }

    #[test]
    fn finds_tags_that_are_not_defined() {
        let list = RoleTagList::from_toml(MINIMAL).expect("valid file");
        let carried = vec![
            "access".to_string(),
            "old-name".to_string(),
            "border".to_string(),
        ];
        assert_eq!(list.unknown(&carried), vec!["old-name", "border"]);
    }

    #[test]
    fn refuses_a_duplicate_tag() {
        let text = format!("{MINIMAL}\n[[tag]]\nid = \"access\"\nname = \"Again\"\n");
        assert_eq!(
            RoleTagList::from_toml(&text),
            Err(DataError::Duplicate {
                what: "tag",
                id: "access".to_string()
            })
        );
    }

    #[test]
    fn refuses_tag_ids_with_spaces_or_capitals() {
        for bad in ["Access", "site router", "", "ce_router"] {
            let text = MINIMAL.replace("id = \"access\"", &format!("id = \"{bad}\""));
            assert!(
                matches!(
                    RoleTagList::from_toml(&text),
                    Err(DataError::BadValue { field: "id", .. })
                ),
                "tag id {bad:?} should be refused"
            );
        }
    }

    #[test]
    fn refuses_a_blank_name() {
        let text = MINIMAL.replace("name = \"Access\"", "name = \"  \"");
        assert!(matches!(
            RoleTagList::from_toml(&text),
            Err(DataError::BadValue { field: "name", .. })
        ));
    }

    #[test]
    fn refuses_a_misspelled_field() {
        let text = MINIMAL.replace("name = ", "nmae = ");
        assert!(matches!(
            RoleTagList::from_toml(&text),
            Err(DataError::Syntax(_))
        ));
    }
}
