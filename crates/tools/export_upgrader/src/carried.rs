//! Step 7's carried-over folders, `Portraits/`, `Collars/` and `Logo/`
//! (`plans/export_upgrader.md` step 7, `DECISIONS.md` 2026-10-18 "the logo
//! rule remeasured with the compiler's resampler, and its member grammar"):
//! portraits renamed to the only name the compiler accepts, with a bytes
//! check against the face folder's own portrait; collars as they are; the
//! logo members into the root `logo.png` and, only when it is a different
//! drawing, `logo_small.png`. What cannot be placed is left for the run's
//! fallback to report as unmigrated content.

use std::collections::{BTreeMap, BTreeSet};

use dds_convert::{Decoded, SourceFormat};
use studio_core::{Disposition, Message, Scope};

use crate::messages::{Code, INPUT, read_failed, tool_message};
use crate::players::path_scope;
use crate::source::SourceFailure;

/// The channel distance past which two pixels differ (of 255), as plan step
/// 7's rule names it.
const TOLERANCE: u8 = 32;
/// The share of differing pixels past which the old small logo is a
/// different drawing: measured on the 94 real `Logo/` pairs with this
/// resampler, every downscale scored under it and every deliberate variant
/// over (plan step 7).
const LOGO_SMALL_DISTINCT_SHARE: usize = 40;

/// What step 7 makes of an export's `Portraits/`, `Collars/` and `Logo/`:
/// the files copied, the input files they account for, and the findings on
/// the input.
#[derive(Debug, Default)]
pub(crate) struct CarriedFiles {
    /// Input path → output path, copied as they are.
    pub(crate) copies: Vec<(String, String)>,
    /// Input files accounted for; the run's fallback reports every other one.
    pub(crate) consumed: BTreeSet<String>,
    /// Findings on the input, in input-path order, a file's own findings in
    /// the order met.
    pub(crate) findings: Vec<Message>,
}

/// The role of an old `Logo/` member, by its name's ending.
#[derive(Clone, Copy)]
enum LogoRole {
    /// `_r_ll.png`, 512²: becomes `logo.png`.
    Large = 0,
    /// `_r_l.png`, 256²: the fallback's first choice, else dropped.
    Medium = 1,
    /// `_r.png`, 128²: `logo_small.png` when a distinct drawing, else
    /// dropped.
    Small = 2,
}

/// The rest of `path` below `top` (its name in any letter case), or `None`.
pub(crate) fn under<'a>(path: &'a str, top: &str) -> Option<&'a str> {
    let (head, rest) = path.split_once('/')?;
    head.eq_ignore_ascii_case(top).then_some(rest)
}

/// The two digits an old portrait's name ends in: `player_` + at least two
/// characters + `NN` + `.dds`, the fixed parts in any letter case
/// (`player_XXX05.dds` → `05`, `player_736917.dds` → `17`).
fn portrait_nn(file_name: &str) -> Option<&str> {
    if !file_name.get(..7)?.eq_ignore_ascii_case("player_") {
        return None;
    }
    if !file_name
        .get(file_name.len().checked_sub(4)?..)?
        .eq_ignore_ascii_case(".dds")
    {
        return None;
    }
    let between = file_name.get(7..file_name.len() - 4)?;
    let nn = between.get(between.len().checked_sub(2)?..)?;
    nn.bytes().all(|byte| byte.is_ascii_digit()).then_some(nn)
}

/// The role a `Logo/` file's name gives, the longest ending first.
fn logo_role(file_name: &str) -> Option<LogoRole> {
    let lowered = file_name.to_ascii_lowercase();
    if lowered.ends_with("_r_ll.png") {
        Some(LogoRole::Large)
    } else if lowered.ends_with("_r_l.png") {
        Some(LogoRole::Medium)
    } else if lowered.ends_with("_r.png") {
        Some(LogoRole::Small)
    } else {
        None
    }
}

/// `decoded`'s top mip resampled to 128 × 128 (a side already 128 is left
/// unfiltered by `resize`; no squaring: every real member is square).
fn at_128(decoded: &Decoded) -> Vec<u8> {
    dds_convert::resize(&decoded.mips[0], decoded.width, decoded.height, 128, 128)
        .expect("a decoded image's top mip resizes: nonzero size and a packed RGBA buffer")
}

/// Whether two RGBA pixels differ: any channel further apart than
/// `TOLERANCE`, two pixels both fully transparent being equal whatever
/// their other channels hold.
fn pixel_differs(a: &[u8; 4], b: &[u8; 4]) -> bool {
    if a[3] == 0 && b[3] == 0 {
        return false;
    }
    a.iter().zip(b).any(|(a, b)| a.abs_diff(*b) > TOLERANCE)
}

/// How many of the 128 × 128 compared pixels differ between `large` and
/// `small` (step 7's downscale-or-drawing test), each resampled there when
/// it is not already.
fn differing_pixels(large: &Decoded, small: &Decoded) -> usize {
    let (large, small) = (at_128(large), at_128(small));
    let (a, b) = (large.as_chunks::<4>().0, small.as_chunks::<4>().0);
    a.iter()
        .zip(b.iter())
        .filter(|(a, b)| pixel_differs(a, b))
        .count()
}

/// The small logo is a different drawing, and `logo_small.png`, when more
/// than `LOGO_SMALL_DISTINCT_SHARE` percent of the compared pixels differ.
fn is_distinct_drawing(differing: usize, total: usize) -> bool {
    differing * 100 > LOGO_SMALL_DISTINCT_SHARE * total
}

/// The differing pixels' share of `total`, rounded half up, formatted like
/// `30%`.
fn share_text(differing: usize, total: usize) -> String {
    format!("{}%", (differing * 200 + total) / (2 * total))
}

/// What step 7 makes of `files`' `Portraits/`, `Collars/` and `Logo/` for a
/// team export whose face folders' portraits are `face_portraits` (slot →
/// input path).
pub(crate) fn carried_files(
    files: &[String],
    face_portraits: &BTreeMap<u8, String>,
    mut read: impl FnMut(&str) -> Result<Vec<u8>, SourceFailure>,
) -> CarriedFiles {
    let mut carried = CarriedFiles::default();
    let mut failures: BTreeMap<String, Vec<Message>> = BTreeMap::new();

    // `Portraits/` (step 7): a file directly in it, `player_…NN.dds` in any
    // letter case, becomes `Portraits/player_NN.dds` — the first of two
    // giving one `NN` by input path. A slot whose face folder carries a
    // different portrait is `portrait_conflict`: the compiler's deep pass
    // would refuse the export.
    let mut slots = BTreeSet::new();
    for file in files {
        let Some(rest) = under(file, "portraits") else {
            continue;
        };
        let Some(nn) = (!rest.contains('/')).then(|| portrait_nn(rest)).flatten() else {
            continue;
        };
        if !slots.insert(nn.to_owned()) {
            continue;
        }
        carried
            .copies
            .push((file.clone(), format!("Portraits/player_{nn}.dds")));
        carried.consumed.insert(file.clone());
        let slot = nn
            .parse::<u8>()
            .expect("the two ASCII digits `portrait_nn` checked always parse");
        let Some(face_file) = face_portraits.get(&slot) else {
            continue;
        };
        let theirs = match read(file) {
            Ok(bytes) => bytes,
            // A read failure is no conflict; the copy loop's read reports
            // the file.
            Err(_) => continue,
        };
        let ours = match read(face_file) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };
        if theirs != ours {
            failures.entry(file.clone()).or_default().push(tool_message(
                Code::PortraitConflict,
                Scope::File {
                    export_id: INPUT,
                    path: path_scope(file),
                },
                Disposition::Keep,
                vec![("slot", nn.to_owned())],
            ));
        }
    }

    // `Collars/` (step 7): every file below it, at its relative path; the
    // shared validation judges what it holds. `Common/` is carried the
    // same way.
    for file in files {
        for (top, output) in [("collars", "Collars"), ("common", "Common")] {
            if let Some(rest) = under(file, top) {
                carried
                    .copies
                    .push((file.clone(), format!("{output}/{rest}")));
                carried.consumed.insert(file.clone());
            }
        }
    }

    // `Logo/` (step 7): a member is a file directly in it ending
    // `_r_ll.png`, `_r_l.png` or `_r.png` in any case, the first of one role
    // by input path — consumed whatever it decodes as. A member is usable
    // when it decodes as a PNG; one that does not is unmigrated content,
    // its error in the context.
    let mut members: [Option<(&String, Option<Decoded>)>; 3] = [None, None, None];
    for file in files {
        let Some(rest) = under(file, "logo") else {
            continue;
        };
        if rest.contains('/') {
            continue;
        }
        let Some(role) = logo_role(rest) else {
            continue;
        };
        let index = role as usize;
        if members[index].is_some() {
            continue;
        }
        match read(file) {
            Err(failure) => {
                failures.entry(file.clone()).or_default().push(read_failed(
                    INPUT,
                    failure,
                    Disposition::DropFile,
                ));
                carried.consumed.insert(file.clone());
                members[index] = Some((file, None));
            }
            Ok(bytes) => {
                carried.consumed.insert(file.clone());
                match dds_convert::decode(&bytes, SourceFormat::Png) {
                    Ok(decoded) => members[index] = Some((file, Some(decoded))),
                    Err(error) => {
                        failures.entry(file.clone()).or_default().push(tool_message(
                            Code::UnmigratedContent,
                            Scope::File {
                                export_id: INPUT,
                                path: path_scope(file),
                            },
                            Disposition::DropFile,
                            vec![("error", error.to_string())],
                        ));
                        members[index] = Some((file, None));
                    }
                }
            }
        }
    }
    let usable = |role: LogoRole| {
        members[role as usize]
            .as_ref()
            .and_then(|(file, decoded)| decoded.as_ref().map(|decoded| (*file, decoded)))
    };
    let (large, medium, small) = (
        usable(LogoRole::Large),
        usable(LogoRole::Medium),
        usable(LogoRole::Small),
    );
    if let Some((large_path, large)) = large {
        // `logo.png` is the large member's bytes (the medium is dropped
        // silently: the compiler regenerates it). A small member that is a
        // different drawing becomes `logo_small.png`; a downscale is
        // dropped, both with the measured share in the report.
        carried
            .copies
            .push((large_path.clone(), "logo.png".to_owned()));
        if let Some((small_path, small)) = small {
            let differing = differing_pixels(large, small);
            let total = 128 * 128;
            let carried_small = is_distinct_drawing(differing, total);
            if carried_small {
                carried
                    .copies
                    .push((small_path.clone(), "logo_small.png".to_owned()));
            }
            failures
                .entry(small_path.clone())
                .or_default()
                .push(tool_message(
                    if carried_small {
                        Code::LogoSmallCarried
                    } else {
                        Code::LogoSmallDropped
                    },
                    Scope::File {
                        export_id: INPUT,
                        path: path_scope(small_path),
                    },
                    Disposition::Keep,
                    vec![("share", share_text(differing, total))],
                ));
        }
    } else if let Some((path, _)) = medium.or(small) {
        // No usable large member: the largest usable one becomes `logo.png`
        // — and there is nothing left to compare a small one against.
        carried.copies.push((path.clone(), "logo.png".to_owned()));
        failures.entry(path.clone()).or_default().push(tool_message(
            Code::LogoFallback,
            Scope::File {
                export_id: INPUT,
                path: path_scope(path),
            },
            Disposition::Keep,
            vec![],
        ));
    }

    // The findings in input-path order (`files` is sorted): the passes met
    // them file by file.
    for file in files {
        if let Some(messages) = failures.remove(file.as_str()) {
            carried.findings.extend(messages);
        }
    }
    carried.findings.extend(failures.into_values().flatten());
    carried
}

#[cfg(test)]
mod tests {
    use studio_core::Severity;

    use super::*;

    /// A real `Logo/` member's PNG bytes, for any member a test needs.
    const PNG: &[u8] = include_bytes!("../tests/fixtures/logos/OMG VGL26/emblem_0XXX_r_ll.png");

    /// `carried_files` over `listing` with `face_portraits` and each listed
    /// file's bytes behind `read`; a listed path without bytes is a test
    /// bug, so it panics.
    fn plan(
        listing: &[&str],
        face_portraits: &[(u8, &str)],
        bytes: &[(&str, &[u8])],
    ) -> CarriedFiles {
        let bytes: BTreeMap<&str, &[u8]> = bytes.iter().copied().collect();
        let files: Vec<String> = listing.iter().map(|file| (*file).to_owned()).collect();
        let face_portraits: BTreeMap<u8, String> = face_portraits
            .iter()
            .map(|(slot, file)| (*slot, (*file).to_owned()))
            .collect();
        carried_files(&files, &face_portraits, |path| {
            bytes
                .get(path)
                .map(|bytes| Ok(bytes.to_vec()))
                .unwrap_or_else(|| panic!("a listed file has no bytes: {path}"))
        })
    }

    /// A 128 × 128 image of `color`.
    fn image(color: [u8; 4]) -> Decoded {
        image_with(color, &[])
    }

    /// A `base`-colored 128 × 128 image with `changes` pixels replaced at
    /// indexes.
    fn image_with(base: [u8; 4], changes: &[(usize, [u8; 4])]) -> Decoded {
        let mut pixels = vec![base; 128 * 128];
        for (index, color) in changes {
            pixels[*index] = *color;
        }
        Decoded {
            width: 128,
            height: 128,
            mips: vec![pixels.into_iter().flatten().collect()],
            blocks: None,
            authored_mips: false,
        }
    }

    #[test]
    fn a_channel_difference_past_the_tolerance_counts() {
        let a = image([10, 10, 10, 255]);
        // 32 in one channel is not a difference; 33 is.
        let b = image_with([10, 10, 10, 255], &[(0, [42, 10, 10, 255])]);
        assert_eq!(differing_pixels(&a, &b), 0);
        let b = image_with([10, 10, 10, 255], &[(0, [43, 10, 10, 255])]);
        assert_eq!(differing_pixels(&a, &b), 1);
    }

    #[test]
    fn both_fully_transparent_pixels_are_equal() {
        let a = image_with([0, 0, 0, 255], &[(0, [255, 0, 0, 0])]);
        let b = image_with([0, 0, 0, 255], &[(0, [0, 255, 0, 0])]);
        assert_eq!(differing_pixels(&a, &b), 0);
        // Transparent on one side only differs.
        let b = image_with([0, 0, 0, 255], &[(0, [255, 0, 0, 255])]);
        assert_eq!(differing_pixels(&a, &b), 1);
    }

    #[test]
    fn the_share_line_sits_at_forty_percent() {
        let a = image([0, 0, 0, 255]);
        // 6,553 differing (39.996 %): dropped; 6,554 (40.002 %): carried.
        for (differing, expected) in [(6553, false), (6554, true)] {
            let changes: Vec<(usize, [u8; 4])> = (0..differing)
                .map(|index| (index, [255, 255, 255, 255]))
                .collect();
            let b = image_with([0, 0, 0, 255], &changes);
            let differing = differing_pixels(&a, &b);
            assert_eq!(
                is_distinct_drawing(differing, 128 * 128),
                expected,
                "{differing}"
            );
        }
    }

    #[test]
    fn the_reported_share_rounds_half_up() {
        let total = 128 * 128;
        assert_eq!(share_text(4919, total), "30%");
        assert_eq!(share_text(8699, total), "53%");
        assert_eq!(share_text(8192, total), "50%");
    }

    #[test]
    fn the_portrait_name_grammar() {
        for (name, expected) in [
            ("player_XXX05.dds", Some("05")),
            ("player_71407.dds", Some("07")),
            ("player_09.dds", Some("09")),
            ("player_736917.dds", Some("17")),
            ("player_5.dds", None),
            ("portrait_05.dds", None),
            ("player_XXXab.dds", None),
            ("player_05.png", None),
        ] {
            assert_eq!(portrait_nn(name), expected, "{name}");
        }
    }

    #[test]
    fn equal_bytes_give_no_portrait_conflict_and_different_bytes_give_one() {
        let listing = &["Faces/XXX07 - A/portrait.dds", "Portraits/player_XXX07.dds"];
        let portraits = &[(7u8, "Faces/XXX07 - A/portrait.dds")];
        let equal = plan(
            listing,
            portraits,
            &[
                ("Faces/XXX07 - A/portrait.dds", b"same"),
                ("Portraits/player_XXX07.dds", b"same"),
            ],
        );
        assert_eq!(
            equal.copies,
            vec![(
                "Portraits/player_XXX07.dds".to_owned(),
                "Portraits/player_07.dds".to_owned()
            )]
        );
        assert!(
            equal
                .findings
                .iter()
                .all(|m| m.code.code.as_ref() != "portrait_conflict")
        );
        let different = plan(
            listing,
            portraits,
            &[
                ("Faces/XXX07 - A/portrait.dds", b"mine"),
                ("Portraits/player_XXX07.dds", b"theirs"),
            ],
        );
        let conflict = different
            .findings
            .iter()
            .find(|m| m.code.code.as_ref() == "portrait_conflict")
            .expect("portrait_conflict expected");
        assert_eq!(conflict.severity, Severity::Error);
        assert_eq!(conflict.disposition, Disposition::Keep);
        assert_eq!(conflict.context, vec![("slot".to_owned(), "07".to_owned())]);
        assert_eq!(
            conflict.scope,
            Scope::File {
                export_id: INPUT,
                path: path_scope("Portraits/player_XXX07.dds")
            }
        );
    }

    #[test]
    fn a_stray_file_of_the_logo_folder_is_not_consumed() {
        let plan = plan(
            &[
                "Logo/emblem_0XXX_r.png",
                "Logo/emblem_0XXX_r_l.png",
                "Logo/emblem_0XXX_r_ll.png",
                "Logo/foo.png",
            ],
            &[],
            &[
                ("Logo/emblem_0XXX_r.png", PNG),
                ("Logo/emblem_0XXX_r_l.png", PNG),
                ("Logo/emblem_0XXX_r_ll.png", PNG),
                ("Logo/foo.png", b"stray"),
            ],
        );
        for member in [
            "Logo/emblem_0XXX_r.png",
            "Logo/emblem_0XXX_r_l.png",
            "Logo/emblem_0XXX_r_ll.png",
        ] {
            assert!(plan.consumed.contains(member), "{member}");
        }
        assert!(!plan.consumed.contains("Logo/foo.png"));
    }

    #[test]
    fn two_portraits_giving_one_slot_keep_the_first() {
        let plan = plan(
            &["Portraits/player_71405.dds", "Portraits/player_XXX05.dds"],
            &[],
            &[
                ("Portraits/player_71405.dds", b"714"),
                ("Portraits/player_XXX05.dds", b"xxx"),
            ],
        );
        // `player_71405.dds` sorts before `player_XXX05.dds`: the first by
        // input path wins the slot.
        assert_eq!(
            plan.copies,
            vec![(
                "Portraits/player_71405.dds".to_owned(),
                "Portraits/player_05.dds".to_owned()
            )]
        );
        assert!(plan.consumed.contains("Portraits/player_71405.dds"));
        assert!(!plan.consumed.contains("Portraits/player_XXX05.dds"));
    }
}
