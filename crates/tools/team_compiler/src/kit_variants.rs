//! Kit-dependent assets (`model_format.md` "Kit-dependent assets (`kitN`)"): the kit number of
//! a kit slot, and the per-kit model files a Fox target cannot use (`team_compiler/pipeline.md`
//! "4. Per-export non-model steps", Kit-dependent assets). The token a file stem carries,
//! `kit1` to `kit9` or `kitN`, is the export format's (`aesthetics_export::kit_token`).

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
/// `pants_kit2.fmdl`): a Fox target has no way to switch models with the kit, so only the
/// lowest variant is compiled.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ModelVariantSet {
    /// The set's reference with its extension, as `kit_variant_model_fox` names it
    /// (`pants_kitN.fmdl`).
    pub(crate) reference: String,
    /// The file name of the lowest variant, the one compiled (`pants_kit1.fmdl`).
    pub(crate) used: String,
    /// The export paths of the other variants, which nothing reads.
    pub(crate) left_out: Vec<ScopePath>,
}

/// The sets of per-kit FMDL files among `files`, a model folder's: the variants whose names
/// differ only in their token's digit (compared case-folded, as the file system compares
/// names) in one directory form a set, and a set of one is an ordinary model. Ordered by
/// directory, then reference.
pub(crate) fn model_variant_sets(files: &[FileDescriptor]) -> Vec<ModelVariantSet> {
    // Each set's variants by directory and folded reference name: (kit number, file name,
    // path, reference name), in the folder's file order.
    type Variant<'a> = (u8, &'a str, &'a ScopePath, String);
    let mut sets: BTreeMap<(String, String), Vec<Variant<'_>>> = BTreeMap::new();
    for file in files {
        if file.kind != FileKind::Model(ModelFormat::Fmdl) {
            continue;
        }
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
        sets.entry((directory, vtree::fold_name(&reference_name)))
            .or_default()
            .push((kit, name, &file.path, reference_name));
    }
    sets.into_values()
        .filter_map(|mut variants| {
            // A stable sort: of two files of one number, the first in the folder is used.
            variants.sort_by_key(|(kit, ..)| *kit);
            let ((_, used, _, reference), others) = variants.split_first()?;
            if others.is_empty() {
                return None;
            }
            Some(ModelVariantSet {
                reference: reference.clone(),
                used: (*used).to_owned(),
                left_out: others
                    .iter()
                    .map(|(_, _, path, _)| (*path).clone())
                    .collect(),
            })
        })
        .collect()
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
}
