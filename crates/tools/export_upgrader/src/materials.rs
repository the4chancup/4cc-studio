//! Step 7's `.mtl` texture references into the team's `Common/` (`plans/
//! export_upgrader.md` step 7, `DECISIONS.md` 2026-10-18 "old `.mtl`
//! references keep their extension, and a Common reference no link can
//! spell is kept"): a reference naming a `Common/` file becomes the bare
//! name plus the `.common` link spelling it, and one no link can spell
//! stays as written, its `???` respelled the resolved team's ID, reported
//! `common_reference_kept`.

use std::collections::BTreeMap;

use aesthetics_export::{FileKind, classify, common_link_name};
use pes_model::format::mtl::MaterialSet;
use pes_model::ops::paths::{TexturePath, rewrite_texture_paths};
use studio_core::{Disposition, Message, Scope};
use teams_list::TeamId;

use crate::carried::under;
use crate::messages::{Code, INPUT, tool_message};
use crate::players::path_scope;
use crate::source::SourceFailure;

/// What step 7 makes of the migrated model folders' `.mtl` files: the ones
/// rewritten, the `.common` links their references need, and the findings
/// on the input.
#[derive(Debug, Default)]
pub(crate) struct MaterialRewrites {
    /// Rewritten `.mtl` files: output path and contents; `run` writes them
    /// instead of copying their input.
    pub(crate) written: Vec<(String, Vec<u8>)>,
    /// Empty `.common` link files to write, output paths.
    pub(crate) links: Vec<String>,
    /// Findings on the input, in input-path order.
    pub(crate) findings: Vec<Message>,
}

/// The game directory prefix the team's `Common/` stands in, matched in
/// any letter case.
const COMMON_PREFIX: &str = "model/character/uniform/common/";

/// The folded stem of a file name (`hair.dds` → `hair`, `Hair.PNG` the
/// same).
fn folded_stem(name: &str) -> String {
    let stem = name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name);
    vtree::fold_name(stem)
}

/// Whether the file `name` claims the folded texture stem `stem` in its
/// folder: a texture of that stem, or a `.common` link to one (a model's
/// link, `hair.model.common`, claims no texture stem).
fn claims_stem(name: &str, stem: &str) -> bool {
    if classify(name) == FileKind::Texture {
        return folded_stem(name) == stem;
    }
    common_link_name(name).is_some_and(|target| {
        classify(&target) == FileKind::Texture && folded_stem(&target) == stem
    })
}

/// A reference's `???` marker and `sub` (the path below the Common
/// directory, ending in `/` or empty) when its directory is
/// `model/character/uniform/common/` + exactly three characters + `/`, in
/// any letter case; `None` otherwise.
fn reference_into_common(directory: &str) -> Option<(&str, &str)> {
    let prefix = directory.get(..COMMON_PREFIX.len())?;
    if !prefix.eq_ignore_ascii_case(COMMON_PREFIX) {
        return None;
    }
    let (marker, sub) = directory[COMMON_PREFIX.len()..].split_once('/')?;
    (marker.chars().count() == 3).then_some((marker, sub))
}

/// The model folder an output path sits in (`Players/07 - A`,
/// `Boots/Crocs`, `Gloves/Mitts`) and the file's path below it, or `None`.
fn model_folder_of(output: &str) -> Option<(&str, &str)> {
    let (top, rest) = output.split_once('/')?;
    if !matches!(top, "Players" | "Boots" | "Gloves") {
        return None;
    }
    let (folder, rest) = rest.split_once('/')?;
    Some((&output[..top.len() + 1 + folder.len()], rest))
}

/// A reference `path` cannot spell: `common_reference_kept` on the `.mtl`,
/// its `???` respelled the team's ID in place (which itself counts as a
/// change, so the file is written).
fn keep_reference(
    path: &mut TexturePath,
    sub: &str,
    team: TeamId,
    input: &str,
    findings: &mut Vec<Message>,
) {
    findings.push(tool_message(
        Code::CommonReferenceKept,
        Scope::File {
            export_id: INPUT,
            path: path_scope(input),
        },
        Disposition::Keep,
        vec![("reference", format!("{}{}", path.directory, path.file_name))],
    ));
    path.directory = format!(
        "{}{:03}/{sub}",
        &path.directory[..COMMON_PREFIX.len()],
        team.get()
    );
}

/// What step 7 makes of the migrated model folders' `.mtl` files for a team
/// export: `copies` the player plan's (input, output) list, `common` the
/// input's files below `Common/` (input paths, any case of the folder).
pub(crate) fn material_references(
    copies: &[(String, String)],
    common: &[String],
    team: TeamId,
    mut read: impl FnMut(&str) -> Result<Vec<u8>, SourceFailure>,
) -> MaterialRewrites {
    let mut rewrites = MaterialRewrites::default();
    let mut failures: BTreeMap<String, Vec<Message>> = BTreeMap::new();
    let mut planned: BTreeMap<String, String> = BTreeMap::new();

    for (input, output) in copies {
        let Some((model_folder, rest)) = model_folder_of(output) else {
            continue;
        };
        let Some(name) = rest.rsplit('/').next() else {
            continue;
        };
        if classify(name) != FileKind::Mtl {
            continue;
        }
        // The copy loop's own read reports a failure; an `.mtl` that does
        // not parse is the compile's to report, copied as it is.
        let Ok(bytes) = read(input) else {
            continue;
        };
        let wrapped = wezlib::is_wrapped(&bytes);
        let Ok(mut set) = MaterialSet::read(&bytes) else {
            continue;
        };
        let mut file_findings = Vec::new();
        let changed = rewrite_texture_paths(&mut set, |path| {
            // Every other reference stays as written.
            let Some((_marker, sub)) = reference_into_common(&path.directory) else {
                return;
            };
            let sub = sub.to_owned();
            let stem = folded_stem(&path.file_name);
            // The `Common/` file whose path below `Common/` is `sub` + a
            // name of that stem, any extension.
            let Some(common_rest) = common.iter().find_map(|file| {
                let rest = under(file, "common")?;
                let (folder, name) = match rest.rsplit_once('/') {
                    Some((folder, name)) => (format!("{folder}/"), name),
                    None => (String::new(), rest),
                };
                (vtree::fold_name(&folder) == vtree::fold_name(&sub) && folded_stem(name) == stem)
                    .then_some(rest)
            }) else {
                keep_reference(path, &sub, team, input, &mut file_findings);
                return;
            };
            // A `.mtl` in a subfolder naming a Common subfolder: the path
            // below it and the link mirroring `Common/` cannot agree.
            if !sub.is_empty() && rest.contains('/') {
                keep_reference(path, &sub, team, input, &mut file_findings);
                return;
            }
            // The clash check: the folders a bare name looks in, nearest
            // first (the `.mtl`'s folder up to the model folder), or for a
            // `sub` the one `model folder/sub` folder.
            let mtl_folder = match rest.rsplit_once('/') {
                Some((dir, _)) => format!("{model_folder}/{dir}"),
                None => model_folder.to_owned(),
            };
            let mut folders = Vec::new();
            if sub.is_empty() {
                let mut folder = mtl_folder.as_str();
                loop {
                    folders.push(folder.to_owned());
                    if folder == model_folder {
                        break;
                    }
                    folder = folder
                        .rsplit_once('/')
                        .map(|(parent, _)| parent)
                        .expect("a folder below the model folder has a parent");
                }
            } else {
                folders.push(format!("{model_folder}/{sub}"));
            }
            let clashing = |folder: &str| {
                let folder = vtree::fold_name(folder.trim_end_matches('/'));
                copies.iter().any(|(_, output)| {
                    let Some((parent, name)) = output.rsplit_once('/') else {
                        return false;
                    };
                    if vtree::fold_name(parent) != folder {
                        return false;
                    }
                    claims_stem(name, &stem)
                }) || planned.iter().any(|(link, rest)| {
                    *rest != common_rest
                        && link.rsplit_once('/').is_some_and(|(parent, name)| {
                            vtree::fold_name(parent) == folder && claims_stem(name, &stem)
                        })
                })
            };
            if folders.iter().any(|folder| clashing(folder)) {
                keep_reference(path, &sub, team, input, &mut file_findings);
                return;
            }
            // `./` + `sub` + the name as written, and the link for the
            // `Common/` file's own name, planned once.
            path.directory = format!("./{sub}");
            planned.insert(
                format!("{model_folder}/{common_rest}.common"),
                common_rest.to_owned(),
            );
        });
        if changed > 0 {
            let out = set.write();
            rewrites.written.push((
                output.clone(),
                if wrapped { wezlib::compress(&out) } else { out },
            ));
        }
        if !file_findings.is_empty() {
            failures
                .entry(input.clone())
                .or_default()
                .extend(file_findings);
        }
    }
    rewrites.links = planned.keys().cloned().collect();

    for (input, _) in copies {
        if let Some(messages) = failures.remove(input.as_str()) {
            rewrites.findings.extend(messages);
        }
    }
    rewrites.findings.extend(failures.into_values().flatten());
    rewrites
}

#[cfg(test)]
mod tests {
    use studio_core::Scope;

    use super::*;
    use crate::messages::INPUT;
    use crate::players::path_scope;

    /// The resolved team, 714.
    fn team() -> TeamId {
        TeamId::new(714).unwrap()
    }

    /// A `.mtl` of one material holding one sampler per `paths` entry.
    fn mtl(paths: &[&str]) -> Vec<u8> {
        let samplers: String = paths
            .iter()
            .map(|path| {
                format!("        <sampler name=\"DiffuseMap\" path=\"{path}\" srgb=\"1\" minfilter=\"linear\" magfilter=\"linear\" uaddr=\"clamp\" vaddr=\"clamp\"/>\n")
            })
            .collect();
        format!(
            "<materialset>\n    <material name=\"face_phong\" shader=\"Wrinkle_Face\">\n{samplers}    </material>\n</materialset>\n"
        )
        .into_bytes()
    }

    /// `material_references` over `copies` and `common`, each `.mtl`'s bytes
    /// behind `read`; a listed copy without bytes is a test bug.
    fn plan(copies: &[(&str, &str)], common: &[&str], bytes: &[(&str, &[u8])]) -> MaterialRewrites {
        let copies: Vec<(String, String)> = copies
            .iter()
            .map(|(from, to)| ((*from).to_owned(), (*to).to_owned()))
            .collect();
        let common: Vec<String> = common.iter().map(|file| (*file).to_owned()).collect();
        let bytes: BTreeMap<&str, &[u8]> = bytes.iter().copied().collect();
        material_references(&copies, &common, team(), |path| {
            bytes
                .get(path)
                .map(|bytes| Ok(bytes.to_vec()))
                .unwrap_or_else(|| panic!("a listed copy has no bytes: {path}"))
        })
    }

    /// The one output path's `.mtl` text, parsed back.
    fn written_text(rewrites: &MaterialRewrites, output: &str) -> String {
        let (_, bytes) = rewrites
            .written
            .iter()
            .find(|(path, _)| path == output)
            .expect("the .mtl was rewritten");
        String::from_utf8(bytes.clone()).unwrap()
    }

    #[test]
    fn a_reference_into_common_becomes_the_bare_name_and_the_link() {
        let face = mtl(&["model/character/uniform/common/XXX/hair.dds"]);
        let rewrites = plan(
            &[
                ("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl"),
                ("Faces/XXX07 - A/face.fmdl", "Players/07 - A/face.fmdl"),
            ],
            &["Common/hair.dds"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        let out = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(out.contains("./hair.dds"), "{out}");
        assert_eq!(
            rewrites.links,
            ["Players/07 - A/hair.dds.common".to_owned()]
        );
        // The marker's own spelling and case likewise.
        let face = mtl(&["MODEL/Character/Uniform/Common/714/Hair.DDS"]);
        let rewrites = plan(
            &[("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl")],
            &["Common/hair.dds"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        let out = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(out.contains("./Hair.DDS"), "{out}");
        // `Common/hair.png` is found by stem and the link names its file.
        let rewrites = plan(
            &[("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl")],
            &["Common/hair.png"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        assert_eq!(
            rewrites.links,
            ["Players/07 - A/hair.png.common".to_owned()]
        );
    }

    #[test]
    fn a_common_subfolder_reference_links_at_the_path_below_the_folder() {
        let face = mtl(&["model/character/uniform/common/XXX/jessie/hair.dds"]);
        let rewrites = plan(
            &[
                ("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl"),
                (
                    "Faces/XXX07 - A/realkit/x.mtl",
                    "Players/07 - A/realkit/x.mtl",
                ),
            ],
            &["Common/jessie/hair.dds"],
            &[
                ("Faces/XXX07 - A/face.mtl", &face),
                ("Faces/XXX07 - A/realkit/x.mtl", &face),
            ],
        );
        // The root `.mtl` gets the path below the model folder and the
        // mirrored link; the subfolder `.mtl`'s is kept, `???` respelled.
        let out = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(out.contains("./jessie/hair.dds"), "{out}");
        assert_eq!(
            rewrites.links,
            ["Players/07 - A/jessie/hair.dds.common".to_owned()]
        );
        let out = written_text(&rewrites, "Players/07 - A/realkit/x.mtl");
        assert!(
            out.contains("model/character/uniform/common/714/jessie/hair.dds"),
            "{out}"
        );
        let kept = rewrites
            .findings
            .iter()
            .find(|m| m.code.code.as_ref() == "common_reference_kept")
            .expect("common_reference_kept expected");
        assert_eq!(
            kept.context,
            vec![(
                "reference".to_owned(),
                "model/character/uniform/common/XXX/jessie/hair.dds".to_owned()
            )]
        );
        assert_eq!(
            kept.scope,
            Scope::File {
                export_id: INPUT,
                path: path_scope("Faces/XXX07 - A/realkit/x.mtl")
            }
        );
    }

    #[test]
    fn a_folder_file_of_the_stem_keeps_the_reference() {
        // `Players/07 - A/` already holds `hair.png`: the link would clash
        // or lose to the nearer file — kept.
        let face = mtl(&["model/character/uniform/common/XXX/hair.dds"]);
        let rewrites = plan(
            &[
                ("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl"),
                ("Faces/XXX07 - A/hair.png", "Players/07 - A/hair.png"),
            ],
            &["Common/hair.dds"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        let out = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(
            out.contains("model/character/uniform/common/714/hair.dds"),
            "{out}"
        );
        assert!(rewrites.links.is_empty());
        // From a subfolder `.mtl` the bare name looks up to the model folder,
        // so the root's `hair.png` keeps it too.
        let rewrites = plan(
            &[
                (
                    "Faces/XXX07 - A/realkit/apc.mtl",
                    "Players/07 - A/realkit/apc.mtl",
                ),
                ("Faces/XXX07 - A/hair.png", "Players/07 - A/hair.png"),
            ],
            &["Common/hair.dds"],
            &[("Faces/XXX07 - A/realkit/apc.mtl", &face)],
        );
        let out = written_text(&rewrites, "Players/07 - A/realkit/apc.mtl");
        assert!(
            out.contains("model/character/uniform/common/714/hair.dds"),
            "{out}"
        );
        assert!(rewrites.links.is_empty());
        // A model's link of that stem claims no texture: the link is made.
        let rewrites = plan(
            &[
                ("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl"),
                (
                    "Faces/XXX07 - A/hair.model.common",
                    "Players/07 - A/hair.model.common",
                ),
            ],
            &["Common/hair.dds"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        assert_eq!(
            rewrites.links,
            ["Players/07 - A/hair.dds.common".to_owned()]
        );
        // Two `.mtl` files naming one `Common/` file share its one link.
        let two = plan(
            &[
                ("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl"),
                ("Faces/XXX07 - A/other.mtl", "Players/07 - A/other.mtl"),
            ],
            &["Common/hair.dds"],
            &[
                ("Faces/XXX07 - A/face.mtl", &face),
                ("Faces/XXX07 - A/other.mtl", &face),
            ],
        );
        assert_eq!(two.links, ["Players/07 - A/hair.dds.common".to_owned()]);
        for output in ["Players/07 - A/face.mtl", "Players/07 - A/other.mtl"] {
            assert!(
                written_text(&two, output).contains("./hair.dds"),
                "{output}"
            );
        }
    }

    #[test]
    fn an_mtl_naming_nothing_or_unparsable_stays_as_it_is() {
        // `./face.dds` and `model/character/face/common/` are not the team's
        // Common — nothing changes, so nothing is written.
        let unchanged = mtl(&["./face.dds", "model/character/face/common/face_eyelash.dds"]);
        let bad = b"mtl".to_vec();
        let plan = plan(
            &[
                ("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl"),
                ("Faces/XXX07 - A/bad.mtl", "Players/07 - A/bad.mtl"),
            ],
            &["Common/hair.dds"],
            &[
                ("Faces/XXX07 - A/face.mtl", &unchanged),
                ("Faces/XXX07 - A/bad.mtl", &bad),
            ],
        );
        assert!(plan.written.is_empty());
        assert!(plan.links.is_empty());
        assert!(plan.findings.is_empty());
    }

    #[test]
    fn the_common_file_must_match_both_folder_and_stem() {
        // `Common/` holds `beard.dds` (the folder, not the stem) and
        // `jessie/hair.dds` (the stem, not the folder): `hair` is kept.
        let face = mtl(&["model/character/uniform/common/XXX/hair.dds"]);
        let rewrites = plan(
            &[("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl")],
            &["Common/beard.dds", "Common/jessie/hair.dds"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        let out = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(
            out.contains("model/character/uniform/common/714/hair.dds"),
            "{out}"
        );
        assert!(rewrites.links.is_empty());
    }

    #[test]
    fn two_common_files_named_from_one_folder_both_link() {
        let face = mtl(&[
            "model/character/uniform/common/XXX/hair.dds",
            "model/character/uniform/common/XXX/beard.dds",
        ]);
        let rewrites = plan(
            &[("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl")],
            &["Common/beard.dds", "Common/hair.dds"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        assert_eq!(
            rewrites.links,
            [
                "Players/07 - A/beard.dds.common".to_owned(),
                "Players/07 - A/hair.dds.common".to_owned()
            ]
        );
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn a_shared_folder_mtl_links_inside_it() {
        let boots = mtl(&["model/character/uniform/common/XXX/hair.dds"]);
        let rewrites = plan(
            &[("Boots/k0430 - Crocs/boots.mtl", "Boots/Crocs/boots.mtl")],
            &["Common/hair.dds"],
            &[("Boots/k0430 - Crocs/boots.mtl", &boots)],
        );
        let out = written_text(&rewrites, "Boots/Crocs/boots.mtl");
        assert!(out.contains("./hair.dds"), "{out}");
        assert_eq!(rewrites.links, ["Boots/Crocs/hair.dds.common".to_owned()]);
    }
}
