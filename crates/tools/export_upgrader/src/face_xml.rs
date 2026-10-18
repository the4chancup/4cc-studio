//! Step 7's respelling of a member `face.xml`, and step 1's unwrapping
//! (`plans/export_upgrader.md` steps 1 and 7, `DECISIONS.md` 2026-10-18
//! "a WESYS-wrapped `face.xml` is written unwrapped"): `path` and
//! `material` attributes whose values name the player folder's own or the
//! team's Common get the kit magic respelled; everything else, the `<dif>`
//! included, is byte for byte as written — a wrapped one is written
//! unwrapped.

use std::borrow::Cow;
use std::collections::BTreeSet;

use aesthetics_export::common_link_name;
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

/// The folded names of the `Players/<folder>/` folders holding a member
/// `face.xml`, whose files are left entirely as named (step 7's model
/// renames do not reach them, subfolders included).
pub(crate) fn xml_folders(copies: &[(String, String)]) -> BTreeSet<String> {
    copies
        .iter()
        .filter(|(_, output)| is_member_face_xml(output))
        .filter_map(|(_, output)| {
            output["Players/".len()..]
                .split_once('/')
                .map(|(folder, _)| vtree::fold_name(folder))
        })
        .collect()
}

/// `input`'s `face.xml` read and unwrapped, as both passes over it read it
/// (`face_xml_rewrites`, `kept_common_files`): whether it was WESYS-wrapped
/// (a rewrite is written unwrapped) and the payload; `None` when the read
/// or the unwrap fails — such a file is left as written and names nothing.
fn read_face_xml(
    input: &str,
    read: &mut impl FnMut(&str) -> Result<Vec<u8>, SourceFailure>,
) -> Option<(bool, Vec<u8>)> {
    let bytes = read(input).ok()?;
    let payload = wezlib::decompress_if_wrapped(&bytes).ok()?.into_owned();
    Some((wezlib::is_wrapped(&bytes), payload))
}

/// `payload`'s XML text past a BOM, and whether a BOM stood before it;
/// `None` when `payload` is not UTF-8.
fn xml_text(payload: &[u8]) -> Option<(bool, &str)> {
    let text = std::str::from_utf8(payload).ok()?;
    Some(match text.strip_prefix('\u{feff}') {
        Some(text) => (true, text),
        None => (false, text),
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
        let Some((wrapped, payload)) = read_face_xml(input, &mut read) else {
            continue;
        };
        let respelled =
            xml_text(&payload).and_then(|(bom, text)| match respell_face_xml(text, team) {
                Cow::Owned(respelled) => {
                    let mut out = String::new();
                    if bom {
                        out.push('\u{feff}');
                    }
                    out.push_str(&respelled);
                    Some(out.into_bytes())
                }
                Cow::Borrowed(_) => None,
            });
        let new = match respelled {
            Some(new) => new,
            // Unchanged, or not UTF-8: a wrapped file still lands
            // unwrapped, untouched.
            None if wrapped => payload,
            None => continue,
        };
        rewrites.push((output.clone(), new));
    }
    rewrites
}

/// `value` minus `prefix` when it starts with it (ASCII
/// case-insensitive).
fn strip_prefix_ci<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    value
        .get(..prefix.len())
        .filter(|head| head.eq_ignore_ascii_case(prefix))
        .map(|_| &value[prefix.len()..])
}

/// The `Common/` path a `face.xml` `path`/`material` value names, as the
/// compiler reads it (`model/character/uniform/common/<segment>/<path>`,
/// the file name's `*` read as `win32`, directories as written), respelled
/// as the `Common/` outputs are (`Spelling::Name`) and folded; `None`
/// when it names none (a value naming a folder gives a path no file has,
/// which keeps nothing).
fn common_below(value: &str, team: TeamId) -> Option<String> {
    let rest = strip_prefix_ci(value, COMMON_PREFIX)?;
    let (_segment, path) = rest.split_once('/')?;
    let path = match path.rsplit_once('/') {
        Some((dir, name)) => format!("{dir}/{}", name.replace('*', "win32")),
        None => path.replace('*', "win32"),
    };
    Some(vtree::fold_name(&respell(&path, Spelling::Name, team)))
}

/// The `Common/` paths (folded, below `Common/`) that keep their names
/// through step 7's renames (`plans/export_upgrader.md` step 7,
/// `DECISIONS.md` 2026-10-18 "a pre-Fox leading type moves only when `_`
/// follows it…" (4)): those a member `face.xml`'s `path`/`material` values
/// name, and those a `.common` link in such a folder names — the link is
/// left as written with its folder, so its target keeps its name too.
pub(crate) fn kept_common_files(
    copies: &[(String, String)],
    team: TeamId,
    mut read: impl FnMut(&str) -> Result<Vec<u8>, SourceFailure>,
) -> BTreeSet<String> {
    let xml = xml_folders(copies);
    let mut kept = BTreeSet::new();
    for (input, output) in copies {
        if !is_member_face_xml(output) {
            continue;
        }
        let Some((_, payload)) = read_face_xml(input, &mut read) else {
            continue;
        };
        let Some((_, text)) = xml_text(&payload) else {
            continue;
        };
        let Ok(document) = Document::parse(text) else {
            continue;
        };
        for attribute in document
            .descendants()
            .filter(|node| node.is_element())
            .flat_map(|element| element.attributes())
        {
            if matches!(attribute.name(), "path" | "material")
                && let Some(below) = common_below(&text[attribute.range_value()], team)
            {
                kept.insert(below);
            }
        }
    }
    for (_, output) in copies {
        let Some(rest) = output.strip_prefix("Players/") else {
            continue;
        };
        let Some((folder, below)) = rest.split_once('/') else {
            continue;
        };
        if !xml.contains(&vtree::fold_name(folder)) {
            continue;
        }
        let name = below.rsplit('/').next().unwrap_or(below);
        let Some(target) = common_link_name(name) else {
            continue;
        };
        let suffix_len = name.len() - target.len();
        kept.insert(vtree::fold_name(&below[..below.len() - suffix_len]));
    }
    kept
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

    #[test]
    fn kept_common_files_names_the_values_common_paths() {
        let copies = vec![("Faces/x".to_owned(), "Players/07 - A/face.xml".to_owned())];
        let text = "<config><model path=\"model/character/uniform/common/o/oral_hand_l_*.model\" material=\"model/character/uniform/common/XXX/sub/x.mtl\"/></config>";
        let mut read = reader(vec![("Faces/x".to_owned(), text.as_bytes().to_vec())]);
        let kept = kept_common_files(&copies, team(), &mut read);
        assert_eq!(
            kept.iter().collect::<Vec<_>>(),
            ["oral_hand_l_win32.model", "sub/x.mtl"]
        );
    }

    #[test]
    fn kept_common_files_ignores_local_and_bare_values() {
        let copies = vec![("Faces/x".to_owned(), "Players/07 - A/face.xml".to_owned())];
        let text = "<config><model path=\"./x.model\" material=\"model/character/uniform/common/\"/><model path=\"model/character/uniform/common/XXX\"/></config>";
        let mut read = reader(vec![("Faces/x".to_owned(), text.as_bytes().to_vec())]);
        assert!(kept_common_files(&copies, team(), &mut read).is_empty());
    }

    #[test]
    fn kept_common_files_counts_a_link_in_the_xml_folder() {
        let copies = vec![
            ("Faces/x".to_owned(), "Players/07 - A/face.xml".to_owned()),
            (
                "Faces/x.link".to_owned(),
                "Players/07 - A/oral_y_win32.model.common".to_owned(),
            ),
        ];
        let mut read = reader(vec![("Faces/x".to_owned(), b"<config/>".to_vec())]);
        let kept = kept_common_files(&copies, team(), &mut read);
        assert_eq!(kept.iter().collect::<Vec<_>>(), ["oral_y_win32.model"]);
    }

    #[test]
    fn kept_common_files_reads_a_wrapped_face_xml() {
        let copies = vec![("Faces/x".to_owned(), "Players/07 - A/face.xml".to_owned())];
        let wrapped = wezlib::compress(
            "<config><model path=\"model/character/uniform/common/o/oral_hand_l_*.model\"/></config>"
                .as_bytes(),
        );
        let mut read = reader(vec![("Faces/x".to_owned(), wrapped)]);
        let kept = kept_common_files(&copies, team(), &mut read);
        assert_eq!(kept.iter().collect::<Vec<_>>(), ["oral_hand_l_win32.model"]);
    }
}
