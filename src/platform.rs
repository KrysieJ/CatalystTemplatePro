//! Hardware platforms, and the target a template is built for.
//!
//! Families and models are data, kept in data/platforms.toml and built into
//! the program so it works offline. Adding a device is an edit to that
//! file, not a code change. See docs/DESIGN-DECISIONS.md, D8 and D20.

use serde::Deserialize;

use crate::data::DataError;
use crate::license::Licenses;
use crate::release::Release;
use crate::source::{RawSource, Source, SourceTable};

/// The platform list that ships inside the program.
pub const BUILT_IN_PLATFORMS: &str = include_str!("../data/platforms.toml");

/// The platform file layout this build understands.
const SCHEMA_VERSION: u32 = 1;

/// A group of hardware models that share one Cisco command reference.
///
/// Example of data: the Catalyst 9300 family has id "catalyst-9300" and
/// name "Catalyst 9300".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family {
    /// A short, unique id that models use to point at this family.
    pub id: String,
    /// The family's name as people say it.
    pub name: String,
    /// Anything worth knowing about the family.
    pub note: Option<String>,
}

/// One hardware model, and the source that says which family it is in.
///
/// Example of data: the C9350 is in family "catalyst-9300", backed by the
/// engineer's statement that it runs the C9300 image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    /// The model name as Cisco writes it, such as "C9350" or "IR1835".
    pub name: String,
    /// The id of the family whose command reference this model follows.
    pub family_id: String,
    /// The source that says this model belongs to that family.
    pub source: Source,
    /// Anything worth knowing about the model, such as extra features.
    pub note: Option<String>,
}

/// Every known family and model, checked and ready to use.
///
/// The fields are private so that a list can only be made by loading and
/// checking a file. Every model in a `PlatformList` is guaranteed to point
/// at a family in the same list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformList {
    families: Vec<Family>,
    models: Vec<Model>,
}

/// The platform file exactly as written, before checking.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPlatformFile {
    schema_version: u32,
    #[serde(default)]
    source: Vec<RawSource>,
    #[serde(default)]
    family: Vec<RawFamily>,
    #[serde(default)]
    model: Vec<RawModel>,
}

/// A `[[family]]` entry exactly as written, before checking.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFamily {
    id: String,
    name: String,
    #[serde(default)]
    note: Option<String>,
}

/// A `[[model]]` entry exactly as written, before checking.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawModel {
    name: String,
    family: String,
    source: String,
    #[serde(default)]
    note: Option<String>,
}

impl PlatformList {
    /// Purpose: load the platform list that ships inside the program.
    pub fn built_in() -> Result<PlatformList, DataError> {
        PlatformList::from_toml(BUILT_IN_PLATFORMS)
    }

    /// Purpose: read and check a platform file. Gives back the whole list,
    /// or refuses the whole file with the first problem found.
    pub fn from_toml(text: &str) -> Result<PlatformList, DataError> {
        let raw: RawPlatformFile =
            toml::from_str(text).map_err(|error| DataError::Syntax(error.to_string()))?;
        if raw.schema_version != SCHEMA_VERSION {
            return Err(DataError::WrongSchemaVersion {
                found: raw.schema_version,
                expected: SCHEMA_VERSION,
            });
        }
        let sources = SourceTable::from_raw(raw.source)?;

        let mut families: Vec<Family> = Vec::new();
        for entry in raw.family {
            require_text("family", &entry.id, "id", &entry.id)?;
            require_text("family", &entry.id, "name", &entry.name)?;
            if families.iter().any(|family| family.id == entry.id) {
                return Err(DataError::Duplicate {
                    what: "family",
                    id: entry.id,
                });
            }
            families.push(Family {
                id: entry.id,
                name: entry.name,
                note: entry.note,
            });
        }

        let mut models: Vec<Model> = Vec::new();
        for entry in raw.model {
            require_text("model", &entry.name, "name", &entry.name)?;
            if models.iter().any(|model| model.name == entry.name) {
                return Err(DataError::Duplicate {
                    what: "model",
                    id: entry.name,
                });
            }
            if !families.iter().any(|family| family.id == entry.family) {
                return Err(DataError::UnknownReference {
                    what: "model",
                    id: entry.name,
                    field: "family",
                    target: entry.family,
                });
            }
            let source = match sources.get(&entry.source) {
                Some(source) => source.clone(),
                None => {
                    return Err(DataError::UnknownReference {
                        what: "model",
                        id: entry.name,
                        field: "source",
                        target: entry.source,
                    })
                }
            };
            models.push(Model {
                name: entry.name,
                family_id: entry.family,
                source,
                note: entry.note,
            });
        }

        Ok(PlatformList { families, models })
    }

    /// Purpose: every family, in the order the file lists them.
    pub fn families(&self) -> &[Family] {
        &self.families
    }

    /// Purpose: every model, in the order the file lists them.
    pub fn models(&self) -> &[Model] {
        &self.models
    }

    /// Purpose: find a family by its id.
    pub fn family(&self, id: &str) -> Option<&Family> {
        self.families.iter().find(|family| family.id == id)
    }

    /// Purpose: find a model by its exact name.
    pub fn model(&self, name: &str) -> Option<&Model> {
        self.models.iter().find(|model| model.name == name)
    }

    /// Purpose: every model in one family, in file order.
    pub fn models_in(&self, family_id: &str) -> Vec<&Model> {
        self.models
            .iter()
            .filter(|model| model.family_id == family_id)
            .collect()
    }
}

/// Purpose: refuse an empty or blank value for a field that needs text.
fn require_text(
    what: &'static str,
    id: &str,
    field: &'static str,
    value: &str,
) -> Result<(), DataError> {
    if value.trim().is_empty() {
        return Err(DataError::BadValue {
            what,
            id: id.to_string(),
            field,
            value: value.to_string(),
        });
    }
    Ok(())
}

/// What a template is being built for.
///
/// Example of data: a C9200 going to 26.2.1 with Network Essentials and
/// DNA Essentials is a `Target` with that model, `Release::new(26, 2, 1)`,
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
    use super::PlatformList;
    use crate::data::DataError;

    /// The smallest valid platform file. Synthetic: not real hardware.
    const MINIMAL: &str = r#"
schema_version = 1

[[source]]
id = "example"
kind = "engineer-statement"
title = "Synthetic test fixture"
retrieved = "2026-11-08"

[[family]]
id = "fam-a"
name = "Family A"

[[model]]
name = "MODEL-1"
family = "fam-a"
source = "example"
"#;

    #[test]
    fn built_in_list_loads() {
        let list = PlatformList::built_in().expect("data/platforms.toml must load");
        assert_eq!(list.families().len(), 10);
        assert_eq!(
            list.family("catalyst-9300").map(|family| family.name.as_str()),
            Some("Catalyst 9300")
        );
    }

    #[test]
    fn built_in_list_groups_models_by_image() {
        let list = PlatformList::built_in().expect("data/platforms.toml must load");
        let family_of = |name: &str| list.model(name).map(|model| model.family_id.clone());
        assert_eq!(family_of("C9350"), Some("catalyst-9300".to_string()));
        assert_eq!(family_of("C9550"), Some("catalyst-9500".to_string()));
        assert_eq!(family_of("C9610"), Some("catalyst-9600".to_string()));
        assert_eq!(family_of("IE3500"), Some("ie3x00".to_string()));
        assert_eq!(family_of("IR1835"), Some("ir1800".to_string()));
        assert_eq!(list.models_in("ir1800").len(), 2);
    }

    #[test]
    fn minimal_file_loads() {
        let list = PlatformList::from_toml(MINIMAL).expect("the minimal file is valid");
        assert_eq!(list.models_in("fam-a").len(), 1);
        assert!(list.model("MODEL-1").is_some());
        assert!(list.model("model-1").is_none());
    }

    #[test]
    fn refuses_a_duplicate_family() {
        let text = format!("{MINIMAL}\n[[family]]\nid = \"fam-a\"\nname = \"Again\"\n");
        assert_eq!(
            PlatformList::from_toml(&text),
            Err(DataError::Duplicate {
                what: "family",
                id: "fam-a".to_string()
            })
        );
    }

    #[test]
    fn refuses_a_duplicate_model() {
        let text = format!(
            "{MINIMAL}\n[[model]]\nname = \"MODEL-1\"\nfamily = \"fam-a\"\nsource = \"example\"\n"
        );
        assert_eq!(
            PlatformList::from_toml(&text),
            Err(DataError::Duplicate {
                what: "model",
                id: "MODEL-1".to_string()
            })
        );
    }

    #[test]
    fn refuses_a_model_in_a_family_that_does_not_exist() {
        let text = format!(
            "{MINIMAL}\n[[model]]\nname = \"MODEL-2\"\nfamily = \"fam-b\"\nsource = \"example\"\n"
        );
        assert_eq!(
            PlatformList::from_toml(&text),
            Err(DataError::UnknownReference {
                what: "model",
                id: "MODEL-2".to_string(),
                field: "family",
                target: "fam-b".to_string()
            })
        );
    }

    #[test]
    fn refuses_a_model_with_a_source_that_does_not_exist() {
        let text = format!(
            "{MINIMAL}\n[[model]]\nname = \"MODEL-2\"\nfamily = \"fam-a\"\nsource = \"nope\"\n"
        );
        assert_eq!(
            PlatformList::from_toml(&text),
            Err(DataError::UnknownReference {
                what: "model",
                id: "MODEL-2".to_string(),
                field: "source",
                target: "nope".to_string()
            })
        );
    }

    #[test]
    fn refuses_an_empty_model_name() {
        let text = format!(
            "{MINIMAL}\n[[model]]\nname = \"\"\nfamily = \"fam-a\"\nsource = \"example\"\n"
        );
        assert!(matches!(
            PlatformList::from_toml(&text),
            Err(DataError::BadValue { field: "name", .. })
        ));
    }

    #[test]
    fn refuses_a_newer_layout() {
        let text = MINIMAL.replace("schema_version = 1", "schema_version = 2");
        assert_eq!(
            PlatformList::from_toml(&text),
            Err(DataError::WrongSchemaVersion {
                found: 2,
                expected: 1
            })
        );
    }

    #[test]
    fn refuses_a_misspelled_field() {
        let text = MINIMAL.replace("family = \"fam-a\"", "famly = \"fam-a\"");
        assert!(matches!(
            PlatformList::from_toml(&text),
            Err(DataError::Syntax(_))
        ));
    }

    #[test]
    fn refuses_an_unknown_source_kind() {
        let text = MINIMAL.replace("engineer-statement", "rumour");
        assert!(matches!(
            PlatformList::from_toml(&text),
            Err(DataError::Syntax(_))
        ));
    }

    #[test]
    fn refuses_text_that_is_not_toml() {
        assert!(matches!(
            PlatformList::from_toml("this is not [ toml"),
            Err(DataError::Syntax(_))
        ));
    }
}
