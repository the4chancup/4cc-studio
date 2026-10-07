//! The resources bundled into the binary, and the data directory's `templates/` folder, whose
//! files replace them one by one (`team_compiler/pipeline.md` "Resolved decisions",
//! "Templates and fallback bins"): a cup maintainer swaps a fallback bin or a template between
//! releases without a new build.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use pes_version::PesVersion;
use studio_core::{Disposition, Message, Scope};

use crate::messages::{Code, tool_message};

/// The folder's name in the data directory.
const FOLDER_NAME: &str = "templates";

/// A resource the compiler embeds: the file name its replacement in `templates/` carries, and
/// its embedded bytes.
struct Resource {
    name: &'static str,
    embedded: &'static [u8],
}

/// The `TeamColor.bin` a compile sets its teams' colors in when it has no installed one to
/// start from: the same for every version (`resources/bins/README.md`).
const TEAM_COLOR: Resource = Resource {
    name: "TeamColor.bin",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/bins/TeamColor.bin"
    )),
};

/// The `UniColor.bin` a compile sets its kits' menu colors in when it has no installed one to
/// start from: the same for every version (`resources/bins/README.md`).
const UNI_COLOR: Resource = Resource {
    name: "UniColor.bin",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/bins/UniColor.bin"
    )),
};

/// PES 18's `UniformParameter.bin` base (`uniform_parameter_base`).
const UNIFORM_PARAMETER_18: Resource = Resource {
    name: "UniformParameter18.bin",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/bins/UniformParameter18.bin"
    )),
};

/// PES 19 to 21's `UniformParameter.bin` base (`uniform_parameter_base`).
const UNIFORM_PARAMETER_19: Resource = Resource {
    name: "UniformParameter19.bin",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/bins/UniformParameter19.bin"
    )),
};

/// The `kit` texture of a placeholder kit: the magenta/black checkerboard, as a DDS
/// (`resources/kits/README.md`).
const PLACEHOLDER_KIT: Resource = Resource {
    name: "placeholder_kit.dds",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/kits/placeholder_kit.dds"
    )),
};

/// The skeleton packed under a slot's name (`boots.skl`, `fcl_hair_sim.skl`) beside a boots or
/// hair model that brings no skeleton of its own: PES 2021's full-body `body.skl`, since
/// exports put full-body models in both slots (`resources/skeletons/README.md`), not the
/// game's four-bone boots skeleton.
const BODY_SKELETON: Resource = Resource {
    name: "body.skl",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/skeletons/pes21/body.skl"
    )),
};

/// The `face_diff.bin` packed into a Fox face package whose sources hold none: the face
/// parameter file the game expects beside every face's models (`resources/templates/README.md`).
const FACE_DIFF: Resource = Resource {
    name: "face_diff.bin",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/templates/face_diff.bin"
    )),
};

/// The `fcl_hair_sim.fclo` packed beside a `fcl_hair.fmdl` whose sources hold none: a cloth
/// simulation with nothing in it (`resources/templates/README.md`).
const FCL_HAIR_SIM_FCLO: Resource = Resource {
    name: "fcl_hair_sim.fclo",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/templates/fcl_hair_sim.fclo"
    )),
};

/// Every resource a `templates/` file can replace, in the order the replacements are reported.
const RESOURCES: [&Resource; 8] = [
    &TEAM_COLOR,
    &UNI_COLOR,
    &UNIFORM_PARAMETER_18,
    &UNIFORM_PARAMETER_19,
    &PLACEHOLDER_KIT,
    &BODY_SKELETON,
    &FACE_DIFF,
    &FCL_HAIR_SIM_FCLO,
];

/// An override in `templates/` that cannot be read: `template_override_unreadable`'s context.
#[derive(Debug)]
pub(crate) struct Unreadable {
    /// The override file, or the folder when it cannot be listed.
    pub(crate) path: PathBuf,
    /// What failed.
    pub(crate) error: io::Error,
}

/// The resources a compile builds with: each embedded one, or the data directory's
/// `templates/` file of the same name that replaces it (`pipeline.md` "Resolved decisions",
/// "Templates and fallback bins").
pub(crate) struct Templates {
    /// The replacements read from `templates/`, by resource file name.
    overrides: BTreeMap<&'static str, Vec<u8>>,
}

impl Templates {
    /// The embedded resources alone.
    pub(crate) fn embedded() -> Templates {
        Templates {
            overrides: BTreeMap::new(),
        }
    }

    /// The resources with the overrides of `<data_dir>/templates/`, and one
    /// `template_override_active` per override read, in `RESOURCES` order. Nothing is read
    /// without a data directory or without the folder, and a file there naming no resource,
    /// exactly as it is spelled, is not read. An override that cannot be read, or a folder that
    /// cannot be listed, is the error. The bytes are not checked: an override that does not
    /// parse fails where the embedded resource would be parsed.
    pub(crate) fn read(data_dir: Option<&Path>) -> Result<(Templates, Vec<Message>), Unreadable> {
        let mut templates = Templates::embedded();
        let mut messages = Vec::new();
        let Some(data_dir) = data_dir else {
            return Ok((templates, messages));
        };
        let folder = data_dir.join(FOLDER_NAME);
        if !folder.is_dir() {
            return Ok((templates, messages));
        }
        let names = file_names(&folder)?;
        for resource in RESOURCES {
            if !names.contains(OsStr::new(resource.name)) {
                continue;
            }
            let path = folder.join(resource.name);
            let bytes = match fs::read(&path) {
                Ok(bytes) => bytes,
                Err(error) => return Err(Unreadable { path, error }),
            };
            messages.push(tool_message(
                Code::TemplateOverrideActive,
                Scope::Run,
                Disposition::Keep,
                vec![("path", path.display().to_string())],
            ));
            templates.overrides.insert(resource.name, bytes);
        }
        Ok((templates, messages))
    }

    /// `resource`'s bytes: its override when one was read, else the embedded ones.
    fn bytes(&self, resource: &Resource) -> &[u8] {
        self.overrides
            .get(resource.name)
            .map_or(resource.embedded, Vec::as_slice)
    }

    /// The `TeamColor.bin` a compile sets its teams' colors in when it has no installed one to
    /// start from.
    pub(crate) fn team_color(&self) -> &[u8] {
        self.bytes(&TEAM_COLOR)
    }

    /// The `UniColor.bin` a compile sets its kits' menu colors in when it has no installed one
    /// to start from.
    pub(crate) fn uni_color(&self) -> &[u8] {
        self.bytes(&UNI_COLOR)
    }

    /// The `UniformParameter.bin` a compile adds its kit configs to when it has no installed
    /// one to start from: one base for PES 18, one for 19-21. `None` for the pre-Fox versions,
    /// which have no such bin.
    pub(crate) fn uniform_parameter_base(&self, version: PesVersion) -> Option<&[u8]> {
        match version {
            PesVersion::Pes18 => Some(self.bytes(&UNIFORM_PARAMETER_18)),
            PesVersion::Pes19 | PesVersion::Pes20 | PesVersion::Pes21 => {
                Some(self.bytes(&UNIFORM_PARAMETER_19))
            }
            PesVersion::Pes15 | PesVersion::Pes16 | PesVersion::Pes17 => None,
        }
    }

    /// The `kit` texture of a placeholder kit.
    pub(crate) fn placeholder_kit(&self) -> &[u8] {
        self.bytes(&PLACEHOLDER_KIT)
    }

    /// The skeleton packed beside a boots or hair model that brings none of its own.
    pub(crate) fn body_skeleton(&self) -> &[u8] {
        self.bytes(&BODY_SKELETON)
    }

    /// The `face_diff.bin` packed into a Fox face package whose sources hold none.
    pub(crate) fn face_diff(&self) -> &[u8] {
        self.bytes(&FACE_DIFF)
    }

    /// The `fcl_hair_sim.fclo` packed beside a `fcl_hair.fmdl` whose sources hold none.
    pub(crate) fn fcl_hair_sim(&self) -> &[u8] {
        self.bytes(&FCL_HAIR_SIM_FCLO)
    }
}

/// The names of the entries of `folder`, as they are spelled: a Windows file system would open
/// `unicolor.bin` for `UniColor.bin`, so a resource's file is found in the listing, by its
/// exact name, rather than by opening its path. A folder that cannot be listed is the error.
fn file_names(folder: &Path) -> Result<BTreeSet<OsString>, Unreadable> {
    let unreadable = |error| Unreadable {
        path: folder.to_owned(),
        error,
    };
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(folder).map_err(unreadable)? {
        names.insert(entry.map_err(unreadable)?.file_name());
    }
    Ok(names)
}

#[cfg(test)]
mod tests {
    use studio_core::Severity;

    use super::*;
    use crate::testing::scratch;

    #[test]
    fn pes_18_has_its_own_base_and_19_to_21_share_one() {
        let templates = Templates::embedded();
        assert_eq!(
            templates.uniform_parameter_base(PesVersion::Pes18),
            Some(UNIFORM_PARAMETER_18.embedded)
        );
        for version in [PesVersion::Pes19, PesVersion::Pes20, PesVersion::Pes21] {
            assert_eq!(
                templates.uniform_parameter_base(version),
                Some(UNIFORM_PARAMETER_19.embedded)
            );
        }
        for version in [PesVersion::Pes15, PesVersion::Pes16, PesVersion::Pes17] {
            assert_eq!(templates.uniform_parameter_base(version), None);
        }
        assert_ne!(UNIFORM_PARAMETER_18.embedded, UNIFORM_PARAMETER_19.embedded);
    }

    #[test]
    fn every_resource_has_a_name_of_its_own() {
        let names: BTreeSet<&str> = RESOURCES.iter().map(|resource| resource.name).collect();
        assert_eq!(names.len(), RESOURCES.len());
    }

    /// Each resource's bytes in `templates`, through the accessors, in `RESOURCES` order.
    fn every_resource(templates: &Templates) -> [&[u8]; 8] {
        [
            templates.team_color(),
            templates.uni_color(),
            templates.uniform_parameter_base(PesVersion::Pes18).unwrap(),
            templates.uniform_parameter_base(PesVersion::Pes21).unwrap(),
            templates.placeholder_kit(),
            templates.body_skeleton(),
            templates.face_diff(),
            templates.fcl_hair_sim(),
        ]
    }

    /// Asserts that `templates` holds the embedded bytes of every resource.
    fn assert_embedded(templates: &Templates) {
        let embedded = RESOURCES.map(|resource| resource.embedded);
        assert!(
            every_resource(templates) == embedded,
            "every resource embedded"
        );
    }

    #[test]
    fn no_data_directory_or_no_folder_gives_the_embedded_resources_and_no_finding() {
        let temp = scratch("templates_none");
        let (templates, messages) = Templates::read(None).unwrap();
        assert_embedded(&templates);
        assert_eq!(messages, []);

        let (templates, messages) = Templates::read(Some(temp.path())).unwrap();
        assert_embedded(&templates);
        assert_eq!(messages, []);
    }

    #[test]
    fn each_file_naming_a_resource_replaces_it_and_is_reported_in_the_table_s_order() {
        let temp = scratch("templates_read");
        let folder = temp.path().join("templates");
        fs::create_dir(&folder).unwrap();
        // Written in the other order, so the findings' order is the table's, not the folder's.
        fs::write(folder.join("face_diff.bin"), b"face diff override").unwrap();
        fs::write(folder.join("UniColor.bin"), b"kit colors override").unwrap();

        let (templates, messages) = Templates::read(Some(temp.path())).unwrap();

        let mut expected = RESOURCES.map(|resource| resource.embedded);
        expected[1] = b"kit colors override";
        expected[6] = b"face diff override";
        assert!(every_resource(&templates) == expected);
        let active = |name: &str| {
            tool_message(
                Code::TemplateOverrideActive,
                Scope::Run,
                Disposition::Keep,
                vec![("path", folder.join(name).display().to_string())],
            )
        };
        assert_eq!(messages, [active("UniColor.bin"), active("face_diff.bin")]);
        assert_eq!(messages[0].code.code, "template_override_active");
        assert_eq!(messages[0].severity, Severity::Info);
    }

    #[test]
    fn a_file_naming_no_resource_as_it_is_spelled_is_not_read() {
        let temp = scratch("templates_other_names");
        let folder = temp.path().join("templates");
        fs::create_dir(&folder).unwrap();
        fs::write(folder.join("notes.txt"), b"notes").unwrap();
        // `UniColor.bin` in another letter case, which a Windows file system would open as it.
        fs::write(folder.join("unicolor.bin"), b"kit colors override").unwrap();

        let (templates, messages) = Templates::read(Some(temp.path())).unwrap();

        assert_embedded(&templates);
        assert_eq!(messages, []);
    }

    #[test]
    fn an_override_that_cannot_be_read_is_the_error_naming_it() {
        let temp = scratch("templates_unreadable");
        let folder = temp.path().join("templates");
        // Reading a folder fails on every system.
        let team_color = folder.join("TeamColor.bin");
        fs::create_dir_all(&team_color).unwrap();

        let Err(unreadable) = Templates::read(Some(temp.path())) else {
            panic!("an override that cannot be read must be the error, not an absent one");
        };
        assert_eq!(unreadable.path, team_color);
    }
}
