//! A face's `face_diff.bin`, the face parameter file the game reads beside a face's models:
//! decoded from the text a `face_diff.xml` gives it in, and checked to be a face diff the game
//! can read (`player_folders.md` "`face_diff.xml`"). The deep pass checks a folder's face diff
//! in either form; packing decodes a `face_diff.xml` into the bytes it packs. A member's own
//! `face.xml` gives it as a `<dif>` element, decoded the same way (`from_dif`).

use std::fmt;

use base64::engine::general_purpose::STANDARD;
use base64::{DecodeError, Engine};

/// Why a face folder's face diff cannot be used, as one sentence a member can act on: the
/// `reason` of `face_diff_invalid`.
#[derive(Debug)]
pub(crate) enum FaceDiffError {
    /// A `face_diff.xml` that is not UTF-8 text.
    NotUtf8,
    /// A `face_diff.xml` read as XML (its first non-blank character is `<`) that does not parse.
    Xml(roxmltree::Error),
    /// A `face_diff.xml` whose root element, named here, is not `<dif>`.
    RootNotDif(String),
    /// A `<dif>` holding child elements, Konami's structured form, which is not supported.
    DifHoldsElements,
    /// A `face_diff.xml` with no base64 text, blank or empty.
    NoBase64,
    /// Base64 text that does not decode.
    Base64(DecodeError),
    /// A face diff shorter than its fixed header, its length given.
    ShorterThanHeader(usize),
    /// A face diff not opening with the magic `FACE`.
    NotFace,
    /// A face diff shorter than the length its header gives: the game would read past its end.
    ShorterThanItsCounts {
        /// The file's length.
        len: usize,
        /// The length its header gives.
        expected: u64,
    },
}

/// The length of a face diff's fixed header, which ends with the two counts the rest of the
/// file is sized by.
const HEADER_LEN: usize = 0x50;

/// Checks that `bytes` are a face diff the game can read: the magic `FACE`, and at least the
/// length its header gives, `0xF0 + 0x10 × count(0x48) + 0x20 × count(0x4C)`.
pub(crate) fn check(bytes: &[u8]) -> Result<(), FaceDiffError> {
    if bytes.len() < HEADER_LEN {
        return Err(FaceDiffError::ShorterThanHeader(bytes.len()));
    }
    if !bytes.starts_with(b"FACE") {
        return Err(FaceDiffError::NotFace);
    }
    // In `u64`, where even the largest counts cannot overflow.
    let expected = 0xF0 + 0x10 * count_at(bytes, 0x48) + 0x20 * count_at(bytes, 0x4C);
    // At least, not exactly: face diffs longer than their header gives are common in cups'
    // CPKs (`player_folders.md` "`face_diff.xml`"), so only a shorter one is refused. A length
    // too large for this platform's `usize` is one no file here reaches either.
    let long_enough = usize::try_from(expected).is_ok_and(|expected| bytes.len() >= expected);
    if !long_enough {
        return Err(FaceDiffError::ShorterThanItsCounts {
            len: bytes.len(),
            expected,
        });
    }
    Ok(())
}

/// The `u32` little-endian count at `offset` of a face diff's header, which `check` has
/// already found long enough to hold it.
fn count_at(bytes: &[u8], offset: usize) -> u64 {
    let field: [u8; 4] = bytes[offset..offset + 4]
        .try_into()
        .expect("a four-byte range is a four-byte array");
    u64::from(u32::from_le_bytes(field))
}

/// The `face_diff.bin` that the text of a `face_diff.xml` holds: base64 (standard alphabet,
/// padded, whitespace anywhere), alone in the file or as the text of a `<dif>` root element,
/// decoded and checked (`check`). A file whose first non-blank character is `<` is read as
/// XML; a UTF-8 byte order mark is skipped.
pub(crate) fn from_xml(bytes: &[u8]) -> Result<Vec<u8>, FaceDiffError> {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let text = std::str::from_utf8(bytes).map_err(|_| FaceDiffError::NotUtf8)?;
    let text = text.trim_start_matches(|c: char| c.is_ascii_whitespace());
    if !text.starts_with('<') {
        return decoded(text);
    }
    let document = roxmltree::Document::parse(text).map_err(FaceDiffError::Xml)?;
    let root = document.root_element();
    let name = root.tag_name().name();
    if name != "dif" {
        return Err(FaceDiffError::RootNotDif(name.to_owned()));
    }
    from_dif(root)
}

/// The `face_diff.bin` a `<dif>` element holds as base64 text, decoded and checked (`check`):
/// the root of a `face_diff.xml`, or a child of a member's own `face.xml`'s `<config>`.
pub(crate) fn from_dif(dif: roxmltree::Node) -> Result<Vec<u8>, FaceDiffError> {
    if dif.children().any(|node| node.is_element()) {
        return Err(FaceDiffError::DifHoldsElements);
    }
    // Text nodes only: a comment's text is not the payload.
    let payload: String = dif
        .children()
        .filter(|node| node.is_text())
        .filter_map(|node| node.text())
        .collect();
    decoded(&payload)
}

/// The face diff the base64 text `payload` encodes (standard alphabet, padded, whitespace
/// anywhere), decoded and checked (`check`).
fn decoded(payload: &str) -> Result<Vec<u8>, FaceDiffError> {
    let base64: Vec<u8> = payload
        .bytes()
        .filter(|byte| !byte.is_ascii_whitespace())
        .collect();
    // Nothing decodes to nothing, which `check` would report as a short face diff: the
    // member's mistake is the missing text.
    if base64.is_empty() {
        return Err(FaceDiffError::NoBase64);
    }
    let decoded = STANDARD.decode(&base64).map_err(FaceDiffError::Base64)?;
    check(&decoded)?;
    Ok(decoded)
}

impl fmt::Display for FaceDiffError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FaceDiffError::NotUtf8 => write!(f, "the file is not UTF-8 text"),
            FaceDiffError::Xml(error) => write!(f, "the file is not well-formed XML: {error}"),
            FaceDiffError::RootNotDif(name) => {
                write!(f, "its root element is <{name}>, not <dif>")
            }
            FaceDiffError::DifHoldsElements => {
                write!(f, "its <dif> holds elements instead of base64 text")
            }
            FaceDiffError::NoBase64 => write!(f, "the file holds no base64 text"),
            FaceDiffError::Base64(DecodeError::InvalidByte(_, byte)) => write!(
                f,
                "the base64 text holds {:?} where base64 cannot",
                char::from(*byte)
            ),
            FaceDiffError::Base64(
                DecodeError::InvalidLength(_)
                | DecodeError::InvalidLastSymbol { .. }
                | DecodeError::InvalidPadding,
            ) => write!(f, "the base64 text is cut short or wrongly padded"),
            FaceDiffError::ShorterThanHeader(len) => write!(
                f,
                "the face diff is {len} bytes long, shorter than its 80-byte header"
            ),
            FaceDiffError::NotFace => write!(f, "the face diff does not start with FACE"),
            FaceDiffError::ShorterThanItsCounts { len, expected } => write!(
                f,
                "the face diff is {len} bytes long, but its header gives {expected}: the game \
                 would read past its end"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::Templates;

    const DIF_XML: &[u8] = include_bytes!("../tests/fixtures/face_diff/dif.xml");
    const DIF_BIN: &[u8] = include_bytes!("../tests/fixtures/face_diff/dif.bin");
    const PLAIN_XML: &[u8] = include_bytes!("../tests/fixtures/face_diff/plain.xml");
    const PLAIN_BIN: &[u8] = include_bytes!("../tests/fixtures/face_diff/plain.bin");

    /// The reason `result` failed with.
    fn reason<T: fmt::Debug>(result: Result<T, FaceDiffError>) -> String {
        result.unwrap_err().to_string()
    }

    #[test]
    fn a_face_diff_xml_decodes_to_the_bin_it_was_encoded_from() {
        assert_eq!(from_xml(DIF_XML).unwrap(), DIF_BIN);
        assert_eq!(from_xml(PLAIN_XML).unwrap(), PLAIN_BIN);
    }

    #[test]
    fn a_bom_and_blank_lines_before_the_text_are_skipped() {
        let bom = [b"\xEF\xBB\xBF".as_slice(), PLAIN_XML].concat();
        assert_eq!(from_xml(&bom).unwrap(), PLAIN_BIN);
        let blank_lines = [b"\r\n\r\n".as_slice(), DIF_XML].concat();
        assert_eq!(from_xml(&blank_lines).unwrap(), DIF_BIN);
    }

    #[test]
    fn a_face_diff_at_least_as_long_as_its_header_gives_passes_the_check() {
        // 944 bytes, the length its header gives.
        check(DIF_BIN).unwrap();
        // 960 bytes with a header that gives 944: a class of files cups ship.
        check(PLAIN_BIN).unwrap();
        check(Templates::embedded().face_diff()).unwrap();
    }

    #[test]
    fn each_malformed_face_diff_xml_is_refused_with_its_reason() {
        assert_eq!(
            reason(from_xml(b"<config>RkFDRQ==</config>")),
            "its root element is <config>, not <dif>"
        );
        assert_eq!(
            reason(from_xml(b"<dif><a/></dif>")),
            "its <dif> holds elements instead of base64 text"
        );
        assert_eq!(
            reason(from_xml(b"<dif> </dif>")),
            "the file holds no base64 text"
        );
        assert_eq!(reason(from_xml(b"")), "the file holds no base64 text");
        assert_eq!(reason(from_xml(b"\xFF\xFE")), "the file is not UTF-8 text");
        assert_eq!(
            reason(from_xml(b"<dif>RkFD")),
            "the file is not well-formed XML: the root node was opened but never closed"
        );

        let payload = STANDARD.encode(DIF_BIN);
        let starred = format!("{}*{}", &payload[..100], &payload[101..]);
        assert_eq!(
            reason(from_xml(starred.as_bytes())),
            "the base64 text holds '*' where base64 cannot"
        );
        let unpadded = &payload[..payload.len() - 1];
        assert_eq!(
            reason(from_xml(unpadded.as_bytes())),
            "the base64 text is cut short or wrongly padded"
        );
    }

    #[test]
    fn each_malformed_face_diff_is_refused_with_its_reason_in_either_form() {
        let cut = &DIF_BIN[..943];
        let mut not_face = DIF_BIN.to_vec();
        not_face[0] = b'G';
        let cases: [(&[u8], &str); 4] = [
            (
                &DIF_BIN[..0x4F],
                "the face diff is 79 bytes long, shorter than its 80-byte header",
            ),
            // The header alone holds both counts: what it lacks is what they size.
            (
                &DIF_BIN[..0x50],
                "the face diff is 80 bytes long, but its header gives 944: the game would \
                 read past its end",
            ),
            (&not_face, "the face diff does not start with FACE"),
            (
                cut,
                "the face diff is 943 bytes long, but its header gives 944: the game would \
                 read past its end",
            ),
        ];
        for (bytes, expected) in cases {
            assert_eq!(reason(check(bytes)), expected);
            assert_eq!(
                reason(from_xml(STANDARD.encode(bytes).as_bytes())),
                expected
            );
        }
    }
}
