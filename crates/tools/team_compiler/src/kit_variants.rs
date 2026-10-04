//! Kit-dependent assets (`model_format.md` "Kit-dependent assets (`kitN`)"): the token a file
//! stem carries to say which kit number it belongs to, `kit1` to `kit9`, or that it stands for
//! whichever kit is picked in the game, `kitN`; and the per-kit model files a Fox target cannot
//! use (`team_compiler/pipeline.md` "4. Per-export non-model steps", Kit-dependent assets).

use std::collections::BTreeMap;

use aesthetics_export::{FileDescriptor, FileKind, ModelFormat};
use kit_config::KitSlot;
use vtree::ScopePath;

use crate::plan::subset::file_stem;

/// A stem's kit token: the reference `kitN`, or the variant for one kit number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KitToken {
    /// `kitN`: the variant of the kit number picked in the game.
    Reference,
    /// `kit1` to `kit9`: the variant shown when that kit number is picked.
    Variant(u8),
}

/// The first kit token of `stem` and the stem's reference spelling (the token as `kitN`),
/// or `None` when it holds none.
pub(crate) fn kit_token(stem: &str) -> Option<(KitToken, String)> {
    let (at, token) = find_token(stem)?;
    Some((token, with_token_char(stem, at, 'N')))
}

/// `stem`, which holds a kit token, with its first token spelled for `kit` (`pants_kit1` and
/// 3 give `pants_kit3`); `None` when `stem` holds no token.
pub(crate) fn variant_stem(stem: &str, kit: u8) -> Option<String> {
    let (at, _) = find_token(stem)?;
    Some(with_token_char(stem, at, char::from(b'0' + kit)))
}

/// The byte offset in `stem` of its first token's last character (the `N` or the digit), with
/// the token. A token is `kit` followed by `N` or `1` to `9`, spelled exactly so, with `_`, `-`,
/// `.` or the stem's end on each side: `skitN` and `kitNx` hold none.
fn find_token(stem: &str) -> Option<(usize, KitToken)> {
    let bytes = stem.as_bytes();
    let delimits = |byte: Option<&u8>| byte.is_none_or(|byte| matches!(byte, b'_' | b'-' | b'.'));
    stem.match_indices("kit").find_map(|(start, _)| {
        let at = start + "kit".len();
        let token = match bytes.get(at) {
            Some(b'N') => KitToken::Reference,
            Some(digit @ b'1'..=b'9') => KitToken::Variant(digit - b'0'),
            _ => return None,
        };
        let before = start.checked_sub(1).and_then(|index| bytes.get(index));
        (delimits(before) && delimits(bytes.get(at + 1))).then_some((at, token))
    })
}

/// `stem` with the ASCII character at byte offset `at` replaced by `replacement`.
fn with_token_char(stem: &str, at: usize, replacement: char) -> String {
    format!("{}{replacement}{}", &stem[..at], &stem[at + 1..])
}

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
    fn a_token_is_kit_n_or_kit_1_to_9_between_delimiters_or_the_stem_s_ends() {
        let cases = [
            ("pants_kitN", KitToken::Reference, "pants_kitN"),
            ("pants_kit3", KitToken::Variant(3), "pants_kitN"),
            ("kit2_pants", KitToken::Variant(2), "kitN_pants"),
            ("kit1", KitToken::Variant(1), "kitN"),
            ("a-kit4.b", KitToken::Variant(4), "a-kitN.b"),
            ("pants_kit9", KitToken::Variant(9), "pants_kitN"),
        ];
        for (stem, token, reference) in cases {
            assert_eq!(
                kit_token(stem),
                Some((token, reference.to_owned())),
                "{stem}"
            );
        }
        for stem in [
            "skitN",
            "pants_kit0",
            "pants_kit10",
            "pants_kit",
            "pants_KIT1",
            "pants_kitn",
            "kitNx",
            "kit",
        ] {
            assert_eq!(kit_token(stem), None, "{stem}");
        }
    }

    #[test]
    fn a_variant_stem_respells_the_first_token_alone() {
        assert_eq!(variant_stem("pants_kit1", 3), Some("pants_kit3".to_owned()));
        assert_eq!(
            variant_stem("kit3_a-kit1", 9),
            Some("kit9_a-kit1".to_owned())
        );
        assert_eq!(variant_stem("skit1_kit2", 4), Some("skit1_kit4".to_owned()));
        assert_eq!(variant_stem("pants", 1), None);
    }

    #[test]
    fn only_the_player_kit_slots_are_kit_numbers() {
        assert_eq!(kit_number(KitSlot::P1), Some(1));
        assert_eq!(kit_number(KitSlot::P9), Some(9));
        assert_eq!(kit_number(KitSlot::G1), None);
    }
}
