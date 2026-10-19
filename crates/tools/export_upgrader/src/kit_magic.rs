//! Step 7's legacy kit-dependent magic and its Studio spelling (`plans/
//! export_upgrader.md` step 7, `DECISIONS.md` 2026-10-18 "the kit magic's
//! grammar, where it is renamed, and a Common kit set's link"): `u0???pN`
//! (`u0` and three ASCII letters or digits, `p` and a digit) becomes
//! `kit<N>` in a path and `kitF` in a file or folder name, `u0???g`
//! becomes `u0<path ID>g` (the team's, 999 for referees; kept as written
//! in model-folder mode), matched in the letter case written, anywhere
//! and every occurrence. In a path segment
//! holding a `u0???p` token, a Studio kit token already there
//! (`kit1_u0827p1.dds`, which some VGL26 exports write) gets its `k`
//! written `K` so the respelled token stays the only one. Only the upgrader
//! reads the legacy spelling.

use std::borrow::Cow;
use std::collections::BTreeSet;

use aesthetics_export::kit_token_starts;
use studio_core::{Disposition, Message, Scope};

use crate::messages::{Code, INPUT, tool_message};
use crate::players::path_scope;
use crate::upgrade::Identity;

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
/// becomes `u0<the identity's path ID, three digits>g`, or stays as
/// written when the identity has no path ID (model-folder mode). Matched
/// in the letter case written, anywhere, left to right, every occurrence.
/// In each
/// `/`-separated segment that holds a `p` token, every Studio kit token the
/// segment already holds gets its `k` written `K`, so the respelled token is
/// the segment's only one for `find_token` to read (`kit1_u0827p0.dds` →
/// `Kit1_kitN.dds`, `u0???g` not triggering it). Borrowed when nothing
/// changed.
pub(crate) fn respell<'a>(text: &'a str, spelling: Spelling, identity: &Identity) -> Cow<'a, str> {
    let bytes = text.as_bytes();
    // The edits, as (start, end, replacement) byte ranges of `text`,
    // disjoint: the legacy respells, and the `k`→`K` of every Studio kit
    // token in a segment holding a legacy `p` one.
    let mut edits: Vec<(usize, usize, Cow<str>)> = Vec::new();
    let mut last = 0;
    let mut marked = BTreeSet::new();
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
        let (replacement, end, legacy): (Cow<str>, usize, bool) = match bytes.get(at + 5) {
            Some(&b'p') => match bytes.get(at + 6) {
                Some(&b'0') => (
                    Cow::Borrowed(match spelling {
                        Spelling::Name => "kitF",
                        Spelling::Path => "kitN",
                    }),
                    at + 7,
                    true,
                ),
                Some(digit) if digit.is_ascii_digit() => (
                    Cow::Owned(format!("kit{}", char::from(*digit))),
                    at + 7,
                    true,
                ),
                _ => continue,
            },
            Some(&b'g') => match identity.path_id() {
                Some(id) => (Cow::Owned(format!("u0{id:03}g")), at + 6, false),
                // Model-folder mode has no team: the token stays as written.
                None => continue,
            },
            _ => continue,
        };
        if legacy {
            // The `/`-separated segment the token stands in.
            let start = text[..at].rfind('/').map_or(0, |slash| slash + 1);
            let end = text[at..].find('/').map_or(text.len(), |slash| at + slash);
            marked.insert((start, end));
        }
        edits.push((at, end, replacement));
        last = end;
    }
    for (start, end) in marked {
        for k in kit_token_starts(&text[start..end]) {
            edits.push((start + k, start + k + 1, Cow::Borrowed("K")));
        }
    }
    if edits.is_empty() {
        return Cow::Borrowed(text);
    }
    edits.sort_by_key(|(at, ..)| *at);
    let mut out = String::with_capacity(text.len() + edits.len() * 3);
    let mut cursor = 0;
    for (at, end, replacement) in edits {
        out.push_str(&text[cursor..at]);
        out.push_str(&replacement);
        cursor = end;
    }
    out.push_str(&text[cursor..]);
    Cow::Owned(out)
}

/// The copies with their output names respelled (`Spelling::Name`) below a
/// player folder (`Players/<folder>/`), a shared folder (`Boots/<name>/`,
/// `Gloves/<name>/`) or `Common/`; every other output, and those folders'
/// own names, as they are.
pub(crate) fn respell_outputs(
    copies: Vec<(String, String)>,
    identity: &Identity,
) -> Vec<(String, String)> {
    copies
        .into_iter()
        .map(|(from, to)| {
            let to = match to.split_once('/') {
                Some((top, rest)) => {
                    // The path the magic can reach: `Players`/`Boots`/
                    // `Gloves` keep their folder's own name and respell
                    // below it; `Common/` respells the whole path below.
                    let (keep, below) = match top {
                        "Players" | "Boots" | "Gloves" => match rest.split_once('/') {
                            Some((_folder, below)) => (&to[..to.len() - below.len()], below),
                            None => (&to[..], ""),
                        },
                        "Common" => ("Common/", rest),
                        _ => (&to[..], ""),
                    };
                    match respell(below, Spelling::Name, identity) {
                        Cow::Owned(below) => format!("{keep}{below}"),
                        Cow::Borrowed(_) => to,
                    }
                }
                _ => to,
            };
            (from, to)
        })
        .collect()
}

/// `copies` minus every output whose path, folded with
/// `vtree::fold_name`, an earlier copy in the list already takes; the
/// later input is reported `unmigrated_content` (E,
/// `Disposition::DropFile`, the fallback's scope and empty context).
pub(crate) fn drop_collisions(
    copies: Vec<(String, String)>,
) -> (Vec<(String, String)>, Vec<Message>) {
    let mut out = Vec::with_capacity(copies.len());
    let mut taken: BTreeSet<String> = BTreeSet::new();
    let mut findings = Vec::new();
    for (from, to) in copies {
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
    use teams_list::{TeamId, TeamName};

    use super::*;

    fn team() -> Identity {
        Identity::Team {
            id: TeamId::new(714).unwrap(),
            name: TeamName::new("co").unwrap(),
        }
    }

    #[test]
    fn every_u0_p_token_becomes_kit_d() {
        assert_eq!(respell("u0XXXp1.dds", Spelling::Name, &team()), "kit1.dds");
        assert_eq!(
            respell("skin_u0771p3.dds", Spelling::Name, &team()),
            "skin_kit3.dds"
        );
        // `p0` is `kitF` in a name, `kitN` in a path.
        assert_eq!(respell("u0875p0.dds", Spelling::Name, &team()), "kitF.dds");
        assert_eq!(
            respell("./u0875p0/armor.dds", Spelling::Path, &team()),
            "./kitN/armor.dds"
        );
        assert_eq!(
            respell("u0XXXg1.dds", Spelling::Name, &team()),
            "u0714g1.dds"
        );
        // Every occurrence, anywhere.
        assert_eq!(
            respell("u0XXXp1_u0XXXg1", Spelling::Name, &team()),
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
                matches!(respell(text, Spelling::Name, &team()), Cow::Borrowed(_)),
                "{text}"
            );
        }
    }

    #[test]
    fn a_match_starting_inside_a_match_is_not_taken() {
        // The `g` match at 0 ends at 6; the `p` at 7 starts a token at 2 —
        // inside the first match, left to right.
        assert_eq!(respell("u0u0ZgAp1", Spelling::Name, &team()), "u0714gAp1");
    }

    #[test]
    fn a_studio_token_beside_a_legacy_one_gets_its_k_written_capital() {
        assert_eq!(
            respell("kit1_u0827p0.dds", Spelling::Path, &team()),
            "Kit1_kitN.dds"
        );
        assert_eq!(
            respell("kit1_u0827p1.dds", Spelling::Name, &team()),
            "Kit1_kit1.dds"
        );
        assert_eq!(
            respell("kitN_u0XXXp2.dds", Spelling::Name, &team()),
            "KitN_kit2.dds"
        );
        assert_eq!(
            respell("kit1_kit2_u0XXXp3.dds", Spelling::Name, &team()),
            "Kit1_Kit2_kit3.dds"
        );
        assert_eq!(
            respell("skit1_u0XXXp1.dds", Spelling::Name, &team()),
            "skit1_kit1.dds"
        );
        // `Kit2` was never a token: its K stays.
        assert_eq!(
            respell("Kit2_u0XXXp1.dds", Spelling::Name, &team()),
            "Kit2_kit1.dds"
        );
        // Only the token's own segment is rewritten; a `g` token does not
        // trigger it.
        assert_eq!(
            respell("kit1/x_u0XXXp1.dds", Spelling::Name, &team()),
            "kit1/x_kit1.dds"
        );
        // A segment after a `/` is read from its first character.
        assert_eq!(
            respell("ltd/kit1_u0XXXp1.dds", Spelling::Name, &team()),
            "ltd/Kit1_kit1.dds"
        );
        assert_eq!(
            respell("kit1_u0XXXg1.dds", Spelling::Name, &team()),
            "kit1_u0714g1.dds"
        );
        assert!(matches!(
            respell("kit1_x.dds", Spelling::Name, &team()),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn player_and_common_outputs_respell_their_rest() {
        let copies = respell_outputs(
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
            &team(),
        );
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
    fn respelled_names_collide_as_before() {
        let (copies, findings) = drop_collisions(respell_outputs(
            vec![
                ("a".into(), "Players/07 - A/u0XXXp1.dds".into()),
                ("b".into(), "Players/07 - A/u0714p1.dds".into()),
            ],
            &team(),
        ));
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

    #[test]
    fn a_referee_export_respells_the_placeholder_999() {
        assert_eq!(
            respell("u0XXXg1.dds", Spelling::Name, &Identity::Referees),
            "u0999g1.dds"
        );
        assert_eq!(
            respell_outputs(
                vec![("a".into(), "Common/u0XXXg1.dds".into())],
                &Identity::Referees
            ),
            vec![("a".to_owned(), "Common/u0999g1.dds".to_owned())]
        );
    }

    #[test]
    fn model_folders_leave_the_goalkeeper_token_as_written() {
        assert!(matches!(
            respell("u0XXXg1.dds", Spelling::Name, &Identity::ModelFolders),
            Cow::Borrowed(_)
        ));
        assert_eq!(
            respell("u0XXXp1_u0XXXg1", Spelling::Path, &Identity::ModelFolders),
            "kit1_u0XXXg1"
        );
    }
}
