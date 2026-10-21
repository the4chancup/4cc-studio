//! `display_name`: a raw save name without its colour codes. `name` keeps the
//! raw form for the editor's colour rendering; this is the text consumers show.

use std::borrow::Cow;

use pes_version::PesVersion;

use crate::codec::{TextMisfit, text_max, text_misfit};
use crate::schema::fields::PlayerText;
use crate::schema::{portable_text_max, schema_for};

/// The name without its colour codes: every `\x11c` plus the exactly eight
/// bytes that follow it (any bytes — the game does not check they are hex, so
/// a code at the string's end shorter than that is not a code and stays) and
/// every two-byte `\x11d` reset. Any other `0x11` sequence is untouched.
/// Stripping happens on bytes at char boundaries only, so the result is always
/// valid UTF-8.
pub fn display_name(raw: &str) -> String {
    name_spans(raw).into_iter().map(|(_, text)| text).collect()
}

/// The field's length unit (`PlayerText`'s): the name counts UTF-8 bytes,
/// the shirt name single-byte chars.
pub(crate) fn single_byte(which: PlayerText) -> bool {
    match which {
        PlayerText::Name => false,
        PlayerText::ShirtName => true,
    }
}

/// Whether `text` fits `version`'s `which` text field, by the codec's own
/// rule (`text_misfit`: the name's UTF-8 bytes, the shirt name's single-byte
/// encoding, each within the field's length): the editors refuse what
/// `to_bytes` would.
pub fn player_text_fits(version: PesVersion, which: PlayerText, text: &str) -> bool {
    text_misfit(
        text,
        text_max(schema_for(version).player.texts, which),
        single_byte(which),
    )
    .is_none()
}

/// Whether `text` may replace `stored` in `version`'s shirt-name field
/// (the info strip's rule, `plans/save_editor.md` "Editing UI"; the name
/// field's rule is `player_text_fits` on the version's own field): the
/// codec's own content rule (no NUL, the single-byte charset) and, in
/// length, either within the shorter of `portable_text_max` and the
/// version's own field, or no longer than `stored`, since a shirt name the
/// save already holds past the limit can be shortened a keystroke at a
/// time, never lengthened.
pub fn shirt_name_edit_fits(version: PesVersion, stored: &str, text: &str) -> bool {
    let max = portable_text_max(PlayerText::ShirtName).min(text_max(
        schema_for(version).player.texts,
        PlayerText::ShirtName,
    ));
    match text_misfit(text, max, true) {
        None => true,
        Some(TextMisfit::TooLong { .. }) => text.chars().count() <= stored.chars().count(),
        Some(TextMisfit::Nul | TextMisfit::BeyondSingleByte) => false,
    }
}

/// `text` cut to what `version`'s `which` field holds: the longest prefix
/// of whole spans (`name_spans`'s boundaries: a character or a colour
/// code) that fits, so a crop never splits a character or leaves half a
/// colour code drawn as text. Borrowed when it fits.
pub fn crop_player_text<'a>(version: PesVersion, which: PlayerText, text: &'a str) -> Cow<'a, str> {
    let max = text_max(schema_for(version).player.texts, which);
    if text_misfit(text, max, single_byte(which)).is_none() {
        return Cow::Borrowed(text);
    }
    let mut left = max;
    let mut end = 0;
    for (start, stop) in text_spans(text) {
        let len = if single_byte(which) {
            text[start..stop].chars().count()
        } else {
            stop - start
        };
        if len > left {
            break;
        }
        left -= len;
        end = stop;
    }
    Cow::Owned(text[..end].to_string())
}

/// The raw text's spans as `(start, end)` byte ranges: each colour code
/// whole (`\x11c` plus its eight bytes, `\x11d`) or one character — the
/// boundaries `name_spans` uses.
fn text_spans(text: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    let bytes = text.as_bytes();
    let mut i = 0;
    std::iter::from_fn(move || {
        if i >= bytes.len() {
            return None;
        }
        let code = if bytes[i] == 0x11 && i + 1 < bytes.len() && bytes[i + 1] == b'c' {
            Some(10).filter(|end| i + end <= bytes.len() && text.is_char_boundary(i + end))
        } else if bytes[i] == 0x11 && i + 1 < bytes.len() && bytes[i + 1] == b'd' {
            Some(2).filter(|end| text.is_char_boundary(i + end))
        } else {
            None
        };
        let end = code.map(|len| i + len).unwrap_or_else(|| {
            let mut end = i + 1;
            while end < bytes.len() && !text.is_char_boundary(end) {
                end += 1;
            }
            end
        });
        let span = (i, end);
        i = end;
        Some(span)
    })
}

/// The name's text runs and the color each is drawn in, as the game draws
/// them: a `\x11c` code (`display_name`'s rule for what one is) colors the
/// text after it with its first six bytes as RGB hex (the alpha not drawn),
/// `\x11d` resets to the normal color; a code whose six bytes are not hex
/// resets too. Runs are never empty; the runs' text joined is
/// `display_name(raw)`.
pub fn name_spans(raw: &str) -> Vec<(Option<[u8; 3]>, &str)> {
    let mut spans = Vec::new();
    let mut color = None;
    let mut start = 0;
    for (s, e) in text_spans(raw) {
        // The colour the code sets for the text after it (`None`: the normal
        // colour); a character span is no code.
        let next_color = match &raw.as_bytes()[s..e] {
            [0x11, b'c', ..] => rgb(&raw.as_bytes()[s + 2..s + 8]),
            [0x11, b'd'] => None,
            _ => continue,
        };
        if start < s {
            spans.push((color, &raw[start..s]));
        }
        color = next_color;
        start = e;
    }
    if start < raw.len() {
        spans.push((color, &raw[start..]));
    }
    spans
}

/// The RGB of a `\x11c` code's first six bytes (`code`), `None` when one is
/// not hex (the game does not check: the text keeps the normal color).
fn rgb(code: &[u8]) -> Option<[u8; 3]> {
    let nibble = |byte: u8| -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            b'A'..=b'F' => Some(byte - b'A' + 10),
            _ => None,
        }
    };
    let channel = |hi: u8, lo: u8| Some(nibble(hi)? * 16 + nibble(lo)?);
    Some([
        channel(code[0], code[1])?,
        channel(code[2], code[3])?,
        channel(code[4], code[5])?,
    ])
}

/// The eight bytes after a leading `\x11c` colour code (`"cc9900ff"`),
/// under `display_name`'s rule for what a code is (exactly eight bytes,
/// ending at a char boundary). `None` for a name without a leading code —
/// including a code that is not at the start.
pub fn name_color(raw: &str) -> Option<&str> {
    let bytes = raw.as_bytes();
    if bytes.len() >= 10 && bytes[0] == 0x11 && bytes[1] == b'c' && raw.is_char_boundary(10) {
        Some(&raw[2..10])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_text_is_accepted_up_to_the_portable_limit_or_no_longer_than_stored() {
        // The portable shirt-name limit (18 letters) holds whatever the
        // save's version is; on PES 16 the shirt's own 16 letters bind
        // instead.
        assert!(shirt_name_edit_fits(
            PesVersion::Pes21,
            "s",
            &"t".repeat(18)
        ));
        assert!(shirt_name_edit_fits(PesVersion::Pes21, "s", "t"));
        assert!(!shirt_name_edit_fits(
            PesVersion::Pes21,
            "s",
            &"s".repeat(19)
        ));
        assert!(!shirt_name_edit_fits(
            PesVersion::Pes16,
            "s",
            &"s".repeat(17)
        ));
        assert!(shirt_name_edit_fits(
            PesVersion::Pes16,
            "s",
            &"s".repeat(16)
        ));
        // A stored shirt name past the limit shortens a keystroke at a
        // time, in letters.
        assert!(shirt_name_edit_fits(
            PesVersion::Pes21,
            &"s".repeat(19),
            &"x".repeat(19)
        ));
        assert!(!shirt_name_edit_fits(
            PesVersion::Pes21,
            &"s".repeat(19),
            &"x".repeat(20)
        ));
        // A NUL and a char above U+00FF refuse whatever the lengths.
        assert!(!shirt_name_edit_fits(
            PesVersion::Pes21,
            &"s".repeat(19),
            "a\0b"
        ));
        assert!(!shirt_name_edit_fits(
            PesVersion::Pes21,
            &"s".repeat(19),
            "ła"
        ));
    }

    #[test]
    fn a_crop_cuts_to_the_fields_limit_at_whole_spans() {
        // PES 19's name field holds 46 bytes: a 59-byte ASCII name keeps
        // its first 46; a two-byte `é` on the boundary stays whole before it.
        let name = "N".repeat(59);
        assert_eq!(
            crop_player_text(PesVersion::Pes19, PlayerText::Name, &name),
            "N".repeat(46)
        );
        let name = format!("{}é{}", "N".repeat(45), "N".repeat(13));
        assert_eq!(name.len(), 60);
        assert_eq!(
            crop_player_text(PesVersion::Pes19, PlayerText::Name, &name),
            "N".repeat(45)
        );
        // A limit inside a colour code (ten bytes) drops the code whole,
        // never the text after it either: the crop is the prefix.
        let name = format!("{}\x11c112233ff{}", "N".repeat(40), "N".repeat(10));
        assert_eq!(name.len(), 60);
        assert_eq!(
            crop_player_text(PesVersion::Pes19, PlayerText::Name, &name),
            "N".repeat(40)
        );
        // PES 16's shirt field holds 16 letters: a 20-letter one keeps 16.
        assert_eq!(
            crop_player_text(PesVersion::Pes16, PlayerText::ShirtName, &"s".repeat(20)),
            "s".repeat(16)
        );
        // A text that fits is borrowed, not owned.
        let fits = "FITS".to_string();
        assert!(matches!(
            crop_player_text(PesVersion::Pes19, PlayerText::Name, &fits),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn colour_codes_strip_and_everything_else_stays() {
        assert_eq!(
            display_name("\x11cffb50effLORD \x11cfff17affFULP"),
            "LORD FULP"
        );
        assert_eq!(
            display_name("\x11cc9002bffPEP\x11cffffffffSI \x11c004b93ffMAN"),
            "PEPSI MAN"
        );
        assert_eq!(
            display_name("\x11c789922ff>99% Chance to hit.\x11d"),
            ">99% Chance to hit."
        );
        assert_eq!(
            display_name("WHEN THE \x11cb7bec5ffWORK RESULT\x11d WAS GOOD"),
            "WHEN THE WORK RESULT WAS GOOD"
        );
        assert_eq!(display_name("\x11ca000c8ONTriple T"), "Triple T");
        assert_eq!(
            display_name("\x11ca000c8ONeF\u{da}TBOL\u{2122}"),
            "eF\u{da}TBOL\u{2122}"
        );
        // A code shorter than eight bytes at the end is not a code.
        assert_eq!(display_name("\x11c12345"), "\x11c12345");
        assert_eq!(display_name("plain"), "plain");
        assert_eq!(display_name("\x11x?"), "\x11x?");
    }

    #[test]
    fn name_spans_carry_the_color_of_the_code_before_them() {
        assert_eq!(
            name_spans("\x11cc9002bffPEP\x11cffffffffSI \x11c004b93ffMAN"),
            [
                (Some([0xc9, 0x00, 0x2b]), "PEP"),
                (Some([0xff, 0xff, 0xff]), "SI "),
                (Some([0x00, 0x4b, 0x93]), "MAN"),
            ]
        );
        assert_eq!(
            name_spans("WHEN THE \x11cb7bec5ffWORK RESULT\x11d WAS GOOD"),
            [
                (None, "WHEN THE "),
                (Some([0xb7, 0xbe, 0xc5]), "WORK RESULT"),
                (None, " WAS GOOD"),
            ]
        );
        // The code's alpha is no part of the color, even when it is not hex.
        assert_eq!(
            name_spans("\x11ca000c8ONAME"),
            [(Some([0xa0, 0x00, 0xc8]), "AME")]
        );
        assert_eq!(name_spans("plain"), [(None, "plain")]);
    }

    #[test]
    fn name_spans_reset_on_a_reset_or_a_non_hex_code() {
        // A non-hex RGB resets like `\x11d` does (the game does not check).
        assert_eq!(
            name_spans("\x11ca000c8ONone\x11cggggggffplain\x11dagain"),
            [
                (Some([0xa0, 0x00, 0xc8]), "one"),
                (None, "plain"),
                (None, "again"),
            ]
        );
        // Two resets in a row merge into one run; runs are never empty.
        assert_eq!(
            name_spans("\x11c112233ffcolor\x11d\x11dplain"),
            [(Some([0x11, 0x22, 0x33]), "color"), (None, "plain")]
        );
        // A code-shaped suffix that is not a code is text, color and all.
        assert_eq!(
            name_spans("\x11c112233ffend\x11c123"),
            [(Some([0x11, 0x22, 0x33]), "end\x11c123")]
        );
        // A code ending at the string's end is a code: the run before it is
        // kept and there is nothing after.
        assert_eq!(name_spans("end\x11c12345678"), [(None, "end")]);
        // Seven bytes after `\x11c` is not a code: kept as text.
        assert_eq!(name_spans("end\x11c1234567"), [(None, "end\x11c1234567")]);
        // A lone `\x11` at the end is not a code's start: kept.
        assert_eq!(name_spans("end\x11"), [(None, "end\x11")]);
        // A trailing `\x11d` is a code: it resets and leaves no run after.
        assert_eq!(
            name_spans("\x11c112233ffcolor\x11d"),
            [(Some([0x11, 0x22, 0x33]), "color")]
        );
        // Upper-case hex is hex.
        assert_eq!(
            name_spans("\x11cA0B0C8FFNAME"),
            [(Some([0xa0, 0xb0, 0xc8]), "NAME")]
        );
        // The runs joined are the display name.
        for name in [
            "\x11cffd28cffROAD\x11cf389adffTRIP",
            "\x11ca51818ffRESIZE\x11cdddf3bffTHE\x11ca51818ffFLASH!",
            "plain",
        ] {
            let joined: String = name_spans(name).into_iter().map(|(_, text)| text).collect();
            assert_eq!(joined, display_name(name), "{name:?}");
        }
    }

    #[test]
    fn name_color_reads_a_leading_code_only() {
        assert_eq!(name_color("\x11ccc9900ffJones"), Some("cc9900ff"));
        assert_eq!(name_color("Jones"), None);
        // A `c` second byte is no code without the `\x11` before it.
        assert_eq!(name_color("McAllister"), None);
        // A code shorter than eight bytes at the name's end is not a code.
        assert_eq!(name_color("\x11ccc99"), None);
        // A code not at the start is no leading code.
        assert_eq!(name_color("J\x11ccc9900ffones"), None);
    }

    #[test]
    fn codes_at_the_names_edge_are_stripped_or_kept_whole() {
        // A complete \x11c code as the name's last ten bytes (the
        // `i + end <= len` boundary, inclusive) is still a code.
        assert_eq!(display_name("ab\x11c12345678"), "ab");
        // A lone 0x11 as the last byte is not a code's start: kept.
        assert_eq!(display_name("ab\x11"), "ab\x11");
        // A \x11c code whose ten bytes end inside a multi-byte char is not
        // a code: the boundary check rejects it even though the span fits.
        assert_eq!(display_name("x\x11c1234567€"), "x\x11c1234567€");
        // A \x11d reset two bytes after a two-byte char, before a
        // three-byte one: the boundary is checked at i + 2 (just past the
        // reset), not i - 2 (inside 'é').
        assert_eq!(display_name("aéx\x11d€y"), "aéx€y");
    }
}
