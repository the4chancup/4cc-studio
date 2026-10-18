//! Step 7's respelling of a member `face.xml`, and step 1's unwrapping
//! (`plans/export_upgrader.md` steps 1 and 7, `DECISIONS.md` 2026-10-18
//! "a WESYS-wrapped `face.xml` is written unwrapped"): `path` and
//! `material` attributes whose values name the player folder's own or the
//! team's Common get the kit magic respelled; everything else, the `<dif>`
//! included, is byte for byte as written — a wrapped one is written
//! unwrapped.

use std::borrow::Cow;

use roxmltree::Document;
use teams_list::TeamId;

use crate::kit_magic::{Spelling, respell};
use crate::source::SourceFailure;

/// The `model/character/uniform/common/` prefix the team's `Common/`
/// stands in, as `materials.rs` spells it.
const COMMON_PREFIX: &str = "model/character/uniform/common/";

/// Whether `output` is a member `face.xml`: `Players/<folder>/face.xml`
/// directly in the player folder (a subfolder's is
/// `file_type_disallowed`, never a member).
fn is_member_face_xml(output: &str) -> bool {
    let Some(rest) = output.strip_prefix("Players/") else {
        return false;
    };
    rest.rsplit_once('/').is_some_and(|(_, name)| {
        name.eq_ignore_ascii_case("face.xml") && !rest[..rest.len() - name.len() - 1].contains('/')
    })
}

/// The member `face.xml` files to write instead of copying: output path
/// and contents. One is here when its input is WESYS-wrapped (written
/// unwrapped, step 1) or a `path`/`material` value changed.
pub(crate) fn face_xml_rewrites(
    copies: &[(String, String)],
    team: TeamId,
    mut read: impl FnMut(&str) -> Result<Vec<u8>, SourceFailure>,
) -> Vec<(String, Vec<u8>)> {
    let mut rewrites = Vec::new();
    for (input, output) in copies {
        if !is_member_face_xml(output) {
            continue;
        }
        // The copy loop's own read reports a failure; a file that does not
        // unwrap is copied as it is (the compile reports it).
        let Ok(bytes) = read(input) else {
            continue;
        };
        let wrapped = wezlib::is_wrapped(&bytes);
        let payload = match wezlib::decompress_if_wrapped(&bytes) {
            Ok(payload) => payload,
            Err(_) => continue,
        };
        let text = match std::str::from_utf8(&payload) {
            Ok(text) => text,
            // Not UTF-8: a wrapped file still lands unwrapped, untouched.
            Err(_) => {
                if wrapped {
                    rewrites.push((output.clone(), payload.into_owned()));
                }
                continue;
            }
        };
        let (bom, stripped) = match text.strip_prefix('\u{feff}') {
            Some(stripped) => (true, stripped),
            None => (false, text),
        };
        let respelled: Cow<[u8]> = match respell_face_xml(stripped, team) {
            Cow::Owned(text) => {
                let mut out = if bom {
                    String::from("\u{feff}")
                } else {
                    String::new()
                };
                out.push_str(&text);
                Cow::Owned(out.into_bytes())
            }
            Cow::Borrowed(_) => Cow::Borrowed(&*payload),
        };
        if wrapped || matches!(respelled, Cow::Owned(_)) {
            rewrites.push((output.clone(), respelled.into_owned()));
        }
    }
    rewrites
}

/// `text`, a parseable `face.xml`, with the `path` and `material`
/// attribute values that name the player folder's own (`./`) or the
/// team's Common (`model/character/uniform/common/`, any letter case) kit
/// magic respelled; `Cow::Borrowed` when nothing changed.
fn respell_face_xml<'a>(text: &'a str, team: TeamId) -> Cow<'a, str> {
    let Ok(document) = Document::parse(text) else {
        return Cow::Borrowed(text);
    };
    let edits: Vec<(std::ops::Range<usize>, String)> = document
        .descendants()
        .filter(|node| node.is_element())
        .flat_map(|element| element.attributes())
        .filter_map(|attribute| {
            if !matches!(attribute.name(), "path" | "material") {
                return None;
            }
            let raw = &text[attribute.range_value()];
            if !(raw.starts_with("./")
                || raw
                    .get(..COMMON_PREFIX.len())
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(COMMON_PREFIX)))
            {
                return None;
            }
            match respell(raw, Spelling::Path, team) {
                Cow::Owned(new) => Some((attribute.range_value(), new)),
                Cow::Borrowed(_) => None,
            }
        })
        .collect();
    if edits.is_empty() {
        return Cow::Borrowed(text);
    }
    let mut out = text.to_owned();
    // The ranges name the original text: last first, so an earlier splice
    // does not shift the ones after it.
    for (range, new) in edits.into_iter().rev() {
        out.replace_range(range, &new);
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn team() -> TeamId {
        TeamId::new(714).unwrap()
    }

    /// A `face.xml` in the real one's shape: an `<?xml` declaration, a
    /// `<config>` of multi-line `<model>` elements, comments, a `<dif>`
    /// block whose text mentions a kit token.
    fn face_xml_text() -> String {
        [
            "<?xml version=\"1.0\" encoding=\"UTF-8\" ?>",
            "<config>",
            "    <!-- The player's jacket: u0XXXp0/jacket_win32.model -->",
            "    <model level=\"0\" type=\"parts\" path=\"./u0XXXp0/jacket_win32.model\"",
            "                                      material=\"./face.mtl\"/>",
            "    <dif>u0XXXp1</dif>",
            "</config>",
        ]
        .join("\n")
    }

    fn reader(
        entries: Vec<(String, Vec<u8>)>,
    ) -> impl FnMut(&str) -> Result<Vec<u8>, SourceFailure> {
        move |path| {
            entries
                .iter()
                .find(|(p, _)| p == path)
                .map(|(_, bytes)| bytes.clone())
                .ok_or_else(|| SourceFailure {
                    path: path.to_owned(),
                    error: "not listed".to_owned(),
                })
        }
    }

    #[test]
    fn a_wrapped_face_xml_is_written_unwrapped_with_path_respelled() {
        let copies = vec![("Faces/x".to_owned(), "Players/07 - A/face.xml".to_owned())];
        let wrapped = wezlib::compress(face_xml_text().as_bytes());
        let mut read = reader(vec![("Faces/x".to_owned(), wrapped)]);
        let rewrites = face_xml_rewrites(&copies, team(), &mut read);
        assert_eq!(rewrites.len(), 1);
        let (_, bytes) = &rewrites[0];
        assert!(!wezlib::is_wrapped(bytes));
        let text = String::from_utf8(bytes.clone()).unwrap();
        assert!(
            text.contains("path=\"./kitN/jacket_win32.model\""),
            "{text}"
        );
        assert!(
            text.contains("<!-- The player's jacket: u0XXXp0/jacket_win32.model -->"),
            "{text}"
        );
        assert!(text.contains("<dif>u0XXXp1</dif>"), "{text}");
        assert!(text.contains("material=\"./face.mtl\""), "{text}");
        assert!(
            text.contains("<?xml version=\"1.0\" encoding=\"UTF-8\" ?>"),
            "{text}"
        );
    }

    #[test]
    fn a_common_path_s_u0xxxg_respells_the_team() {
        let copies = vec![("Faces/x".to_owned(), "Players/07 - A/face.xml".to_owned())];
        let text =
            "<face><model path=\"model/character/uniform/common/XXX/u0XXXg1_shirt.model\"/></face>";
        let mut read = reader(vec![("Faces/x".to_owned(), text.as_bytes().to_vec())]);
        let rewrites = face_xml_rewrites(&copies, team(), &mut read);
        assert_eq!(rewrites.len(), 1);
        let text = String::from_utf8(rewrites[0].1.clone()).unwrap();
        assert_eq!(
            text,
            "<face><model path=\"model/character/uniform/common/XXX/u0714g1_shirt.model\"/></face>"
        );
    }

    #[test]
    fn a_plain_face_xml_without_tokens_is_not_rewritten() {
        let copies = vec![("Faces/x".to_owned(), "Players/07 - A/face.xml".to_owned())];
        let text = "<face><model path=\"./face.model\"/></face>";
        let mut read = reader(vec![("Faces/x".to_owned(), text.as_bytes().to_vec())]);
        let rewrites = face_xml_rewrites(&copies, team(), &mut read);
        assert!(rewrites.is_empty());
    }

    #[test]
    fn a_wrapped_face_xml_without_tokens_is_written_unwrapped() {
        let copies = vec![("Faces/x".to_owned(), "Players/07 - A/face.xml".to_owned())];
        let text = "<face><model path=\"./face.model\"/></face>";
        let wrapped = wezlib::compress(text.as_bytes());
        let mut read = reader(vec![("Faces/x".to_owned(), wrapped)]);
        let rewrites = face_xml_rewrites(&copies, team(), &mut read);
        assert_eq!(rewrites.len(), 1);
        assert_eq!(rewrites[0].1, text.as_bytes().to_vec());
    }

    #[test]
    fn a_wrapped_non_xml_payload_is_written_as_is() {
        let copies = vec![("Faces/x".to_owned(), "Players/07 - A/face.xml".to_owned())];
        let wrapped = wezlib::compress(b"face");
        let mut read = reader(vec![("Faces/x".to_owned(), wrapped)]);
        let rewrites = face_xml_rewrites(&copies, team(), &mut read);
        assert_eq!(rewrites.len(), 1);
        assert_eq!(rewrites[0].1, b"face".to_vec());
    }

    #[test]
    fn a_model_with_two_respelled_attributes_stays_intact() {
        let copies = vec![("Faces/x".to_owned(), "Players/07 - A/face.xml".to_owned())];
        let text = "<config><model path=\"./jacket_u0XXXp0.model\" material=\"./jacket_u0XXXp0.mtl\"/></config>";
        let mut read = reader(vec![("Faces/x".to_owned(), text.as_bytes().to_vec())]);
        let rewrites = face_xml_rewrites(&copies, team(), &mut read);
        assert_eq!(rewrites.len(), 1);
        assert_eq!(
            String::from_utf8(rewrites[0].1.clone()).unwrap(),
            "<config><model path=\"./jacket_kitN.model\" material=\"./jacket_kitN.mtl\"/></config>"
        );
    }

    #[test]
    fn a_face_xml_in_a_subfolder_is_not_a_member() {
        let copies = vec![(
            "Faces/x".to_owned(),
            "Players/07 - A/kitF/face.xml".to_owned(),
        )];
        let mut read = reader(vec![("Faces/x".to_owned(), face_xml_text().into_bytes())]);
        assert!(face_xml_rewrites(&copies, team(), &mut read).is_empty());
    }
}
