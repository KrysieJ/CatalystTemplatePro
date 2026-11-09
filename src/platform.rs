//! Hardware platforms, and the target a template is built for.
//!
//! The hardware levels follow Catalyst Center: a device **family** (such as
//! Switches and Hubs) holds **series** (such as Catalyst 9300 Series), and a
//! series holds **platforms** (product IDs such as C9300-48P). Each series
//! also records the software **image** it runs, the Cisco **command
//! reference** it follows, and Cisco's usual Catalyst Center role for it.
//! Images list the **variants** they come in, such as npe.
//!
//! All of it is data, kept in data/platforms.toml and built into the
//! program so it works offline. Adding a device is an edit to that file,
//! not a code change. See docs/DESIGN-DECISIONS.md, D8, D20, D24 and D25.

use std::fmt;

use serde::Deserialize;

use crate::data::DataError;
use crate::license::Licenses;
use crate::release::Release;
use crate::roles::CatalystCenterRole;
use crate::source::{RawSource, Source, SourceTable};

/// The platform list that ships inside the program.
pub const BUILT_IN_PLATFORMS: &str = include_str!("../data/platforms.toml");

/// The platform file layout this build understands.
const SCHEMA_VERSION: u32 = 1;

// ------------------------------------------------------- data definitions

/// A Catalyst Center device family: the broad device type.
///
/// Example of data: id "switches-and-hubs", name "Switches and Hubs".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceFamily {
    /// A short, unique id that series use to point at this family.
    pub id: String,
    /// The family's name.
    pub name: String,
    /// The source that gives this family.
    pub source: Source,
    /// Anything worth knowing about the family.
    pub note: Option<String>,
}

/// A variant an image file can come in, with some features removed.
///
/// Example of data: id "npe", name "No Payload Encryption".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// The short id as it appears in file names, such as "npe" or "noli".
    pub id: String,
    /// What the variant is called.
    pub name: String,
    /// Anything worth knowing about the variant.
    pub note: Option<String>,
}

/// A software image family: the image file you install on a device.
///
/// Example of data: "c8000aep-universalk9", which comes in the variants
/// "npe" and "noli", backed by the engineer's statement from installing
/// firmware.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// The image name as it appears in file names, such as "cat9k_iosxe".
    pub name: String,
    /// The ids of the variants this image is known to come in.
    pub variants: Vec<String>,
    /// The source that says which hardware runs this image.
    pub source: Source,
    /// Anything worth knowing about the image.
    pub note: Option<String>,
}

/// A Cisco command reference that one or more series follow.
///
/// Example of data: id "catalyst-9300", followed by both the Catalyst 9300
/// and Catalyst 9350 series.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandReference {
    /// A short, unique id that series use to point at this reference.
    pub id: String,
    /// The reference's name.
    pub name: String,
    /// The source that says this reference exists.
    pub source: Source,
    /// Anything worth knowing about the reference.
    pub note: Option<String>,
}

/// Cisco's usual Catalyst Center role for a series, with its source.
/// A starting suggestion only; each device's role is set by the engineer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleDefault {
    /// The suggested role.
    pub role: CatalystCenterRole,
    /// The source that gives this role for the series.
    pub source: Source,
}

/// A Catalyst Center series.
///
/// Example of data: the Catalyst 9350 Series is in family
/// "switches-and-hubs", runs "cat9k_iosxe", follows command reference
/// "catalyst-9300", and defaults to the Access role.
///
/// Any of family, image, reference or default role may be `None`, meaning
/// not recorded yet. The program reports those as unknown, never guesses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Series {
    /// A short, unique id that platforms use to point at this series.
    pub id: String,
    /// The series' name as people say it.
    pub name: String,
    /// The id of the Catalyst Center family it belongs to.
    pub family_id: Option<String>,
    /// The name of the image it runs.
    pub image: Option<String>,
    /// The id of the command reference it follows.
    pub reference_id: Option<String>,
    /// Cisco's usual Catalyst Center role for it.
    pub default_role: Option<RoleDefault>,
    /// The source that places this series in its family, image and
    /// reference.
    pub source: Source,
    /// Anything worth knowing about the series.
    pub note: Option<String>,
}

/// A Catalyst Center platform: one product ID.
///
/// Example of data: "C9300-48P" in series "catalyst-9300".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Platform {
    /// The product ID as Cisco writes it.
    pub pid: String,
    /// The id of the series it belongs to.
    pub series_id: String,
    /// The source that places it in that series.
    pub source: Source,
    /// Anything worth knowing about the platform.
    pub note: Option<String>,
}

/// Everything in a platform file, checked and ready to use.
///
/// The fields are private so that a list can only be made by loading and
/// checking a file. Every reference between entries is guaranteed to point
/// at an entry in the same list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformList {
    families: Vec<DeviceFamily>,
    variants: Vec<Variant>,
    images: Vec<Image>,
    references: Vec<CommandReference>,
    series: Vec<Series>,
    platforms: Vec<Platform>,
}

/// A firmware file name, broken into its parts.
///
/// Example of data: "c8000aep-universalk9_npe_noli.17.15.05.SPA.bin" is
/// image "c8000aep-universalk9", variants ["npe", "noli"], release 17.15.5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageFile {
    /// The image name.
    pub image: String,
    /// The variant ids, in the order they appear in the file name.
    pub variants: Vec<String>,
    /// The release.
    pub release: Release,
}

/// Why a file name could not be read as a firmware image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageFileError {
    /// The name does not end in ".SPA.bin".
    NotAnImageFile,
    /// The name does not start with any image in the platform list.
    UnknownImage,
    /// A part after the image name is not a variant that image is known to
    /// come in.
    UnknownVariant(String),
    /// The same variant appears twice.
    RepeatedVariant(String),
    /// The release part is missing or is not a release.
    BadRelease,
}

impl fmt::Display for ImageFileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageFileError::NotAnImageFile => write!(f, "the name does not end in .SPA.bin"),
            ImageFileError::UnknownImage => {
                write!(f, "the name does not start with any known image")
            }
            ImageFileError::UnknownVariant(variant) => {
                write!(f, "\"{variant}\" is not a known variant of this image")
            }
            ImageFileError::RepeatedVariant(variant) => {
                write!(f, "the variant \"{variant}\" appears twice")
            }
            ImageFileError::BadRelease => write!(f, "the release part is missing or not valid"),
        }
    }
}

impl std::error::Error for ImageFileError {}

/// What a template is being built for.
///
/// Example of data: a Catalyst 9300 series switch on 26.1.2 with Network
/// Advantage, given the Catalyst Center role Access and the role tags
/// "ce", "distribution" and "access".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    /// The series.
    pub series: Series,
    /// The exact platform, if the template is for one PID only.
    pub platform: Option<Platform>,
    /// The software release the device will run.
    pub release: Release,
    /// The image variants it will run, such as "npe". Empty for the
    /// standard image.
    pub variants: Vec<String>,
    /// The licenses the device holds, or `None` if not recorded. Unknown
    /// licenses produce reminders, never blocks.
    pub licenses: Option<Licenses>,
    /// The one role Catalyst Center will give the device, or `None` if not
    /// chosen yet.
    pub catalyst_center_role: Option<CatalystCenterRole>,
    /// The ids of the role tags the device carries. Any number.
    pub role_tags: Vec<String>,
}

// -------------------------------------------- the file as written (raw)

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPlatformFile {
    schema_version: u32,
    #[serde(default)]
    source: Vec<RawSource>,
    #[serde(default)]
    family: Vec<RawFamily>,
    #[serde(default)]
    variant: Vec<RawVariant>,
    #[serde(default)]
    image: Vec<RawImage>,
    #[serde(default)]
    reference: Vec<RawReference>,
    #[serde(default)]
    series: Vec<RawSeries>,
    #[serde(default)]
    platform: Vec<RawPlatform>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFamily {
    id: String,
    name: String,
    source: String,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawVariant {
    id: String,
    name: String,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawImage {
    name: String,
    #[serde(default)]
    variants: Vec<String>,
    source: String,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReference {
    id: String,
    name: String,
    source: String,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSeries {
    id: String,
    name: String,
    #[serde(default)]
    family: Option<String>,
    #[serde(default)]
    image: Option<String>,
    #[serde(default)]
    reference: Option<String>,
    #[serde(default)]
    default_role: Option<CatalystCenterRole>,
    #[serde(default)]
    default_role_source: Option<String>,
    source: String,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPlatform {
    pid: String,
    series: String,
    source: String,
    #[serde(default)]
    note: Option<String>,
}

// ---------------------------------------------------------------- loading

impl PlatformList {
    /// Purpose: load the platform list that ships inside the program.
    pub fn built_in() -> Result<PlatformList, DataError> {
        PlatformList::from_toml(BUILT_IN_PLATFORMS)
    }

    /// Purpose: read and check a platform file. Gives back the whole list,
    /// or refuses the whole file with the first problem found.
    ///
    /// Entries are checked in this order, so each kind can only point at
    /// kinds checked before it: sources, families, variants, images,
    /// command references, series, platforms. Their order in the file does
    /// not matter.
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

        let mut families: Vec<DeviceFamily> = Vec::new();
        for entry in raw.family {
            require_text("family", &entry.id, "id", &entry.id)?;
            require_text("family", &entry.id, "name", &entry.name)?;
            refuse_duplicate(
                families.iter().any(|family| family.id == entry.id),
                "family",
                &entry.id,
            )?;
            let source = source_for(&sources, "family", &entry.id, &entry.source)?;
            families.push(DeviceFamily {
                id: entry.id,
                name: entry.name,
                source,
                note: entry.note,
            });
        }

        let mut variants: Vec<Variant> = Vec::new();
        for entry in raw.variant {
            require_text("variant", &entry.id, "id", &entry.id)?;
            require_text("variant", &entry.id, "name", &entry.name)?;
            refuse_duplicate(
                variants.iter().any(|variant| variant.id == entry.id),
                "variant",
                &entry.id,
            )?;
            variants.push(Variant {
                id: entry.id,
                name: entry.name,
                note: entry.note,
            });
        }

        let mut images: Vec<Image> = Vec::new();
        for entry in raw.image {
            require_text("image", &entry.name, "name", &entry.name)?;
            refuse_duplicate(
                images.iter().any(|image| image.name == entry.name),
                "image",
                &entry.name,
            )?;
            for (position, variant_id) in entry.variants.iter().enumerate() {
                must_exist(
                    variants.iter().any(|variant| &variant.id == variant_id),
                    "image",
                    &entry.name,
                    "variants",
                    variant_id,
                )?;
                refuse_duplicate(
                    entry.variants[..position].contains(variant_id),
                    "image variant",
                    variant_id,
                )?;
            }
            let source = source_for(&sources, "image", &entry.name, &entry.source)?;
            images.push(Image {
                name: entry.name,
                variants: entry.variants,
                source,
                note: entry.note,
            });
        }

        let mut references: Vec<CommandReference> = Vec::new();
        for entry in raw.reference {
            require_text("reference", &entry.id, "id", &entry.id)?;
            require_text("reference", &entry.id, "name", &entry.name)?;
            refuse_duplicate(
                references.iter().any(|reference| reference.id == entry.id),
                "reference",
                &entry.id,
            )?;
            let source = source_for(&sources, "reference", &entry.id, &entry.source)?;
            references.push(CommandReference {
                id: entry.id,
                name: entry.name,
                source,
                note: entry.note,
            });
        }

        let mut series_list: Vec<Series> = Vec::new();
        for entry in raw.series {
            require_text("series", &entry.id, "id", &entry.id)?;
            require_text("series", &entry.id, "name", &entry.name)?;
            refuse_duplicate(
                series_list.iter().any(|series| series.id == entry.id),
                "series",
                &entry.id,
            )?;
            if let Some(family_id) = &entry.family {
                must_exist(
                    families.iter().any(|family| &family.id == family_id),
                    "series",
                    &entry.id,
                    "family",
                    family_id,
                )?;
            }
            if let Some(image_name) = &entry.image {
                must_exist(
                    images.iter().any(|image| &image.name == image_name),
                    "series",
                    &entry.id,
                    "image",
                    image_name,
                )?;
            }
            if let Some(reference_id) = &entry.reference {
                must_exist(
                    references
                        .iter()
                        .any(|reference| &reference.id == reference_id),
                    "series",
                    &entry.id,
                    "reference",
                    reference_id,
                )?;
            }
            let default_role = match (entry.default_role, &entry.default_role_source) {
                (Some(role), Some(role_source)) => Some(RoleDefault {
                    role,
                    source: source_for(&sources, "series", &entry.id, role_source)?,
                }),
                (None, None) => None,
                (Some(role), None) => {
                    return Err(DataError::BadValue {
                        what: "series",
                        id: entry.id,
                        field: "default_role_source",
                        value: format!("missing, but default_role is {role}"),
                    })
                }
                (None, Some(role_source)) => {
                    return Err(DataError::BadValue {
                        what: "series",
                        id: entry.id,
                        field: "default_role",
                        value: format!("missing, but default_role_source is {role_source}"),
                    })
                }
            };
            let source = source_for(&sources, "series", &entry.id, &entry.source)?;
            series_list.push(Series {
                id: entry.id,
                name: entry.name,
                family_id: entry.family,
                image: entry.image,
                reference_id: entry.reference,
                default_role,
                source,
                note: entry.note,
            });
        }

        let mut platforms: Vec<Platform> = Vec::new();
        for entry in raw.platform {
            require_text("platform", &entry.pid, "pid", &entry.pid)?;
            refuse_duplicate(
                platforms.iter().any(|platform| platform.pid == entry.pid),
                "platform",
                &entry.pid,
            )?;
            must_exist(
                series_list.iter().any(|series| series.id == entry.series),
                "platform",
                &entry.pid,
                "series",
                &entry.series,
            )?;
            let source = source_for(&sources, "platform", &entry.pid, &entry.source)?;
            platforms.push(Platform {
                pid: entry.pid,
                series_id: entry.series,
                source,
                note: entry.note,
            });
        }

        Ok(PlatformList {
            families,
            variants,
            images,
            references,
            series: series_list,
            platforms,
        })
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

/// Purpose: refuse an entry whose name or id is already taken.
fn refuse_duplicate(already_taken: bool, what: &'static str, id: &str) -> Result<(), DataError> {
    if already_taken {
        return Err(DataError::Duplicate {
            what,
            id: id.to_string(),
        });
    }
    Ok(())
}

/// Purpose: refuse a reference to an entry that does not exist.
fn must_exist(
    found: bool,
    what: &'static str,
    id: &str,
    field: &'static str,
    target: &str,
) -> Result<(), DataError> {
    if !found {
        return Err(DataError::UnknownReference {
            what,
            id: id.to_string(),
            field,
            target: target.to_string(),
        });
    }
    Ok(())
}

/// Purpose: look up the source an entry names, or refuse if it does not
/// exist.
fn source_for(
    sources: &SourceTable,
    what: &'static str,
    id: &str,
    source_id: &str,
) -> Result<Source, DataError> {
    match sources.get(source_id) {
        Some(source) => Ok(source.clone()),
        None => Err(DataError::UnknownReference {
            what,
            id: id.to_string(),
            field: "source",
            target: source_id.to_string(),
        }),
    }
}

// ---------------------------------------------------------------- queries

impl PlatformList {
    /// Purpose: every family, in file order.
    pub fn families(&self) -> &[DeviceFamily] {
        &self.families
    }

    /// Purpose: find a family by its id.
    pub fn family(&self, id: &str) -> Option<&DeviceFamily> {
        self.families.iter().find(|family| family.id == id)
    }

    /// Purpose: every variant, in file order.
    pub fn variants(&self) -> &[Variant] {
        &self.variants
    }

    /// Purpose: every image, in file order.
    pub fn images(&self) -> &[Image] {
        &self.images
    }

    /// Purpose: find an image by its exact name.
    pub fn image(&self, name: &str) -> Option<&Image> {
        self.images.iter().find(|image| image.name == name)
    }

    /// Purpose: every command reference, in file order.
    pub fn references(&self) -> &[CommandReference] {
        &self.references
    }

    /// Purpose: find a command reference by its id.
    pub fn reference(&self, id: &str) -> Option<&CommandReference> {
        self.references.iter().find(|reference| reference.id == id)
    }

    /// Purpose: every series, in file order.
    pub fn series_list(&self) -> &[Series] {
        &self.series
    }

    /// Purpose: find a series by its id.
    pub fn series(&self, id: &str) -> Option<&Series> {
        self.series.iter().find(|series| series.id == id)
    }

    /// Purpose: every series in one family, in file order.
    pub fn series_in_family(&self, family_id: &str) -> Vec<&Series> {
        self.series
            .iter()
            .filter(|series| series.family_id.as_deref() == Some(family_id))
            .collect()
    }

    /// Purpose: every series that runs one image, in file order.
    pub fn series_on_image(&self, image_name: &str) -> Vec<&Series> {
        self.series
            .iter()
            .filter(|series| series.image.as_deref() == Some(image_name))
            .collect()
    }

    /// Purpose: every series that follows one command reference, in file
    /// order.
    pub fn series_following(&self, reference_id: &str) -> Vec<&Series> {
        self.series
            .iter()
            .filter(|series| series.reference_id.as_deref() == Some(reference_id))
            .collect()
    }

    /// Purpose: every platform, in file order.
    pub fn platforms(&self) -> &[Platform] {
        &self.platforms
    }

    /// Purpose: find a platform by its exact PID.
    pub fn platform(&self, pid: &str) -> Option<&Platform> {
        self.platforms.iter().find(|platform| platform.pid == pid)
    }

    /// Purpose: every platform in one series, in file order.
    pub fn platforms_in(&self, series_id: &str) -> Vec<&Platform> {
        self.platforms
            .iter()
            .filter(|platform| platform.series_id == series_id)
            .collect()
    }

    /// Purpose: read a firmware file name such as
    /// "c8000be-universalk9_npe.26.02.01.SPA.bin" into its image, variants
    /// and release.
    ///
    /// The file name must be `<image>[_variant...].<release>.SPA.bin`. Only
    /// images and variants in this list are accepted, so a name that is
    /// almost right is refused rather than guessed at. When two images
    /// could match, the longer name wins.
    pub fn identify_image_file(&self, file_name: &str) -> Result<ImageFile, ImageFileError> {
        let stem = file_name
            .trim()
            .strip_suffix(".SPA.bin")
            .ok_or(ImageFileError::NotAnImageFile)?;
        let (name_part, release_part) = stem.split_once('.').ok_or(ImageFileError::BadRelease)?;
        let release = Release::parse(release_part).map_err(|_| ImageFileError::BadRelease)?;

        let image = self
            .images
            .iter()
            .filter(|image| {
                name_part == image.name || name_part.starts_with(&format!("{}_", image.name))
            })
            .max_by_key(|image| image.name.len())
            .ok_or(ImageFileError::UnknownImage)?;

        let mut variants: Vec<String> = Vec::new();
        let rest = &name_part[image.name.len()..];
        for piece in rest.split('_').skip(1) {
            if !image.variants.iter().any(|known| known == piece) {
                return Err(ImageFileError::UnknownVariant(piece.to_string()));
            }
            if variants.iter().any(|seen| seen == piece) {
                return Err(ImageFileError::RepeatedVariant(piece.to_string()));
            }
            variants.push(piece.to_string());
        }

        Ok(ImageFile {
            image: image.name.clone(),
            variants,
            release,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ImageFile, ImageFileError, PlatformList};
    use crate::data::DataError;
    use crate::release::Release;
    use crate::roles::CatalystCenterRole;

    /// The smallest file that uses every kind of entry. Synthetic: not real
    /// hardware.
    const MINIMAL: &str = r#"
schema_version = 1

[[source]]
id = "example"
kind = "engineer-statement"
title = "Synthetic test fixture"
retrieved = "2026-11-08"

[[family]]
id = "fam"
name = "Example Family"
source = "example"

[[variant]]
id = "npe"
name = "No Payload Encryption"

[[image]]
name = "img-universalk9"
variants = ["npe"]
source = "example"

[[reference]]
id = "ref"
name = "Example Command Reference"
source = "example"

[[series]]
id = "ser"
name = "Example Series"
family = "fam"
image = "img-universalk9"
reference = "ref"
default_role = "access"
default_role_source = "example"
source = "example"

[[platform]]
pid = "EX-48P"
series = "ser"
source = "example"
"#;

    fn built_in() -> PlatformList {
        PlatformList::built_in().expect("data/platforms.toml must load")
    }

    fn ids(series: Vec<&super::Series>) -> Vec<&str> {
        series
            .into_iter()
            .map(|series| series.id.as_str())
            .collect()
    }

    // ---- the built-in list

    #[test]
    fn built_in_newer_series_follow_their_parents() {
        let list = built_in();
        let c9350 = list.series("catalyst-9350").expect("9350 is listed");
        assert_eq!(c9350.reference_id.as_deref(), Some("catalyst-9300"));
        assert_eq!(c9350.image.as_deref(), Some("cat9k_iosxe"));
        assert_eq!(c9350.family_id.as_deref(), Some("switches-and-hubs"));
        assert_eq!(
            ids(list.series_following("catalyst-9600")),
            vec!["catalyst-9600", "catalyst-9610"]
        );
        assert_eq!(
            ids(list.series_following("catalyst-9200")),
            vec!["catalyst-9200", "catalyst-9200cx"]
        );
    }

    #[test]
    fn built_in_default_roles_match_cisco() {
        let list = built_in();
        let role_of = |id: &str| {
            list.series(id)
                .and_then(|series| series.default_role.as_ref())
                .map(|suggestion| suggestion.role)
        };
        assert_eq!(role_of("catalyst-9200cx"), Some(CatalystCenterRole::Access));
        assert_eq!(role_of("catalyst-9400"), Some(CatalystCenterRole::Access));
        assert_eq!(
            role_of("catalyst-9550"),
            Some(CatalystCenterRole::Distribution)
        );
        assert_eq!(role_of("catalyst-9610"), Some(CatalystCenterRole::Core));
        // Not recorded yet: unknown, not guessed.
        assert_eq!(role_of("catalyst-8500-edge"), None);
    }

    #[test]
    fn built_in_router_generations_have_their_own_images() {
        let list = built_in();
        assert_eq!(
            ids(list.series_on_image("c8kg2be-universalk9")),
            vec!["catalyst-8300-secure-gen2", "catalyst-8200-secure-gen2"]
        );
        assert_eq!(
            ids(list.series_on_image("c8000aep-universalk9")),
            vec!["catalyst-8500-edge", "catalyst-8500-secure-gen2"]
        );
        assert_eq!(
            ids(list.series_on_image("c8000be-universalk9")),
            vec!["catalyst-8300-edge"]
        );
    }

    #[test]
    fn built_in_platforms_sit_in_their_series() {
        let list = built_in();
        let pids: Vec<&str> = list
            .platforms_in("ir1800")
            .into_iter()
            .map(|platform| platform.pid.as_str())
            .collect();
        assert_eq!(pids, vec!["IR1833", "IR1835"]);
        assert_eq!(
            list.platform("C9300-48P")
                .map(|platform| platform.series_id.as_str()),
            Some("catalyst-9300")
        );
    }

    #[test]
    fn built_in_keeps_images_no_series_uses_yet() {
        let list = built_in();
        assert!(list.image("cat3k_caa-universalk9").is_some());
        assert!(list.series_on_image("cat3k_caa-universalk9").is_empty());
    }

    // ---- reading firmware file names

    fn file(image: &str, variants: &[&str], release: Release) -> ImageFile {
        ImageFile {
            image: image.to_string(),
            variants: variants.iter().map(|v| v.to_string()).collect(),
            release,
        }
    }

    #[test]
    fn reads_a_plain_image_file() {
        assert_eq!(
            built_in().identify_image_file("cat9k_iosxe.26.01.02.SPA.bin"),
            Ok(file("cat9k_iosxe", &[], Release::new(26, 1, 2)))
        );
    }

    #[test]
    fn reads_an_image_file_with_one_variant() {
        assert_eq!(
            built_in().identify_image_file("c8000be-universalk9_npe.26.02.01.SPA.bin"),
            Ok(file(
                "c8000be-universalk9",
                &["npe"],
                Release::new(26, 2, 1)
            ))
        );
    }

    #[test]
    fn reads_an_image_file_with_two_variants() {
        assert_eq!(
            built_in().identify_image_file("c8000aep-universalk9_npe_noli.17.15.05.SPA.bin"),
            Ok(file(
                "c8000aep-universalk9",
                &["npe", "noli"],
                Release::new(17, 15, 5)
            ))
        );
    }

    #[test]
    fn tells_the_lite_image_from_the_full_one() {
        let result = built_in().identify_image_file("cat9k_lite_iosxe.17.15.04.SPA.bin");
        assert_eq!(
            result.map(|found| found.image),
            Ok("cat9k_lite_iosxe".to_string())
        );
    }

    #[test]
    fn refuses_names_that_are_not_image_files() {
        let list = built_in();
        assert_eq!(
            list.identify_image_file("cat9k_iosxe.26.01.02.bin"),
            Err(ImageFileError::NotAnImageFile)
        );
        assert_eq!(
            list.identify_image_file("unknown-universalk9.26.01.01.SPA.bin"),
            Err(ImageFileError::UnknownImage)
        );
        // Almost a known image name is still unknown.
        assert_eq!(
            list.identify_image_file("cat9k_iosxe2.26.01.02.SPA.bin"),
            Err(ImageFileError::UnknownImage)
        );
    }

    #[test]
    fn refuses_variants_an_image_does_not_come_in() {
        let list = built_in();
        assert_eq!(
            list.identify_image_file("cat9k_iosxe_npe.26.01.02.SPA.bin"),
            Err(ImageFileError::UnknownVariant("npe".to_string()))
        );
        assert_eq!(
            list.identify_image_file("c8000aep-universalk9_npe_npe.17.15.05.SPA.bin"),
            Err(ImageFileError::RepeatedVariant("npe".to_string()))
        );
    }

    #[test]
    fn refuses_bad_release_parts() {
        let list = built_in();
        assert_eq!(
            list.identify_image_file("cat9k_iosxe.26.x.02.SPA.bin"),
            Err(ImageFileError::BadRelease)
        );
        assert_eq!(
            list.identify_image_file("cat9k_iosxe.SPA.bin"),
            Err(ImageFileError::BadRelease)
        );
    }

    // ---- checking a file

    #[test]
    fn minimal_file_loads() {
        let list = PlatformList::from_toml(MINIMAL).expect("the minimal file is valid");
        assert_eq!(list.platforms_in("ser").len(), 1);
        assert_eq!(list.series_in_family("fam").len(), 1);
        assert!(list.platform("EX-48P").is_some());
        assert!(list.platform("ex-48p").is_none());
    }

    #[test]
    fn refuses_a_duplicate_series() {
        let text = format!(
            "{MINIMAL}\n[[series]]\nid = \"ser\"\nname = \"Again\"\nsource = \"example\"\n"
        );
        assert_eq!(
            PlatformList::from_toml(&text),
            Err(DataError::Duplicate {
                what: "series",
                id: "ser".to_string()
            })
        );
    }

    #[test]
    fn refuses_a_series_pointing_at_something_missing() {
        for (field, line, target) in [
            ("family", "family = \"fam\"", "no-fam"),
            ("image", "image = \"img-universalk9\"", "no-img"),
            ("reference", "reference = \"ref\"", "no-ref"),
        ] {
            let text = MINIMAL.replace(line, &format!("{field} = \"{target}\""));
            assert_eq!(
                PlatformList::from_toml(&text),
                Err(DataError::UnknownReference {
                    what: "series",
                    id: "ser".to_string(),
                    field,
                    target: target.to_string()
                }),
                "series {field} pointing at {target}"
            );
        }
    }

    #[test]
    fn refuses_a_default_role_without_its_source() {
        let text = MINIMAL.replace("default_role_source = \"example\"\n", "");
        assert!(matches!(
            PlatformList::from_toml(&text),
            Err(DataError::BadValue {
                field: "default_role_source",
                ..
            })
        ));
    }

    #[test]
    fn refuses_a_role_source_without_a_default_role() {
        let text = MINIMAL.replace("default_role = \"access\"\n", "");
        assert!(matches!(
            PlatformList::from_toml(&text),
            Err(DataError::BadValue {
                field: "default_role",
                ..
            })
        ));
    }

    #[test]
    fn refuses_a_role_catalyst_center_does_not_have() {
        let text = MINIMAL.replace("default_role = \"access\"", "default_role = \"edge\"");
        assert!(matches!(
            PlatformList::from_toml(&text),
            Err(DataError::Syntax(_))
        ));
    }

    #[test]
    fn refuses_an_image_with_an_unknown_variant() {
        let text = MINIMAL.replace("variants = [\"npe\"]", "variants = [\"npe\", \"xyz\"]");
        assert_eq!(
            PlatformList::from_toml(&text),
            Err(DataError::UnknownReference {
                what: "image",
                id: "img-universalk9".to_string(),
                field: "variants",
                target: "xyz".to_string()
            })
        );
    }

    #[test]
    fn refuses_an_image_listing_a_variant_twice() {
        let text = MINIMAL.replace("variants = [\"npe\"]", "variants = [\"npe\", \"npe\"]");
        assert_eq!(
            PlatformList::from_toml(&text),
            Err(DataError::Duplicate {
                what: "image variant",
                id: "npe".to_string()
            })
        );
    }

    #[test]
    fn refuses_a_platform_in_a_series_that_does_not_exist() {
        let text = format!(
            "{MINIMAL}\n[[platform]]\npid = \"EX-24T\"\nseries = \"no-ser\"\nsource = \"example\"\n"
        );
        assert_eq!(
            PlatformList::from_toml(&text),
            Err(DataError::UnknownReference {
                what: "platform",
                id: "EX-24T".to_string(),
                field: "series",
                target: "no-ser".to_string()
            })
        );
    }

    #[test]
    fn refuses_an_entry_with_a_source_that_does_not_exist() {
        let text = MINIMAL.replace(
            "[[platform]]\npid = \"EX-48P\"\nseries = \"ser\"\nsource = \"example\"",
            "[[platform]]\npid = \"EX-48P\"\nseries = \"ser\"\nsource = \"nope\"",
        );
        assert_eq!(
            PlatformList::from_toml(&text),
            Err(DataError::UnknownReference {
                what: "platform",
                id: "EX-48P".to_string(),
                field: "source",
                target: "nope".to_string()
            })
        );
    }

    #[test]
    fn refuses_a_blank_pid() {
        let text = MINIMAL.replace("pid = \"EX-48P\"", "pid = \" \"");
        assert!(matches!(
            PlatformList::from_toml(&text),
            Err(DataError::BadValue { field: "pid", .. })
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
        let text = MINIMAL.replace("series = \"ser\"", "seires = \"ser\"");
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
