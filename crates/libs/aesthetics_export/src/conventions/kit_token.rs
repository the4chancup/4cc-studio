//! Kit-dependent assets (`model_format.md` "Kit-dependent assets (`kitN`)"): the token a file
//! stem carries to say which kit number it belongs to, `kit1` to `kit9`, that it stands for
//! whichever kit is picked in the game, `kitN`, or that it is the set's fallback, `kitF`
//! (`team_compiler/pipeline.md` "4. Per-export non-model steps", Kit-dependent assets).

use std::borrow::Cow;

/// A stem's kit token: the reference `kitN`, the variant for one kit number, or the set's
/// fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KitToken {
    /// `kitN`: the variant of the kit number picked in the game.
    Reference,
    /// `kit1` to `kit9`: the variant shown when that kit number is picked.
    Variant(u8),
    /// `kitF`: the set's fallback, what a kit number with no variant of its own gets. Never
    /// a file under its own name, so a path naming it finds nothing.
    Fallback,
}

/// The first kit token of `stem` and the stem's reference spelling (the token as `kitN`),
/// or `None` when it holds none.
pub fn kit_token(stem: &str) -> Option<(KitToken, String)> {
    let (at, token) = find_token(stem)?;
    Some((token, with_token_char(stem, at, 'N')))
}

/// The member of the kit set `reference` names that `path` is, both paths below one folder
/// (`/`-separated, extension included): `Variant(k)` for a `kit1`…`kit9` member, `Fallback`
/// for the `kitF` one, `None` when `reference` names no set or `path` is not one of its
/// members. Two forms (`model_format.md` "Kit-dependent assets"): a `kitN` stem's set is its
/// own directory's files with a member token of the same reference, whatever their
/// extensions (`pants_kitN.dds` names `pants_kit1.dds`, `pants_kitF.png`); a `kitN`
/// directory segment's is the files of the same folded stem at the same path under the
/// `kit1`…`kit9` and `kitF` spellings of that segment (`kitN/armor_bsm.dds` and
/// `kitN/textures/armor_bsm.dds` name `kit1/armor_bsm.dds`, `kitF/armor_bsm.png`,
/// `kit1/textures/armor_bsm.dds`), the token spelled exactly as the stem's is and the
/// other segments comparing folded. A member counts whatever its file kind: the caller
/// picks (`common_link_files` keeps textures).
pub(crate) fn kit_set_member(reference: &str, path: &str) -> Option<KitToken> {
    let (reference_directory, reference_name) = directory_name(reference);
    let (path_directory, path_name) = directory_name(path);
    let (reference_stem, _) = stem_extension(reference_name);
    let (path_stem, _) = stem_extension(path_name);
    let reference_segments: Vec<&str> = reference_directory.split('/').collect();
    let path_segments: Vec<&str> = path_directory.split('/').collect();
    if let Some(at) = reference_segments
        .iter()
        .position(|segment| *segment == "kitN")
        && reference_segments.len() == path_segments.len()
        && let Some(token @ (KitToken::Variant(_) | KitToken::Fallback)) =
            kit_folder(path_segments[at])
        && reference_segments
            .iter()
            .zip(path_segments.iter())
            .enumerate()
            .all(|(index, (reference, path))| {
                index == at || vtree::fold_name(reference) == vtree::fold_name(path)
            })
        && vtree::fold_name(reference_stem) == vtree::fold_name(path_stem)
    {
        return Some(token);
    }
    if let Some((KitToken::Reference, reference_stem)) = kit_token(reference_stem)
        && vtree::fold_name(reference_directory) == vtree::fold_name(path_directory)
        && let Some((token @ (KitToken::Variant(_) | KitToken::Fallback), member_reference)) =
            kit_token(path_stem)
        && vtree::fold_name(&member_reference) == vtree::fold_name(&reference_stem)
    {
        return Some(token);
    }
    None
}

/// Whether `path`, a file's path below a folder (`/`-separated), names a kit set's `kitF`
/// fallback — its file stem's token (`pants_kitF.dds`) or a `kitF` directory segment
/// (`kitF/a.dds`, `kitF/textures/a.dds`): a path that finds nothing, `kitF` never being a
/// file under its own name (`model_format.md` "Kit-dependent assets").
pub(crate) fn names_kit_fallback(path: &str) -> bool {
    let (directory, name) = directory_name(path);
    let (stem, _) = stem_extension(name);
    matches!(kit_token(stem), Some((KitToken::Fallback, _)))
        || directory.split('/').any(|segment| segment == "kitF")
}

/// `stem`, which holds a kit token, with its first token spelled for `kit` (`pants_kit1` and
/// 3 give `pants_kit3`); `None` when `stem` holds no token.
pub fn variant_stem(stem: &str, kit: u8) -> Option<String> {
    let (at, _) = find_token(stem)?;
    Some(with_token_char(stem, at, char::from(b'0' + kit)))
}

/// `stem` without its first kit token and one delimiter next to it, the one before the token
/// when there is one, otherwise the one after (`boots_kit1` and `kit1_boots` give `boots`,
/// `a_kit2_boots` gives `a_boots`): the stem a model's type and allowed name are read from,
/// so a per-kit model is typed as its set is. Borrowed unchanged when `stem` holds no token.
pub fn without_kit_token(stem: &str) -> Cow<'_, str> {
    let Some((at, _)) = find_token(stem) else {
        return Cow::Borrowed(stem);
    };
    let start = at + 1 - "kitN".len();
    let end = at + 1;
    // The token is delimited on both sides, so a byte next to it is a one-byte delimiter.
    let (cut_start, cut_end) = if start > 0 {
        (start - 1, end)
    } else if end < stem.len() {
        (start, end + 1)
    } else {
        (start, end)
    };
    Cow::Owned(format!("{}{}", &stem[..cut_start], &stem[cut_end..]))
}

/// The byte offset in `stem` of its first token's last character (the `N`, `F` or the
/// digit), with the token. A token is `kit` followed by `N`, `F` or `1` to `9`, spelled
/// exactly so, with `_`, `-`, `.` or the stem's end on each side: `skitN`, `kitNx` and
/// `kitf` hold none.
fn find_token(stem: &str) -> Option<(usize, KitToken)> {
    let bytes = stem.as_bytes();
    let delimits = |byte: Option<&u8>| byte.is_none_or(|byte| matches!(byte, b'_' | b'-' | b'.'));
    stem.match_indices("kit").find_map(|(start, _)| {
        let at = start + "kit".len();
        let token = match bytes.get(at) {
            Some(b'N') => KitToken::Reference,
            Some(b'F') => KitToken::Fallback,
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

/// `path`'s directory and file name: `("a/b", "c.dds")` for `a/b/c.dds`, `("", "c.dds")`
/// for `c.dds`.
fn directory_name(path: &str) -> (&str, &str) {
    path.rsplit_once('/').unwrap_or(("", path))
}

/// A file name's stem and extension: `("a.b", "dds")` for `a.b.dds`, `("x", "")` for `x`.
fn stem_extension(name: &str) -> (&str, &str) {
    name.rsplit_once('.').unwrap_or((name, ""))
}

/// The token a whole folder name is, spelled exactly as the stem token is (`kitN`,
/// `kitF`, `kit1`…`kit9`): `x_kit1` and `KIT1` are none.
fn kit_folder(segment: &str) -> Option<KitToken> {
    if segment.len() != "kitN".len() {
        return None;
    }
    kit_token(segment).map(|(token, _)| token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_token_is_kit_n_or_kit_1_to_9_between_delimiters_or_the_stem_s_ends() {
        let cases = [
            ("pants_kitN", KitToken::Reference, "pants_kitN"),
            ("pants_kitF", KitToken::Fallback, "pants_kitN"),
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
            "pants_kitf",
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
        // The fallback respells like a variant.
        assert_eq!(variant_stem("pants_kitF", 2), Some("pants_kit2".to_owned()));
        assert_eq!(variant_stem("pants", 1), None);
    }

    #[test]
    fn the_token_goes_with_the_delimiter_before_it_or_else_the_one_after() {
        let cases = [
            ("boots_kit1", "boots"),
            ("boots_kitF", "boots"),
            ("kit1_boots", "boots"),
            ("face_high-kitN", "face_high"),
            ("a_kit2_boots", "a_boots"),
            // Two different delimiters show which one goes.
            ("a-kit2_boots", "a_boots"),
            ("kit2-a_boots", "a_boots"),
            ("x.kitN.y", "x.y"),
            ("kit1", ""),
            ("bootskit1", "bootskit1"),
        ];
        for (stem, expected) in cases {
            assert_eq!(without_kit_token(stem), expected, "{stem}");
        }
        assert!(matches!(without_kit_token("boots"), Cow::Borrowed("boots")));
    }

    #[test]
    fn a_kitn_directory_names_the_set_of_the_same_named_files_under_its_member_siblings() {
        let members = [
            (
                "kitN/armor_bsm.dds",
                "kit2/armor_bsm.dds",
                KitToken::Variant(2),
            ),
            (
                "kitN/armor_bsm.dds",
                "kitF/armor_bsm.dds",
                KitToken::Fallback,
            ),
            // The members are one set by stem, whatever their extensions.
            ("kitN/a.dds", "kitF/a.png", KitToken::Fallback),
            (
                "jessie/kitN/a.dds",
                "Jessie/kit1/A.dds",
                KitToken::Variant(1),
            ),
            // The token may sit anywhere in the path, a folder name included.
            (
                "kitN/textures/a.dds",
                "kit1/textures/a.dds",
                KitToken::Variant(1),
            ),
        ];
        for (reference, path, token) in members {
            assert_eq!(kit_set_member(reference, path), Some(token), "{path}");
        }
        for (reference, path) in [
            // Another directory's set.
            ("kitN/a.dds", "jessie/kit1/a.dds"),
            // A member folder is spelled exactly, as the stem token is.
            ("kitN/a.dds", "KIT1/a.dds"),
            ("kitN/a.dds", "kitf/a.dds"),
            // The reference itself is no member, nor is another file name.
            ("kitN/a.dds", "kitN/a.dds"),
            ("kitN/a.dds", "kit1/b.dds"),
            // One `kitN` segment's place names the set: the directories match segment-wise.
            ("jessie/kitN/x/a.dds", "jessie/kit2/y/a.dds"),
            ("kitN/a.dds", "kit1/x/a.dds"),
        ] {
            assert_eq!(kit_set_member(reference, path), None, "{path}");
        }
    }

    #[test]
    fn a_kitn_stem_names_the_set_of_its_directory_s_member_stems() {
        assert_eq!(
            kit_set_member("pants_kitN.dds", "pants_kit1.dds"),
            Some(KitToken::Variant(1))
        );
        assert_eq!(
            kit_set_member("pants_kitN.dds", "pants_kitF.dds"),
            Some(KitToken::Fallback)
        );
        // The members are one set by stem, whatever their extensions.
        assert_eq!(
            kit_set_member("pants_kitN.dds", "pants_kit1.png"),
            Some(KitToken::Variant(1))
        );
        for (reference, path) in [
            // The directory matches, folded.
            ("jessie/pants_kitN.dds", "pants_kit1.dds"),
            // A member's reference is the reference's stem.
            ("pants_kitN.dds", "socks_kit1.dds"),
            // A path with no token names no set and is no member.
            ("pants.dds", "pants.dds"),
            ("pants_kitN.dds", "pants_kitN.dds"),
            ("pants_kit1.dds", "pants_kit2.dds"),
        ] {
            assert_eq!(kit_set_member(reference, path), None, "{path}");
        }
    }

    #[test]
    fn a_path_naming_the_kitf_fallback_finds_nothing() {
        for path in [
            "pants_kitF.dds",
            "kitF/a.dds",
            "jessie/kitF/a.dds",
            "kitF/textures/a.dds",
        ] {
            assert!(names_kit_fallback(path), "{path}");
        }
        for path in [
            "kitf/a.dds",
            "pants.dds",
            "kit1/a.dds",
            "pants_kitN.dds",
            "kitN/a.dds",
        ] {
            assert!(!names_kit_fallback(path), "{path}");
        }
    }
}
