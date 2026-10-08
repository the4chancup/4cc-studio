//! Kit-dependent assets (`model_format.md` "Kit-dependent assets (`kitN`)"): the kit number of
//! a kit slot, the per-kit model files a Fox target cannot use (`team_compiler/pipeline.md`
//! "4. Per-export non-model steps", Kit-dependent assets), and whether a kit reference has a
//! variant among some stems. The token a file stem carries, `kit1` to `kit9` or `kitN`, is the
//! export format's (`aesthetics_export::kit_token`).

use std::collections::BTreeMap;

use aesthetics_export::{FileDescriptor, FileKind, KitToken, ModelFormat, kit_token};
use kit_config::KitSlot;
use vtree::ScopePath;

use crate::plan::subset::file_stem;

/// The kit number of a player kit `slot` (`p1` is 1); `None` for the goalkeeper's `g1`, which
/// is not a number of its own: the number picked in the game selects the outfield and the
/// goalkeeper kit together.
pub(crate) fn kit_number(slot: KitSlot) -> Option<u8> {
    match slot {
        KitSlot::P1 => Some(1),
        KitSlot::P2 => Some(2),
        KitSlot::P3 => Some(3),
        KitSlot::P4 => Some(4),
        KitSlot::P5 => Some(5),
        KitSlot::P6 => Some(6),
        KitSlot::P7 => Some(7),
        KitSlot::P8 => Some(8),
        KitSlot::P9 => Some(9),
        KitSlot::G1 => None,
    }
}

/// Per-kit model files of one directory of a model folder (`pants_kit1.fmdl`,
/// `pants_kit2.model`): a Fox target has no way to switch models with the kit, so only the
/// lowest variant is compiled.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ModelVariantSet {
    /// The set's reference with the lowest variant's extension, as `kit_variant_model_fox`
    /// names it (`pants_kitN.fmdl`).
    pub(crate) reference: String,
    /// The file name of the lowest variant, the one compiled (`pants_kit1.fmdl`).
    pub(crate) used: String,
    /// The export paths of the other variants, which nothing reads.
    pub(crate) left_out: Vec<ScopePath>,
}

/// The sets of per-kit model files among `files`, a model folder's: the `.fmdl` and `.model`
/// variants whose stems differ only in their token's digit (compared case-folded, as the file
/// system compares names) in one directory form a set, whatever their format (`pipeline.md`
/// "Kit-dependent assets"), and a set of one is an ordinary model. A variant present in both
/// formats is one variant, its FMDL the selected representation (target-native first, Fox
/// being the only target with sets): the `.model` beside it is neither `used` nor left out,
/// a model the FMDL beats (`FolderModels::beaten`). Ordered by directory, then reference.
pub(crate) fn model_variant_sets(files: &[FileDescriptor]) -> Vec<ModelVariantSet> {
    // Each set's variants by directory and folded reference stem: (kit number, whether it is
    // a `.model`, file name, path, reference name), in the folder's file order.
    type Variant<'a> = (u8, bool, &'a str, &'a ScopePath, String);
    let mut sets: BTreeMap<(String, String), Vec<Variant<'_>>> = BTreeMap::new();
    for file in files {
        let pre_fox = match file.kind {
            FileKind::Model(ModelFormat::Fmdl) => false,
            FileKind::Model(ModelFormat::PesModel) => true,
            FileKind::Model(ModelFormat::Gltf)
            | FileKind::Texture
            | FileKind::Skl
            | FileKind::Fclo
            | FileKind::Xml
            | FileKind::Mtl
            | FileKind::MaterialsToml
            | FileKind::Bin
            | FileKind::SharedLink(_)
            | FileKind::CommonLink
            | FileKind::Marker(_)
            | FileKind::Metadata(_)
            | FileKind::Other => continue,
        };
        let name = file.path.name();
        let stem = file_stem(name);
        let Some((KitToken::Variant(kit), reference)) = kit_token(stem) else {
            continue;
        };
        let reference_name = format!("{reference}{}", &name[stem.len()..]);
        let directory = file
            .path
            .parent()
            .map_or_else(String::new, |parent| parent.as_str().to_owned());
        sets.entry((directory, vtree::fold_name(&reference)))
            .or_default()
            .push((kit, pre_fox, name, &file.path, reference_name));
    }
    sets.into_values()
        .filter_map(|mut variants| {
            // A stable sort: of two files of one number and format, the first in the folder is
            // used; an FMDL comes before the `.model` of its number, which is then no variant.
            variants.sort_by_key(|(kit, pre_fox, ..)| (*kit, *pre_fox));
            variants.dedup_by(|later, kept| later.0 == kept.0 && later.1 && !kept.1);
            let ((_, _, used, _, reference), others) = variants.split_first()?;
            if others.is_empty() {
                return None;
            }
            Some(ModelVariantSet {
                reference: reference.clone(),
                used: (*used).to_owned(),
                left_out: others
                    .iter()
                    .map(|(_, _, _, path, _)| (*path).clone())
                    .collect(),
            })
        })
        .collect()
}

/// Whether `stem` is a kit reference (`pants_kitN`, never a file of its own) and `stems` hold a
/// variant of its set (`pants_kit2`), compared as `stems` are, as spelled.
pub(crate) fn has_variant_among<'a>(stem: &str, stems: impl IntoIterator<Item = &'a str>) -> bool {
    let Some((KitToken::Reference, reference)) = kit_token(stem) else {
        return false;
    };
    stems.into_iter().any(|held| {
        matches!(kit_token(held), Some((KitToken::Variant(_), held_reference)) if vtree::fold_name(&held_reference) == vtree::fold_name(&reference))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_player_kit_slots_are_kit_numbers() {
        assert_eq!(kit_number(KitSlot::P1), Some(1));
        assert_eq!(kit_number(KitSlot::P9), Some(9));
        assert_eq!(kit_number(KitSlot::G1), None);
    }

    /// The files at `paths`, classified by name.
    fn files(paths: &[&str]) -> Vec<FileDescriptor> {
        paths
            .iter()
            .map(|path| {
                let path = ScopePath::new(path).unwrap();
                FileDescriptor {
                    size: 0,
                    kind: aesthetics_export::classify(path.name()),
                    source: path.clone(),
                    path,
                }
            })
            .collect()
    }

    /// `model_variant_sets` of the files at `paths`, as (reference, used, left-out paths).
    fn sets(paths: &[&str]) -> Vec<(String, String, Vec<String>)> {
        model_variant_sets(&files(paths))
            .into_iter()
            .map(|set| {
                let left_out = set
                    .left_out
                    .iter()
                    .map(|path| path.as_str().to_owned())
                    .collect();
                (set.reference, set.used, left_out)
            })
            .collect()
    }

    #[test]
    fn a_model_variant_set_counts_either_native_format_one_variant_per_kit_its_fmdl_first() {
        let set = |reference: &str, used: &str, left_out: &[&str]| {
            (
                reference.to_owned(),
                used.to_owned(),
                left_out.iter().map(|path| (*path).to_owned()).collect(),
            )
        };
        assert_eq!(
            sets(&[
                "P/pants_kit1.model",
                "P/pants_kit2.model",
                "P/pants_kit1.mtl"
            ]),
            [set(
                "pants_kitN.model",
                "pants_kit1.model",
                &["P/pants_kit2.model"]
            )]
        );
        // Mixed formats: the lowest variant names the reference's extension.
        assert_eq!(
            sets(&["P/pants_kit2.fmdl", "P/pants_kit1.model"]),
            [set(
                "pants_kitN.model",
                "pants_kit1.model",
                &["P/pants_kit2.fmdl"]
            )]
        );
        // A variant in both formats is one variant, its FMDL the selected representation:
        // the `.model` of its number is neither used nor left out (the FMDL beats it).
        assert_eq!(
            sets(&[
                "P/pants_kit1.model",
                "P/pants_kit1.fmdl",
                "P/pants_kit2.model",
                "P/pants_kit2.fmdl",
            ]),
            [set(
                "pants_kitN.fmdl",
                "pants_kit1.fmdl",
                &["P/pants_kit2.fmdl"]
            )]
        );
        // One variant in two formats is no set.
        assert_eq!(sets(&["P/pants_kit1.model", "P/pants_kit1.fmdl"]), []);
        // A glTF is no variant here.
        assert_eq!(sets(&["P/pants_kit1.fmdl", "P/pants_kit2.glb"]), []);
        // Two files of one number and one format (a case-sensitive file system holds both):
        // the first in the folder is used, the other is a left-out variant, not dropped in
        // silence as the `.model` beside an FMDL of its number is.
        assert_eq!(
            sets(&[
                "P/pants_kit1.fmdl",
                "P/Pants_kit1.fmdl",
                "P/pants_kit2.fmdl"
            ]),
            [set(
                "pants_kitN.fmdl",
                "pants_kit1.fmdl",
                &["P/Pants_kit1.fmdl", "P/pants_kit2.fmdl"]
            )]
        );
    }

    #[test]
    fn a_kit_reference_has_a_variant_among_stems_holding_one_of_its_set() {
        assert!(has_variant_among("pants_kitN", ["skin", "Pants_kit2"]));
        assert!(!has_variant_among("pants_kitN", ["skin", "sock_kit2"]));
        // `pants_kit1` is a variant, not a reference: it is found by its own stem.
        assert!(!has_variant_among(
            "pants_kit1",
            ["pants_kit1", "pants_kit2"]
        ));
    }
}
