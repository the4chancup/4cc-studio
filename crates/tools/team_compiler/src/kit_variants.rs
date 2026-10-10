//! Kit-dependent assets (`model_format.md` "Kit-dependent assets (`kitN`)"): the kit number of
//! a kit slot, the per-kit model files an output no `face.xml` lists cannot use
//! (`team_compiler/pipeline.md` "4. Per-export non-model steps", Kit-dependent assets), and
//! whether a kit reference has a variant among some stems. The token a file stem carries,
//! `kit1` to `kit9` or `kitN`, is the export format's (`aesthetics_export::kit_token`).

use std::collections::BTreeMap;

use aesthetics_export::{FileDescriptor, FileKind, KitToken, ModelFormat, kit_token};
use kit_config::KitSlot;
use pes_version::Engine;
use vtree::ScopePath;

use crate::plan::roles::{file_stem, native_format};

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

/// Per-kit model files of one part (`pants_kit1.fmdl`, `pants_kit2.model`): where no `face.xml`
/// names the set, nothing switches models with the kit, so only the lowest variant is
/// compiled.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ModelVariantSet {
    /// The set's reference with the lowest variant's extension, as
    /// `kit_variant_model_left_out` names it (`pants_kitN.fmdl`).
    pub(crate) reference: String,
    /// The export path of the lowest variant, the one compiled
    /// (`Players/05 - A/pants_kit1.fmdl`).
    pub(crate) used: ScopePath,
    /// The export paths of the other variants, which nothing reads.
    pub(crate) left_out: Vec<ScopePath>,
}

/// One per-kit model file of a set, as `model_variant_sets` orders them.
struct Variant<'a> {
    /// Its kit number.
    kit: u8,
    /// The index of its source among the sources given.
    source: usize,
    /// It is in the other engine's format, which the target's format beats.
    other_format: bool,
    /// Its directory's export path.
    directory: Option<ScopePath>,
    /// Its export path.
    path: &'a ScopePath,
    /// Its set's reference with its own extension (`pants_kitN.fmdl`).
    reference: String,
}

/// The sets of per-kit model files among `sources`, the files of each folder one part is built
/// from that may take a role there (`roles::role_files`), the player's own first and then each
/// combined shared folder's in link order, on a target of `engine`: the `.fmdl` and `.model`
/// variants whose stems differ only in their token's digit (compared case-folded, as the file
/// system compares names) form a set, whatever their format and wherever they sit among the
/// sources (`pipeline.md` "Kit-dependent assets": the part merges them, so a set split between
/// a folder and its `face/`, or between a player and a shared folder he combines, is one set),
/// and a set of one is an ordinary model. A variant present in both formats is one variant,
/// the target's format (`.fmdl` on Fox, `.model` on pre-Fox) its selected representation, as
/// the selection order says: the file of the other format beside it is neither `used` nor
/// left out, a model the target's beats (`FolderModels::beaten`), for every number. Of two
/// files of the used (lowest) number in two directories, the earlier source's is the variant
/// (the player's own wins a name he and a combined folder both hold) and the other is neither
/// used nor left out: another part of that kit, which the part merges with it. Every file of
/// another number is left out, wherever it sits: merged, it would show in every kit's model.
/// Of two of the used number in one directory, two spellings of one name on a case-sensitive
/// file system, the first is used and the other left out. Ordered by reference.
pub(crate) fn model_variant_sets(
    sources: &[Vec<&FileDescriptor>],
    engine: Engine,
) -> Vec<ModelVariantSet> {
    // Each set's variants by folded reference stem, in the sources' file order.
    let mut sets: BTreeMap<String, Vec<Variant<'_>>> = BTreeMap::new();
    for (source, files) in sources.iter().enumerate() {
        for file in files {
            let other_format = match file.kind {
                FileKind::Model(format @ (ModelFormat::Fmdl | ModelFormat::PesModel)) => {
                    format != native_format(engine)
                }
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
            sets.entry(vtree::fold_name(&reference))
                .or_default()
                .push(Variant {
                    kit,
                    source,
                    other_format,
                    directory: file.path.parent(),
                    path: &file.path,
                    reference: format!("{reference}{}", &name[stem.len()..]),
                });
        }
    }
    sets.into_values()
        .filter_map(|mut variants| {
            // A stable sort: of the files of one number, the earlier source's first, and in
            // one source the target's format before the other's, then the sources' order.
            variants.sort_by_key(|variant| (variant.kit, variant.source, variant.other_format));
            let used_kit = variants.first()?.kit;
            variants.dedup_by(|later, kept| {
                let beaten =
                    later.directory == kept.directory && later.other_format && !kept.other_format;
                // Another part of the used kit is merged with it; a left-out number's other
                // parts are left out with it, or they would merge into every kit's model.
                let used_kit_part = later.kit == used_kit && later.directory != kept.directory;
                later.kit == kept.kit && (beaten || used_kit_part)
            });
            let (used, others) = variants.split_first()?;
            if others.is_empty() {
                return None;
            }
            Some(ModelVariantSet {
                reference: used.reference.clone(),
                used: used.path.clone(),
                left_out: others.iter().map(|variant| variant.path.clone()).collect(),
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

    /// `model_variant_sets` of the files at `paths` on Fox, as (reference, used, left-out
    /// paths).
    fn sets(paths: &[&str]) -> Vec<(String, String, Vec<String>)> {
        engine_sets(paths, Engine::Fox)
    }

    /// `model_variant_sets` of the files at `paths` on a target of `engine`, as (reference,
    /// used, left-out paths).
    fn engine_sets(paths: &[&str], engine: Engine) -> Vec<(String, String, Vec<String>)> {
        source_sets(&[paths], engine)
    }

    /// `model_variant_sets` of the sources whose files are at `sources` on a target of
    /// `engine`, as (reference, used file name, left-out paths).
    fn source_sets(sources: &[&[&str]], engine: Engine) -> Vec<(String, String, Vec<String>)> {
        let files: Vec<Vec<FileDescriptor>> = sources.iter().map(|paths| files(paths)).collect();
        let sources: Vec<Vec<&FileDescriptor>> =
            files.iter().map(|files| files.iter().collect()).collect();
        model_variant_sets(&sources, engine)
            .into_iter()
            .map(|set| {
                let left_out = set
                    .left_out
                    .iter()
                    .map(|path| path.as_str().to_owned())
                    .collect();
                (set.reference, set.used.name().to_owned(), left_out)
            })
            .collect()
    }

    #[test]
    fn a_set_spans_the_directories_and_sources_of_its_part() {
        // Split between a player folder and its `face/`: one set.
        assert_eq!(
            sets(&["P/pants_kit2.fmdl", "P/face/pants_kit1.fmdl"]),
            [(
                "pants_kitN.fmdl".to_owned(),
                "pants_kit1.fmdl".to_owned(),
                vec!["P/pants_kit2.fmdl".to_owned()]
            )]
        );
        // Split between the player and a shared folder he combines: one set, whichever holds
        // the lower variant.
        assert_eq!(
            source_sets(
                &[
                    &["P/boots_kit2.fmdl"],
                    &["B/boots.fmdl", "B/boots_kit1.fmdl"]
                ],
                Engine::Fox
            ),
            [(
                "boots_kitN.fmdl".to_owned(),
                "boots_kit1.fmdl".to_owned(),
                vec!["P/boots_kit2.fmdl".to_owned()]
            )]
        );
        // One number in two sources: the player's own is the variant, the shared one of its
        // number another part of that kit, neither used nor left out.
        assert_eq!(
            source_sets(
                &[
                    &["P/boots_kit1.fmdl"],
                    &["B/boots_kit1.fmdl", "B/boots_kit2.fmdl"]
                ],
                Engine::Fox
            ),
            [(
                "boots_kitN.fmdl".to_owned(),
                "boots_kit1.fmdl".to_owned(),
                vec!["B/boots_kit2.fmdl".to_owned()]
            )]
        );
        assert_eq!(
            source_sets(
                &[&["P/boots_kit1.model"], &["B/boots_kit1.fmdl"]],
                Engine::Fox
            ),
            []
        );
        // In one source too: a `face/` variant of the number of one directly in the folder is
        // another part of that kit.
        assert_eq!(sets(&["P/pants_kit1.fmdl", "P/face/pants_kit1.fmdl"]), []);
    }

    #[test]
    fn every_file_of_a_left_out_kit_number_is_left_out_wherever_it_sits() {
        // Crocs' `pants_kit2` is a second part of kit 2, which no kit compiles: merged, it
        // would show in every kit's model.
        assert_eq!(
            source_sets(
                &[
                    &["P/pants_kit1.fmdl", "P/pants_kit2.fmdl"],
                    &["Faces/Crocs/pants_kit2.fmdl"]
                ],
                Engine::Fox
            ),
            [(
                "pants_kitN.fmdl".to_owned(),
                "pants_kit1.fmdl".to_owned(),
                vec![
                    "P/pants_kit2.fmdl".to_owned(),
                    "Faces/Crocs/pants_kit2.fmdl".to_owned()
                ]
            )]
        );
        // Of either format: nothing beside the Crocs `.model` beats it, so it is left out too.
        assert_eq!(
            source_sets(
                &[
                    &["P/pants_kit1.fmdl", "P/pants_kit2.fmdl"],
                    &["Faces/Crocs/pants_kit2.model"]
                ],
                Engine::Fox
            ),
            [(
                "pants_kitN.fmdl".to_owned(),
                "pants_kit1.fmdl".to_owned(),
                vec![
                    "P/pants_kit2.fmdl".to_owned(),
                    "Faces/Crocs/pants_kit2.model".to_owned()
                ]
            )]
        );
        // In one source too: a `face/` file of a left-out number is left out.
        assert_eq!(
            sets(&[
                "P/pants_kit1.fmdl",
                "P/pants_kit2.fmdl",
                "P/face/pants_kit2.fmdl"
            ]),
            [(
                "pants_kitN.fmdl".to_owned(),
                "pants_kit1.fmdl".to_owned(),
                vec![
                    "P/pants_kit2.fmdl".to_owned(),
                    "P/face/pants_kit2.fmdl".to_owned()
                ]
            )]
        );
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
    fn a_variant_in_both_formats_is_represented_by_the_target_s_format() {
        let paths = [
            "P/pants_kit1.fmdl",
            "P/pants_kit1.model",
            "P/pants_kit2.model",
        ];
        let left_out = vec!["P/pants_kit2.model".to_owned()];
        assert_eq!(
            engine_sets(&paths, Engine::Fox),
            [(
                "pants_kitN.fmdl".to_owned(),
                "pants_kit1.fmdl".to_owned(),
                left_out.clone()
            )]
        );
        assert_eq!(
            engine_sets(&paths, Engine::PreFox),
            [(
                "pants_kitN.model".to_owned(),
                "pants_kit1.model".to_owned(),
                left_out
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
