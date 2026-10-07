//! The `.mtl` a pre-Fox `.model` uses (`model_format.md` "Pre-Fox: the `.mtl` a `.model`
//! uses"): each model uses exactly one, the first a search finds, and the generated `face.xml`
//! names it. The deep pass reports a model it finds none for, and the face task names the one
//! it finds, so the two never disagree.

use aesthetics_export::{FileDescriptor, FileKind};
use vtree::ScopePath;

use crate::plan::subset::file_stem;

/// The `.mtl` among `files`, the files of the model folder at `folder`, that the `.model` at
/// `model`, one of them, uses: searched in the model's own folder, then in the model folder
/// (the player folder, when the model sits in one of its reserved subfolders), each for a
/// name-matched `.mtl` (its stem, case-folded, starts or ends the model's stem:
/// `body.mtl` for `body_high.model`), then `materials.mtl`, then any `.mtl`, the first in
/// case-folded name order within a kind. `None` when neither folder holds a `.mtl`: the model
/// has every material undefined (`model_material_undefined`).
pub(crate) fn mtl_for<'a>(
    model: &ScopePath,
    folder: &ScopePath,
    files: &'a [FileDescriptor],
) -> Option<&'a FileDescriptor> {
    let model_stem = vtree::fold_name(file_stem(model.name()));
    let own_folder = model
        .parent()
        .expect("a model file sits in a folder: its model folder or a reserved subfolder");
    // A model directly in the model folder searches that folder twice, finding nothing new.
    [&own_folder, folder]
        .into_iter()
        .find_map(|searched| mtl_in(searched, &model_stem, files))
}

/// The `.mtl` directly in the folder `searched`, among `files`, that a model of the folded
/// stem `model_stem` takes from it: a name-matched one, then `materials.mtl`, then any, the
/// first in case-folded name order within a kind (`mtl_for`).
fn mtl_in<'a>(
    searched: &ScopePath,
    model_stem: &str,
    files: &'a [FileDescriptor],
) -> Option<&'a FileDescriptor> {
    let mut mtls: Vec<(String, &FileDescriptor)> = files
        .iter()
        .filter(|file| file.kind == FileKind::Mtl && file.path.parent().as_ref() == Some(searched))
        .map(|file| (vtree::fold_name(file.path.name()), file))
        .collect();
    // The name as spelled breaks a tie between two names that fold alike.
    mtls.sort_by(|(a, a_file), (b, b_file)| {
        a.cmp(b).then(a_file.path.name().cmp(b_file.path.name()))
    });
    let name_matched = mtls.iter().find(|(name, _)| {
        let stem = file_stem(name);
        model_stem.starts_with(stem) || model_stem.ends_with(stem)
    });
    let materials = || mtls.iter().find(|(name, _)| name == "materials.mtl");
    name_matched
        .or_else(materials)
        .or_else(|| mtls.first())
        .map(|(_, file)| *file)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The folder `Players/05 - A`.
    fn folder() -> ScopePath {
        ScopePath::new("Players/05 - A").unwrap()
    }

    /// The files `names` of `Players/05 - A`.
    fn files(names: &[&str]) -> Vec<FileDescriptor> {
        names
            .iter()
            .map(|name| {
                let path = ScopePath::new(&format!("Players/05 - A/{name}")).unwrap();
                FileDescriptor {
                    size: 0,
                    kind: aesthetics_export::classify(path.name()),
                    source: path.clone(),
                    path,
                }
            })
            .collect()
    }

    /// The name, below the folder, of the `.mtl` the model `model` uses among `names`.
    fn found(model: &str, names: &[&str]) -> Option<String> {
        let files = files(names);
        let model = ScopePath::new(&format!("Players/05 - A/{model}")).unwrap();
        mtl_for(&model, &folder(), &files).map(|file| {
            file.path
                .as_str()
                .strip_prefix("Players/05 - A/")
                .unwrap()
                .to_owned()
        })
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
}
