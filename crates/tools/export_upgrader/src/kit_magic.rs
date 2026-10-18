//! Step 7's legacy kit-dependent magic and its Studio spelling (`plans/
//! export_upgrader.md` step 7, `DECISIONS.md` 2026-10-18 "the kit magic's
//! grammar, where it is renamed, and a Common kit set's link"): `u0???pN`
//! (`u0` and three ASCII letters or digits, `p` and a digit) becomes
//! `kit<N>` in a path and `kitF` in a file or folder name, `u0???g`
//! becomes `u0<team>g`, matched in the letter case written, anywhere and
//! every occurrence. Only the upgrader reads the legacy spelling.

use std::borrow::Cow;
use std::collections::BTreeSet;

use studio_core::{Disposition, Message, Scope};
use teams_list::TeamId;

use crate::messages::{Code, INPUT, tool_message};
use crate::players::path_scope;

/// Where a respelled kit token stands: a file or folder name, or a path a
/// reference names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Spelling {
    /// `u0???p0` becomes `kitF`: the file a kit without a variant of its
    /// own loads.
    Name,
    /// `u0???p0` becomes `kitN`: the reference to the variant of the kit
    /// picked.
    Path,
}

/// `text` with every legacy kit token respelled: `u0`, three ASCII letters
/// or digits, then `p` and a digit `d` becomes `kit<d>` (`d` = 0: `kitF`
/// or `kitN` by `spelling`); `u0`, three ASCII letters or digits, then `g`
/// becomes `u0<team, three digits>g`. Matched in the letter case written,
/// anywhere, left to right, every occurrence. Borrowed when nothing
/// changed.
pub(crate) fn respell<'a>(text: &'a str, spelling: Spelling, team: TeamId) -> Cow<'a, str> {
    let bytes = text.as_bytes();
    let mut out: Option<String> = None;
    let mut last = 0;
    for at in 0..bytes.len() {
        // A match starting inside the one before it is not taken (the
        // earlier one wins, left to right).
        if at < last {
            continue;
        }
        // `u0` + three ASCII letters or digits + `p`/`g`: the `u` is as
        // written, lowercase.
        if bytes[at] != b'u' || bytes.get(at + 1) != Some(&b'0') {
            continue;
        }
        if bytes
            .get(at + 2..at + 5)
            .is_none_or(|chars| !chars.iter().all(|byte| byte.is_ascii_alphanumeric()))
        {
            continue;
        }
        let (replacement, end): (Cow<str>, usize) = match bytes.get(at + 5) {
            Some(&b'p') => match bytes.get(at + 6) {
                Some(&b'0') => (
                    Cow::Borrowed(match spelling {
                        Spelling::Name => "kitF",
                        Spelling::Path => "kitN",
                    }),
                    at + 7,
                ),
                Some(digit) if digit.is_ascii_digit() => {
                    (Cow::Owned(format!("kit{}", char::from(*digit))), at + 7)
                }
                _ => continue,
            },
            Some(&b'g') => (Cow::Owned(format!("u0{:03}g", team.get())), at + 6),
            _ => continue,
        };
        let buffer = out.get_or_insert_with(|| String::with_capacity(text.len()));
        buffer.push_str(&text[last..at]);
        buffer.push_str(&replacement);
        last = end;
    }
    match out {
        Some(mut out) => {
            out.push_str(&text[last..]);
            Cow::Owned(out)
        }
        None => Cow::Borrowed(text),
    }
}

/// The copies with their output names respelled (`Spelling::Name`) below a
/// player folder (`Players/<folder>/`), a shared folder (`Boots/<name>/`,
/// `Gloves/<name>/`) or `Common/`; every other output, and those folders'
/// own names, as they are. An output that, compared with
/// `vtree::fold_name`, an earlier copy in the list already takes
/// is left out and its input reported `unmigrated_content` (E,
/// `Disposition::DropFile`, the fallback's scope and empty context).
pub(crate) fn respell_outputs(
    copies: Vec<(String, String)>,
    team: TeamId,
) -> (Vec<(String, String)>, Vec<Message>) {
    let mut out = Vec::with_capacity(copies.len());
    let mut taken: BTreeSet<String> = BTreeSet::new();
    let mut findings = Vec::new();
    for (from, to) in copies {
        let to = match to.split_once('/') {
            Some((top, rest)) => {
                // The path the magic can reach: `Players`/`Boots`/`Gloves`
                // keep their folder's own name and respell below it;
                // `Common/` respells the whole path below.
                let (keep, below) = match top {
                    "Players" | "Boots" | "Gloves" => match rest.split_once('/') {
                        Some((_folder, below)) => (&to[..to.len() - below.len()], below),
                        None => (&to[..], ""),
                    },
                    "Common" => ("Common/", rest),
                    _ => (&to[..], ""),
                };
                match respell(below, Spelling::Name, team) {
                    Cow::Owned(below) => format!("{keep}{below}"),
                    Cow::Borrowed(_) => to.clone(),
                }
            }
            _ => to.clone(),
        };
        if !taken.insert(vtree::fold_name(&to)) {
            findings.push(tool_message(
                Code::UnmigratedContent,
                Scope::File {
                    export_id: INPUT,
                    path: path_scope(&from),
                },
                Disposition::DropFile,
                vec![],
            ));
            continue;
        }
        out.push((from, to));
    }
    (out, findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn team() -> TeamId {
        TeamId::new(714).unwrap()
    }

    #[test]
    fn every_u0_p_token_becomes_kit_d() {
        assert_eq!(respell("u0XXXp1.dds", Spelling::Name, team()), "kit1.dds");
        assert_eq!(
            respell("skin_u0771p3.dds", Spelling::Name, team()),
            "skin_kit3.dds"
        );
        // `p0` is `kitF` in a name, `kitN` in a path.
        assert_eq!(respell("u0875p0.dds", Spelling::Name, team()), "kitF.dds");
        assert_eq!(
            respell("./u0875p0/armor.dds", Spelling::Path, team()),
            "./kitN/armor.dds"
        );
        assert_eq!(
            respell("u0XXXg1.dds", Spelling::Name, team()),
            "u0714g1.dds"
        );
        // Every occurrence, anywhere.
        assert_eq!(
            respell("u0XXXp1_u0XXXg1", Spelling::Name, team()),
            "kit1_u0714g1"
        );
    }

    #[test]
    fn a_non_token_is_borrowed_unchanged() {
        for text in [
            "U0XXXP1.dds",
            "u0X-Xp1.dds",
            "dummy_kit.dds",
            "u0XXXpants.dds",
            "a0XXXp1.dds",
            "uXXXXp1.dds",
            "kit1.dds",
            "portrait.dds",
        ] {
            assert!(
                matches!(respell(text, Spelling::Name, team()), Cow::Borrowed(_)),
                "{text}"
            );
        }
    }

    #[test]
    fn a_match_starting_inside_a_match_is_not_taken() {
        // The `g` match at 0 ends at 6; the `p` at 7 starts a token at 2 —
        // inside the first match, left to right.
        assert_eq!(respell("u0u0ZgAp1", Spelling::Name, team()), "u0714gAp1");
    }

    #[test]
    fn player_and_common_outputs_respell_their_rest() {
        let (copies, findings) = respell_outputs(
            vec![
                ("a".into(), "Players/07 - A/u0XXXp1.dds".into()),
                ("b".into(), "Players/07 - A/u0XXXp0/armor.dds".into()),
                ("c".into(), "Common/ltd/u0XXXp5/a.dds".into()),
                ("d".into(), "Common/u0XXXg1.dds".into()),
                ("e".into(), "Boots/Crocs/body_d_u0XXXp0.dds".into()),
                ("f".into(), "Portraits/player_07.dds".into()),
                ("g".into(), "Kits/p1/kit.dds".into()),
                ("h".into(), "Players/07 - u0XXXp1/x.dds".into()),
            ],
            team(),
        );
        assert!(findings.is_empty());
        assert_eq!(
            copies,
            vec![
                ("a".to_owned(), "Players/07 - A/kit1.dds".to_owned()),
                ("b".to_owned(), "Players/07 - A/kitF/armor.dds".to_owned()),
                ("c".to_owned(), "Common/ltd/kit5/a.dds".to_owned()),
                ("d".to_owned(), "Common/u0714g1.dds".to_owned()),
                ("e".to_owned(), "Boots/Crocs/body_d_kitF.dds".to_owned()),
                ("f".to_owned(), "Portraits/player_07.dds".to_owned()),
                ("g".to_owned(), "Kits/p1/kit.dds".to_owned()),
                ("h".to_owned(), "Players/07 - u0XXXp1/x.dds".to_owned()),
            ]
        );
    }

    #[test]
    fn the_second_folded_output_is_reported_and_left_out() {
        let (copies, findings) = respell_outputs(
            vec![
                ("a".into(), "Players/07 - A/u0XXXp1.dds".into()),
                ("b".into(), "Players/07 - A/u0714p1.dds".into()),
            ],
            team(),
        );
        assert_eq!(
            copies,
            vec![("a".to_owned(), "Players/07 - A/kit1.dds".to_owned())]
        );
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code.code.as_ref(), "unmigrated_content");
        assert_eq!(
            findings[0].scope,
            Scope::File {
                export_id: INPUT,
                path: path_scope("b")
            }
        );
        assert_eq!(findings[0].disposition, Disposition::DropFile);
    }
}
