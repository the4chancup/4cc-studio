//! The `.mtl` a pre-Fox `.model` uses (`model_format.md` "Pre-Fox: the `.mtl` a `.model`
//! uses"): each model uses exactly one, the first a search finds, and the generated `face.xml`
//! names it. The deep pass reports a model it finds none for, and the face task names the one
//! it finds, so the two never disagree.

use aesthetics_export::{FileDescriptor, FileKind, classify, common_link_target};
use vtree::ScopePath;

use crate::face_xml::{ORAL_PREFIX, WIN32_SUFFIX};
use crate::plan::roles::{below_common, common_file, file_stem};
use crate::texture_lookup::split;

/// The `.mtl` that `model` uses: `model` is a `.model` among `files`, the files of the model
/// folder at `folder`, or a `.common` link among them to a `.model` in the export's `Common/`
/// folder, whose files are `common`.
///
/// A `.model` is searched for in its own folder, then in the model folder (the player folder,
/// when the model sits in one of its subfolders). A link is searched for in its own
/// folder for a name-matched `.mtl` only (a local override of the shared model's materials),
/// then in the linked model's folder in `Common/` (where the model really is: `Common/jessie/`
/// for `jessie/legs.model.common`, the link standing for the `Common/` file at its own path,
/// `common_link_target`), then in its own folder, then in the model folder; its stem is the
/// linked model's (`legs` for `legs.model.common.txt`). Each folder is
/// searched for a name-matched `.mtl` (its stem, case-folded, starts or ends the model's stem:
/// `body.mtl` for `body_high.model`), then `materials.mtl`, then any `.mtl`, the first in
/// case-folded name order within a kind. A `.mtl.common` link counts as a `.mtl` of the linked
/// name in the folder holding it, and stands for the `Common/` file at its path, which is what
/// is returned when it is the one found: a caller tells a Common `.mtl` from the folder's own by
/// its path (`is_common_file`). The model's stem the name match reads is `match_stem`'s. `None`
/// when the search finds no `.mtl`: the model has every material undefined
/// (`model_material_undefined`).
pub(crate) fn mtl_for<'a>(
    model: &ScopePath,
    folder: &ScopePath,
    files: &'a [FileDescriptor],
    common: &'a [FileDescriptor],
) -> Option<&'a FileDescriptor> {
    let own_folder = model
        .parent()
        .expect("a model file sits in a folder: its model folder or a subfolder of it");
    let own = mtls_in(&own_folder, folder, files, common);
    let Some(linked) = common_link_target(model, folder) else {
        let model_stem = match_stem(model.name());
        // A model directly in the model folder searches that folder twice, finding nothing
        // new.
        return first_of_kinds(&own, &model_stem)
            .or_else(|| first_of_kinds(&mtls_in(folder, folder, files, common), &model_stem));
    };
    let (linked_directory, linked_name) = split(&linked);
    let model_stem = match_stem(linked_name);
    let in_common: Vec<Candidate> = common
        .iter()
        .filter(|file| {
            let (directory, _) = split(below_common(&file.path));
            file.kind == FileKind::Mtl
                && vtree::fold_name(directory) == vtree::fold_name(linked_directory)
        })
        .map(|file| Candidate::new(file.path.name(), false, file))
        .collect();
    name_matched(&own, &model_stem)
        .or_else(|| first_of_kinds(&sorted(in_common), &model_stem))
        .or_else(|| first_of_kinds(&own, &model_stem))
        .or_else(|| first_of_kinds(&mtls_in(folder, folder, files, common), &model_stem))
}

/// The stem the name match compares `.mtl` stems against for the model named `name`: its file
/// stem, folded, without the `oral_`/`_win32` affixes a model packed under its in-game name
/// carries. A member names a part `oral_X_win32.model` as the game does and its `.mtl`
/// `X.mtl`. Only when the stem carries both affixes: one alone is part of the name, and a stem
/// that would strip to nothing (`oral__win32`) keeps its literal reading.
fn match_stem(name: &str) -> String {
    let stem = vtree::fold_name(file_stem(name));
    match stem
        .strip_prefix(ORAL_PREFIX)
        .and_then(|rest| rest.strip_suffix(WIN32_SUFFIX))
    {
        Some(inner) if !inner.is_empty() => inner.to_owned(),
        _ => stem,
    }
}

/// A `.mtl` one folder offers a model: the name it goes by there and the file the model would
/// use, the `Common/` file a `.mtl.common` link names.
struct Candidate<'a> {
    /// The name, case-folded: what the search matches.
    folded: String,
    /// Whether it is a link: a folder's own `.mtl` comes before a link of its folded name, as
    /// a local file overrides a Common one (`spec/model_format.md` "Link files (`.common`)").
    linked: bool,
    /// The name as spelled, which breaks a tie between two names that fold alike.
    name: String,
    /// The file the model uses when this one is found.
    file: &'a FileDescriptor,
}

impl<'a> Candidate<'a> {
    /// The `.mtl` named `name` in its folder, a link's when `linked`, that stands for `file`.
    fn new(name: &str, linked: bool, file: &'a FileDescriptor) -> Candidate<'a> {
        Candidate {
            folded: vtree::fold_name(name),
            linked,
            name: name.to_owned(),
            file,
        }
    }
}

/// The `.mtl` files directly in the folder `searched`, among `files` (those of the model folder
/// at `folder`), each `.mtl.common` link there standing for the `common` file at its path
/// (`common_link_target`), in the search's order (`sorted`). A link naming no `Common/` file is
/// not one: validation drops a player folder holding such a link, and a shared folder's
/// `.mtl` link has no role on pre-Fox, its tasks' search given no `Common/` files to
/// resolve it against.
fn mtls_in<'a>(
    searched: &ScopePath,
    folder: &ScopePath,
    files: &'a [FileDescriptor],
    common: &'a [FileDescriptor],
) -> Vec<Candidate<'a>> {
    let found = files
        .iter()
        .filter(|file| file.path.parent().as_ref() == Some(searched))
        .filter_map(|file| {
            if file.kind == FileKind::Mtl {
                return Some(Candidate::new(file.path.name(), false, file));
            }
            if file.kind != FileKind::CommonLink {
                return None;
            }
            let linked = common_link_target(&file.path, folder)?;
            let (_, linked_name) = split(&linked);
            if classify(linked_name) != FileKind::Mtl {
                return None;
            }
            let target = common_file(common, &linked)?;
            Some(Candidate::new(linked_name, true, target))
        })
        .collect();
    sorted(found)
}

/// `candidates` in the search's order: by name, case-folded, a folder's own file before a link
/// of its name, then by name as spelled.
fn sorted(mut candidates: Vec<Candidate>) -> Vec<Candidate> {
    candidates.sort_by(|a, b| (&a.folded, a.linked, &a.name).cmp(&(&b.folded, b.linked, &b.name)));
    candidates
}

/// The `.mtl` among one folder's `candidates`, in the search's order, that a model of the
/// folded stem `model_stem` takes from it: a name-matched one, then `materials.mtl`, then any.
fn first_of_kinds<'a>(
    candidates: &[Candidate<'a>],
    model_stem: &str,
) -> Option<&'a FileDescriptor> {
    name_matched(candidates, model_stem)
        .or_else(|| {
            candidates
                .iter()
                .find(|candidate| candidate.folded == "materials.mtl")
                .map(|candidate| candidate.file)
        })
        .or_else(|| candidates.first().map(|candidate| candidate.file))
}

/// The first of `candidates` whose stem starts or ends `model_stem`, both case-folded.
fn name_matched<'a>(candidates: &[Candidate<'a>], model_stem: &str) -> Option<&'a FileDescriptor> {
    candidates
        .iter()
        .find(|candidate| {
            let stem = file_stem(&candidate.folded);
            model_stem.starts_with(stem) || model_stem.ends_with(stem)
        })
        .map(|candidate| candidate.file)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The folder `Players/05 - A`.
    fn folder() -> ScopePath {
        ScopePath::new("Players/05 - A").unwrap()
    }

    /// The files at `paths`.
    fn descriptors(paths: impl Iterator<Item = String>) -> Vec<FileDescriptor> {
        paths
            .map(|path| {
                let path = ScopePath::new(&path).unwrap();
                FileDescriptor {
                    size: 0,
                    kind: aesthetics_export::classify(path.name()),
                    source: path.clone(),
                    path,
                }
            })
            .collect()
    }

    /// The `.mtl` the model `model` uses among `names` of `Players/05 - A` and `common` of
    /// `Common/`: its name below the folder, or its export path when it is a Common file.
    fn found_with(model: &str, names: &[&str], common: &[&str]) -> Option<String> {
        let files = descriptors(names.iter().map(|name| format!("Players/05 - A/{name}")));
        let common = descriptors(common.iter().map(|name| format!("Common/{name}")));
        let model = ScopePath::new(&format!("Players/05 - A/{model}")).unwrap();
        mtl_for(&model, &folder(), &files, &common).map(|file| {
            let path = file.path.as_str();
            path.strip_prefix("Players/05 - A/")
                .unwrap_or(path)
                .to_owned()
        })
    }

    /// The name, below the folder, of the `.mtl` the model `model` uses among `names`, with
    /// nothing in `Common/`.
    fn found(model: &str, names: &[&str]) -> Option<String> {
        found_with(model, names, &[])
    }

    #[test]
    fn in_one_folder_a_name_match_beats_materials_mtl_which_beats_any() {
        let all = ["hat.model", "a.mtl", "materials.mtl", "hat.mtl"];
        assert_eq!(found("hat.model", &all).as_deref(), Some("hat.mtl"));
        assert_eq!(
            found("hat.model", &all[..3]).as_deref(),
            Some("materials.mtl")
        );
        assert_eq!(found("hat.model", &all[..2]).as_deref(), Some("a.mtl"));
    }

    #[test]
    fn the_model_s_own_folder_is_searched_for_every_kind_before_the_model_folder() {
        // An "any" `.mtl` in `face/` is found before the folder's own name-matched one.
        assert_eq!(
            found(
                "face/hat.model",
                &["face/hat.model", "face/x.mtl", "hat.mtl"]
            )
            .as_deref(),
            Some("face/x.mtl")
        );
        // With none in `face/`, the model folder's.
        assert_eq!(
            found(
                "face/hat.model",
                &["face/hat.model", "hat.mtl", "boots/hat.mtl"]
            )
            .as_deref(),
            Some("hat.mtl")
        );
    }

    #[test]
    fn within_a_kind_the_first_in_case_folded_name_order_wins() {
        assert_eq!(
            found("hat.model", &["hat.model", "c.mtl", "B.mtl"]).as_deref(),
            Some("B.mtl")
        );
        assert_eq!(
            found("hat.model", &["hat.model", "B.mtl", "a.mtl"]).as_deref(),
            Some("a.mtl")
        );
    }

    #[test]
    fn a_model_with_no_mtl_in_either_folder_has_none() {
        assert_eq!(found("hat.model", &["hat.model"]), None);
        assert_eq!(
            found("face/hat.model", &["face/hat.model", "boots/hat.mtl"]),
            None
        );
    }

    #[test]
    fn a_name_match_starts_or_ends_the_model_s_stem() {
        for mtl in ["body.mtl", "high.mtl", "Body.mtl"] {
            assert_eq!(
                found("body_high.model", &["body_high.model", "a.mtl", mtl]).as_deref(),
                Some(mtl),
                "{mtl}"
            );
        }
    }

    #[test]
    fn a_packed_name_s_mtl_is_matched_by_the_stem_inside_its_affixes() {
        // The VG export's `Common/`: `oral_fbm_u_win32.model` beside `apc.mtl` and
        // the `fbm.mtl` defining its `fbm_u` material. Read without the packed
        // name's `oral_`/`_win32` affixes the stem is `fbm_u`, which `fbm.mtl`
        // name-matches; read literally the stem matches nothing and `apc.mtl`,
        // the first "any", would be the model's `.mtl`.
        assert_eq!(
            found(
                "oral_fbm_u_win32.model",
                &["oral_fbm_u_win32.model", "apc.mtl", "fbm.mtl"]
            )
            .as_deref(),
            Some("fbm.mtl")
        );
        // `parts_body.mtl` beside `materials.mtl`: the name match wins.
        assert_eq!(
            found(
                "oral_parts_body_win32.model",
                &[
                    "oral_parts_body_win32.model",
                    "parts_body.mtl",
                    "materials.mtl"
                ]
            )
            .as_deref(),
            Some("parts_body.mtl")
        );
        // Letter case aside.
        assert_eq!(
            found(
                "ORAL_FBM_U_WIN32.model",
                &["ORAL_FBM_U_WIN32.model", "apc.mtl", "fbm.mtl"]
            )
            .as_deref(),
            Some("fbm.mtl")
        );
        // Through a link, the linked model's stem is read the same way.
        assert_eq!(
            found_with(
                "oral_fbm_u_win32.model.common",
                &["oral_fbm_u_win32.model.common"],
                &["oral_fbm_u_win32.model", "apc.mtl", "fbm.mtl"]
            )
            .as_deref(),
            Some("Common/fbm.mtl")
        );
    }

    #[test]
    fn a_stem_with_one_affix_alone_matches_as_it_is() {
        // `oral_hat` carries `oral_` without `_win32` and stays `oral_hat`:
        // `oral.mtl` name-matches it. A stem stripped on the prefix alone would
        // be `hat`, which `oral.mtl` neither starts nor ends, so this is what
        // pins the both-affixes rule.
        assert_eq!(
            found(
                "oral_hat.model",
                &["oral_hat.model", "oral.mtl", "materials.mtl"]
            )
            .as_deref(),
            Some("oral.mtl")
        );
        // `hat.mtl` is no proof either way: the inner stem is the literal
        // stem's tail, so it name-matches `oral_hat` without a strip.
        assert_eq!(
            found(
                "oral_hat.model",
                &["oral_hat.model", "hat.mtl", "materials.mtl"]
            )
            .as_deref(),
            Some("hat.mtl")
        );
        // `hat_win32` carries `_win32` without `oral_` and stays `hat_win32`:
        // `win32.mtl` name-matches it; `hat.mtl` starts the literal stem anyway.
        assert_eq!(
            found(
                "hat_win32.model",
                &["hat_win32.model", "win32.mtl", "materials.mtl"]
            )
            .as_deref(),
            Some("win32.mtl")
        );
        assert_eq!(
            found("hat_win32.model", &["hat_win32.model", "hat.mtl"]).as_deref(),
            Some("hat.mtl")
        );
    }

    #[test]
    fn a_material_link_stands_for_the_common_mtl_it_names_in_its_folder() {
        assert_eq!(
            found_with("hat.model", &["hat.model", "x.mtl.common"], &["x.mtl"]).as_deref(),
            Some("Common/x.mtl")
        );
        // Under its linked name: a name match through a link beats a folder's own "any".
        assert_eq!(
            found_with(
                "hat.model",
                &["hat.model", "a.mtl", "Hat.mtl.common.txt"],
                &["hat.mtl"]
            )
            .as_deref(),
            Some("Common/hat.mtl")
        );
        // A link naming nothing in `Common/` is not a `.mtl` of the folder.
        assert_eq!(
            found_with("hat.model", &["hat.model", "x.mtl.common"], &[]),
            None
        );
    }

    #[test]
    fn a_folder_s_own_mtl_comes_before_a_link_of_its_name() {
        // `hat.mtl` and `hat.mtl.common` both go by `hat.mtl`: the local file overrides the
        // Common one, as a local file overrides a link everywhere.
        assert_eq!(
            found_with(
                "hat.model",
                &["hat.model", "hat.mtl.common", "hat.mtl"],
                &["hat.mtl"]
            )
            .as_deref(),
            Some("hat.mtl")
        );
    }

    #[test]
    fn a_model_link_looks_in_common_before_its_own_folder_s_other_kinds() {
        assert_eq!(
            found_with(
                "legs.model.common",
                &["legs.model.common", "materials.mtl"],
                &["legs.model", "legs.mtl"]
            )
            .as_deref(),
            Some("Common/legs.mtl")
        );
        // Common's "any" too comes before the link's folder's `materials.mtl`.
        assert_eq!(
            found_with(
                "legs.model.common",
                &["legs.model.common", "materials.mtl"],
                &["legs.model", "cloth.mtl"]
            )
            .as_deref(),
            Some("Common/cloth.mtl")
        );
        // With nothing in the linked model's folder in Common, the link's folder, then the
        // model folder: a `.mtl` directly in `Common/` is not the linked model's.
        assert_eq!(
            found_with(
                "face/legs.model.common",
                &["face/legs.model.common", "face/x.mtl", "legs.mtl"],
                &["face/legs.model", "legs.mtl"]
            )
            .as_deref(),
            Some("face/x.mtl")
        );
        assert_eq!(
            found_with(
                "face/legs.model.common",
                &["face/legs.model.common", "legs.mtl"],
                &["face/legs.model"]
            )
            .as_deref(),
            Some("legs.mtl")
        );
    }

    #[test]
    fn a_model_link_below_a_subfolder_looks_in_its_model_s_folder_in_common() {
        // `face/legs.model.common` links `Common/face/legs.model`, whose `.mtl` is beside it.
        assert_eq!(
            found_with(
                "face/legs.model.common",
                &["face/legs.model.common", "materials.mtl"],
                &["face/legs.model", "face/legs.mtl", "legs.mtl"]
            )
            .as_deref(),
            Some("Common/face/legs.mtl")
        );
        // A material link below a subfolder stands for the `Common/` file at its own path.
        assert_eq!(
            found_with(
                "face/hat.model",
                &["face/hat.model", "face/hat.mtl.common"],
                &["hat.mtl", "face/hat.mtl"]
            )
            .as_deref(),
            Some("Common/face/hat.mtl")
        );
    }

    #[test]
    fn a_name_matched_mtl_beside_a_model_link_overrides_the_common_one() {
        assert_eq!(
            found_with(
                "legs.model.common",
                &["legs.model.common", "legs.mtl", "materials.mtl"],
                &["legs.model", "legs.mtl"]
            )
            .as_deref(),
            Some("legs.mtl")
        );
    }

    #[test]
    fn a_model_link_matches_by_the_linked_model_s_stem() {
        // By `legs`, `common.mtl` is no name match, so Common's `.mtl` is found; by the link's
        // own stem, `legs.model.common`, it would be.
        assert_eq!(
            found_with(
                "legs.model.common.txt",
                &["legs.model.common.txt", "common.mtl"],
                &["legs.model", "cloth.mtl"]
            )
            .as_deref(),
            Some("Common/cloth.mtl")
        );
    }

    #[test]
    fn a_model_link_with_no_mtl_anywhere_has_none() {
        assert_eq!(
            found_with("legs.model.common", &["legs.model.common"], &["legs.model"]),
            None
        );
    }
}
