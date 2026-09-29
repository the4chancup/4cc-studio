//! `display_name`: a raw save name without its colour codes. `name` keeps the
//! raw form for the editor's colour rendering; this is the text consumers show.

/// The name without its colour codes: every `\x11c` plus the exactly eight
/// bytes that follow it (any bytes — the game does not check they are hex, so
/// a code at the string's end shorter than that is not a code and stays) and
/// every two-byte `\x11d` reset. Any other `0x11` sequence is untouched.
/// Stripping happens on bytes at char boundaries only, so the result is always
/// valid UTF-8.
pub fn display_name(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let strip = if bytes[i] == 0x11 && i + 1 < bytes.len() && bytes[i + 1] == b'c' {
            Some(10).filter(|end| i + end <= bytes.len() && raw.is_char_boundary(i + end))
        } else if bytes[i] == 0x11 && i + 1 < bytes.len() && bytes[i + 1] == b'd' {
            Some(2).filter(|end| raw.is_char_boundary(i + end))
        } else {
            None
        };
        match strip {
            Some(len) => i += len,
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8(out).expect("stripping at char boundaries keeps UTF-8 valid")
}

#[cfg(test)]
mod tests {
    use super::*;

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
