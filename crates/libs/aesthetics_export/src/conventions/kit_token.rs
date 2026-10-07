//! Kit-dependent assets (`model_format.md` "Kit-dependent assets (`kitN`)"): the token a file
//! stem carries to say which kit number it belongs to, `kit1` to `kit9`, or that it stands for
//! whichever kit is picked in the game, `kitN` (`team_compiler/pipeline.md` "4. Per-export
//! non-model steps", Kit-dependent assets).

use std::borrow::Cow;

/// A stem's kit token: the reference `kitN`, or the variant for one kit number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KitToken {
    /// `kitN`: the variant of the kit number picked in the game.
    Reference,
    /// `kit1` to `kit9`: the variant shown when that kit number is picked.
    Variant(u8),
}

/// The first kit token of `stem` and the stem's reference spelling (the token as `kitN`),
/// or `None` when it holds none.
pub fn kit_token(stem: &str) -> Option<(KitToken, String)> {
    let (at, token) = find_token(stem)?;
    Some((token, with_token_char(stem, at, 'N')))
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
pub(crate) fn without_kit_token(stem: &str) -> Cow<'_, str> {
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
    fn the_token_goes_with_the_delimiter_before_it_or_else_the_one_after() {
        let cases = [
            ("boots_kit1", "boots"),
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
}
