//! The resources bundled into the binary, and the data directory's `templates/` folder, whose
//! files replace them one by one (`team_compiler/pipeline.md` "Resolved decisions",
//! "Templates and fallback bins"): a cup maintainer swaps a fallback bin or a template between
//! releases without a new build.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};

use pes_version::PesVersion;
use studio_core::{Disposition, Message, Scope, ToolContext};
use uniparam::UniformParameter;

use crate::bins::{TeamColorBin, UniColorBin, dpfl};
use crate::events::RunEvents;
use crate::messages::{Code, tool_message};

/// The folder's name in the data directory.
const FOLDER_NAME: &str = "templates";

/// A resource the compiler embeds: the file name its replacement in `templates/` carries, its
/// embedded bytes, and what its replacement is parsed as when it is read.
struct Resource {
    name: &'static str,
    embedded: &'static [u8],
    format: Format,
}

/// What a resource's replacement in `templates/` is parsed as when it is read: the three bins
/// and the official `DpFileList.bin` are, so one that does not parse stops the run before any
/// export is read; the other resources are packed or converted as they are, and fail where the
/// embedded one would be used.
#[derive(Clone, Copy)]
enum Format {
    /// Not parsed.
    Unparsed,
    /// A `TeamColor.bin`.
    TeamColor,
    /// A `UniColor.bin`.
    UniColor,
    /// A `UniformParameter.bin`.
    UniformParameter,
    /// A `DpFileList.bin`.
    DpFileList,
}

impl Format {
    /// `bytes`, given back once they parse as the format; the error says why they do not.
    fn check(self, bytes: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        match self {
            Format::Unparsed => Ok(bytes),
            // The bins take the bytes and give them back, so the check copies nothing.
            Format::TeamColor => Ok(TeamColorBin::read(bytes)?.into_bytes()),
            Format::UniColor => Ok(UniColorBin::read(bytes)?.into_bytes()),
            Format::UniformParameter => {
                UniformParameter::read(&bytes)?;
                Ok(bytes)
            }
            Format::DpFileList => {
                dpfl::entries(&bytes)?;
                Ok(bytes)
            }
        }
    }
}

/// The `TeamColor.bin` a compile sets its teams' colors in when it has no installed one to
/// start from: the same for every version (`resources/bins/README.md`).
const TEAM_COLOR: Resource = Resource {
    name: "TeamColor.bin",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/bins/TeamColor.bin"
    )),
    format: Format::TeamColor,
};

/// The `UniColor.bin` a compile sets its kits' menu colors in when it has no installed one to
/// start from: the same for every version (`resources/bins/README.md`).
const UNI_COLOR: Resource = Resource {
    name: "UniColor.bin",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/bins/UniColor.bin"
    )),
    format: Format::UniColor,
};

/// PES 18's `UniformParameter.bin` base (`uniform_parameter_base`).
const UNIFORM_PARAMETER_18: Resource = Resource {
    name: "UniformParameter18.bin",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/bins/UniformParameter18.bin"
    )),
    format: Format::UniformParameter,
};

/// PES 19 to 21's `UniformParameter.bin` base (`uniform_parameter_base`).
const UNIFORM_PARAMETER_19: Resource = Resource {
    name: "UniformParameter19.bin",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/bins/UniformParameter19.bin"
    )),
    format: Format::UniformParameter,
};

/// The `kit` texture of a placeholder kit: the magenta/black checkerboard, as a DDS
/// (`resources/kits/README.md`).
const PLACEHOLDER_KIT: Resource = Resource {
    name: "placeholder_kit.dds",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/kits/placeholder_kit.dds"
    )),
    format: Format::Unparsed,
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
    format: Format::Unparsed,
};

/// The `face_diff.bin` packed into a Fox face package whose sources hold none: the face
/// parameter file the game expects beside every face's models (`resources/templates/README.md`).
const FACE_DIFF: Resource = Resource {
    name: "face_diff.bin",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/templates/face_diff.bin"
    )),
    format: Format::Unparsed,
};

/// The `fcl_hair_sim.fclo` packed beside a `fcl_hair.fmdl` whose sources hold none: a cloth
/// simulation with nothing in it (`resources/templates/README.md`).
const FCL_HAIR_SIM_FCLO: Resource = Resource {
    name: "fcl_hair_sim.fclo",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/templates/fcl_hair_sim.fclo"
    )),
    format: Format::Unparsed,
};

/// The cup's official `DpFileList.bin`, which a compile that deploys compares the installed
/// list with: one list for every PES version (`resources/templates/README.md`).
const DPFILELIST: Resource = Resource {
    name: "DpFileList.bin",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/templates/DpFileList.bin"
    )),
    format: Format::DpFileList,
};

/// The empty CPK `upgrade-dpfl` writes for an official list entry with no file in `download/`:
/// the game loads none of the folder's CPKs when a listed one is missing
/// (`resources/templates/README.md`).
const PLACEHOLDER_CPK: Resource = Resource {
    name: "placeholder.cpk",
    embedded: include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../resources/templates/placeholder.cpk"
    )),
    format: Format::Unparsed,
};

/// Every resource a `templates/` file can replace, in the order the replacements are reported.
const RESOURCES: [&Resource; 10] = [
    &TEAM_COLOR,
    &UNI_COLOR,
    &UNIFORM_PARAMETER_18,
    &UNIFORM_PARAMETER_19,
    &PLACEHOLDER_KIT,
    &BODY_SKELETON,
    &FACE_DIFF,
    &FCL_HAIR_SIM_FCLO,
    &DPFILELIST,
    &PLACEHOLDER_CPK,
];

/// An override in `templates/` that cannot be read, or one of the bins or the list that does
/// not parse:
/// `template_override_unreadable`'s context.
#[derive(Debug)]
pub(crate) struct Unreadable {
    /// The override file, or the folder when it cannot be listed.
    pub(crate) path: PathBuf,
    /// What failed.
    pub(crate) error: anyhow::Error,
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
    /// exactly as it is spelled, is not read. An override that cannot be read, an override of
    /// one of the bins or of `DpFileList.bin` that does not parse as it (whatever the run's
    /// version), or a folder that cannot be listed, is the error. The other overrides' bytes
    /// are not checked: one that does not decode fails where the embedded resource would be
    /// used.
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
            let read = fs::read(&path)
                .map_err(anyhow::Error::from)
                .and_then(|bytes| resource.format.check(bytes));
            let bytes = match read {
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

    /// The CPK file names of the official `DpFileList.bin`, in load order: the list a compile
    /// that deploys compares the installed one with.
    pub(crate) fn official_list(&self) -> Vec<String> {
        dpfl::entries(self.official_list_file()).expect(
            "the official list reads: an override is read as a list when it is read (`read`), \
             and the embedded one is checked by a test",
        )
    }

    /// The official `DpFileList.bin` itself, which `upgrade-dpfl` installs byte for byte.
    pub(crate) fn official_list_file(&self) -> &[u8] {
        self.bytes(&DPFILELIST)
    }

    /// The empty CPK `upgrade-dpfl` writes for an official entry with no file.
    pub(crate) fn placeholder_cpk(&self) -> &[u8] {
        self.bytes(&PLACEHOLDER_CPK)
    }
}

/// A command's resources (`Templates::read`), read before anything else, their findings
/// reported first through `events`. A `templates/` file that cannot be read is
/// `template_override_unreadable`, Fatal, and `None`: the command stops before it reads or
/// writes anything else, whatever the resource, because the file was put there on purpose
/// (`pipeline.md` "Resolved decisions", "Templates and fallback bins").
pub(crate) fn read_reported(ctx: &ToolContext, events: &mut RunEvents) -> Option<Templates> {
    match Templates::read(ctx.paths().data_dir.as_deref()) {
        Ok((templates, messages)) => {
            for message in messages {
                events.message(message);
            }
            Some(templates)
        }
        Err(unreadable) => {
            events.message(tool_message(
                Code::TemplateOverrideUnreadable,
                Scope::Run,
                Disposition::AbortRun,
                vec![
                    ("path", unreadable.path.display().to_string()),
                    ("error", format!("{:#}", unreadable.error)),
                ],
            ));
            None
        }
    }
}

/// The names of the entries of `folder`, as they are spelled: a Windows file system would open
/// `unicolor.bin` for `UniColor.bin`, so a resource's file is found in the listing, by its
/// exact name, rather than by opening its path. A folder that cannot be listed is the error.
fn file_names(folder: &Path) -> Result<BTreeSet<OsString>, Unreadable> {
    let unreadable = |error: std::io::Error| Unreadable {
        path: folder.to_owned(),
        error: error.into(),
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
    use crate::testing::{dpfilelist, scratch};

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
    fn every_resource(templates: &Templates) -> [&[u8]; 10] {
        [
            templates.team_color(),
            templates.uni_color(),
            templates.uniform_parameter_base(PesVersion::Pes18).unwrap(),
            templates.uniform_parameter_base(PesVersion::Pes21).unwrap(),
            templates.placeholder_kit(),
            templates.body_skeleton(),
            templates.face_diff(),
            templates.fcl_hair_sim(),
            templates.official_list_file(),
            templates.placeholder_cpk(),
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
        fs::write(folder.join("placeholder.cpk"), b"placeholder override").unwrap();
        fs::write(folder.join("face_diff.bin"), b"face diff override").unwrap();
        // One `UniColor.bin` record, which parses as the bin.
        let kit_colors = [1; 85];
        fs::write(folder.join("UniColor.bin"), kit_colors).unwrap();

        let (templates, messages) = Templates::read(Some(temp.path())).unwrap();

        let mut expected: [&[u8]; 10] = RESOURCES.map(|resource| resource.embedded);
        expected[1] = &kit_colors;
        expected[6] = b"face diff override";
        expected[9] = b"placeholder override";
        assert!(every_resource(&templates) == expected);
        let active = |name: &str| {
            tool_message(
                Code::TemplateOverrideActive,
                Scope::Run,
                Disposition::Keep,
                vec![("path", folder.join(name).display().to_string())],
            )
        };
        assert_eq!(
            messages,
            [
                active("UniColor.bin"),
                active("face_diff.bin"),
                active("placeholder.cpk")
            ]
        );
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

    #[test]
    fn a_bin_override_that_does_not_parse_is_the_error_naming_it() {
        let temp = scratch("templates_unparsable");
        let folder = temp.path().join("templates");
        fs::create_dir(&folder).unwrap();
        // One byte short of a whole 85-byte record.
        fs::write(folder.join("UniColor.bin"), [0; 84]).unwrap();

        let Err(unreadable) = Templates::read(Some(temp.path())) else {
            panic!("a bin override that does not parse must be the error");
        };
        assert_eq!(unreadable.path, folder.join("UniColor.bin"));
        assert_eq!(
            unreadable.error.to_string(),
            "UniColor.bin is 84 bytes, not a whole number of 85-byte records"
        );
    }

    #[test]
    fn every_bin_override_is_parsed_whatever_the_run_s_version() {
        for name in [
            "TeamColor.bin",
            "UniColor.bin",
            "UniformParameter18.bin",
            "UniformParameter19.bin",
        ] {
            let temp = scratch(&format!("templates_unparsable_{name}"));
            let folder = temp.path().join("templates");
            fs::create_dir(&folder).unwrap();
            // Seven bytes: no whole record of either color bin, and shorter than the eight
            // bytes of a `UniformParameter.bin`'s header.
            fs::write(folder.join(name), [0; 7]).unwrap();

            let Err(unreadable) = Templates::read(Some(temp.path())) else {
                panic!("{name}: a bin override that does not parse must be the error");
            };
            assert_eq!(unreadable.path, folder.join(name));
        }
    }

    #[test]
    fn the_embedded_official_list_is_the_text_list_s_53_entries_in_order() {
        let text = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../resources/templates/DpFileList.txt"),
        )
        .unwrap();
        let expected: Vec<&str> = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();

        let official = Templates::embedded().official_list();

        assert_eq!(official.len(), 53);
        assert_eq!(official[0], "4cc_01_db.cpk");
        assert_eq!(official[52], "4cc_99_test.cpk");
        assert_eq!(official, expected);
    }

    #[test]
    fn a_list_override_that_does_not_read_as_a_list_is_the_error_naming_it() {
        let temp = scratch("templates_list_unreadable");
        let folder = temp.path().join("templates");
        fs::create_dir(&folder).unwrap();
        fs::write(folder.join("DpFileList.bin"), [0; 15]).unwrap();

        let Err(unreadable) = Templates::read(Some(temp.path())) else {
            panic!("a list override that does not read as a list must be the error");
        };
        assert_eq!(unreadable.path, folder.join("DpFileList.bin"));
        assert_eq!(
            unreadable.error.to_string(),
            "the list is 15 bytes, shorter than its 16-byte header"
        );
    }

    #[test]
    fn a_list_override_replaces_the_official_list() {
        let temp = scratch("templates_list_override");
        let folder = temp.path().join("templates");
        fs::create_dir(&folder).unwrap();
        let names = ["4cc_08_bins.cpk", "4cc_61_midcup.cpk", "4cc_99_test.cpk"];
        fs::write(folder.join("DpFileList.bin"), dpfilelist(&names)).unwrap();

        let (templates, messages) = Templates::read(Some(temp.path())).unwrap();

        assert_eq!(templates.official_list(), names);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].code.code, "template_override_active");
    }
}
