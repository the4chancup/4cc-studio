//! The `.mtl` a pre-Fox `.model` uses (`model_format.md` "Pre-Fox: the `.mtl` a `.model`
//! uses"): each model uses exactly one, the first a search finds, and the generated `face.xml`
//! names it. The deep pass reports a model it finds none for, and the face task names the one
//! it finds, so the two never disagree.

use aesthetics_export::{FileDescriptor, FileKind, classify, common_link_name};
use vtree::ScopePath;

use crate::plan::roles::{common_file, file_stem, is_direct_root_folder_file};

/// The `.mtl` that `model` uses: `model` is a `.model` among `files`, the files of the model
/// folder at `folder`, or a `.common` link among them to a `.model` in the export's `Common/`
/// folder, whose files are `common`.
///
/// A `.model` is searched for in its own folder, then in the model folder (the player folder,
/// when the model sits in one of its subfolders). A link is searched for in its own
/// folder for a name-matched `.mtl` only (a local override of the shared model's materials),
/// then in `Common/` (where the model really is), then in its own folder, then in the model
/// folder; its stem is the linked model's (`legs` for `legs.model.common.txt`). Each folder is
/// searched for a name-matched `.mtl` (its stem, case-folded, starts or ends the model's stem:
/// `body.mtl` for `body_high.model`), then `materials.mtl`, then any `.mtl`, the first in
/// case-folded name order within a kind. A `.mtl.common` link counts as a `.mtl` of the linked
/// name in the folder holding it, and stands for the `Common/` file it names, which is what is
/// returned when it is the one found: a caller tells a Common `.mtl` from the folder's own by
/// its path (`is_direct_root_folder_file`). `None` when the search finds no `.mtl`: the model has
/// every material undefined (`model_material_undefined`).
pub(crate) fn mtl_for<'a>(
    model: &ScopePath,
    folder: &ScopePath,
    files: &'a [FileDescriptor],
    common: &'a [FileDescriptor],
) -> Option<&'a FileDescriptor> {
    let own_folder = model
        .parent()
        .expect("a model file sits in a folder: its model folder or a subfolder of it");
    let own = mtls_in(&own_folder, files, common);
    let Some(linked) = common_link_name(model.name()) else {
        let model_stem = vtree::fold_name(file_stem(model.name()));
        // A model directly in the model folder searches that folder twice, finding nothing
        // new.
        return first_of_kinds(&own, &model_stem)
            .or_else(|| first_of_kinds(&mtls_in(folder, files, common), &model_stem));
    };
    let model_stem = vtree::fold_name(file_stem(&linked));
    let in_common: Vec<Candidate> = common
        .iter()
        .filter(|file| file.kind == FileKind::Mtl && is_direct_root_folder_file(&file.path))
        .map(|file| Candidate::new(file.path.name(), false, file))
        .collect();
    name_matched(&own, &model_stem)
        .or_else(|| first_of_kinds(&sorted(in_common), &model_stem))
        .or_else(|| first_of_kinds(&own, &model_stem))
        .or_else(|| first_of_kinds(&mtls_in(folder, files, common), &model_stem))
}

/// A `.mtl` one folder offers a model: the name it goes by there and the file the model would
/// use, the `Common/` file a `.mtl.common` link names.
struct Candidate<'a> {
    /// The name, case-folded: what the search matches.
    folded: String,
    /// Whether it is a link: a folder's own `.mtl` comes before a link of its folded name, as
    /// a local file overrides a Common one (`model_format.md` "Link files (`.common`)").
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

/// The `.mtl` files directly in the folder `searched`, among `files`, each `.mtl.common` link
/// there standing for the `common` file it names, in the search's order (`sorted`). A link
/// naming no `Common/` file is not one: validation drops a player folder holding such a link,
/// and a shared folder's links are not resolved (they have no role there, and its tasks are
/// given no `Common/` files).
fn mtls_in<'a>(
    searched: &ScopePath,
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
            let linked = common_link_name(file.path.name())?;
            if classify(&linked) != FileKind::Mtl {
                return None;
            }
            let target = common_file(common, &linked)?;
            Some(Candidate::new(&linked, true, target))
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
        // With nothing in Common, the link's folder, then the model folder.
        assert_eq!(
            found_with(
                "face/legs.model.common",
                &["face/legs.model.common", "face/x.mtl", "legs.mtl"],
                &["legs.model"]
            )
            .as_deref(),
            Some("face/x.mtl")
        );
        assert_eq!(
            found_with(
                "face/legs.model.common",
                &["face/legs.model.common", "legs.mtl"],
                &["legs.model"]
            )
            .as_deref(),
            Some("legs.mtl")
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
