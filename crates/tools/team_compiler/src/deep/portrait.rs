//! The deep pass's portrait checks: each portrait held to the portrait's size rule, and a
//! slot's two portraits, its player folder's and its `Portraits/` file, compared
//! (`team_compiler/messages.md` "Textures", `portrait_conflict`).

use aesthetics_export::{
    ContentFinding, Disposition, FileDescriptor, IssueScope, PlayerSlot, ValidatedAestheticsExport,
    ValidatedRoster,
};

use super::texture::SizeRule;
use super::{checked_as, file_findings};
use crate::messages::Code;
use crate::reader::ContentSource;

/// The findings of the portrait `file`, named `name`, held to the portrait's size rule on any
/// target: each on the file's own scope, dropping that file alone.
pub(super) fn portrait_findings(
    content: &mut ContentSource,
    file: &FileDescriptor,
    name: &str,
) -> Vec<ContentFinding> {
    let Some(checked) = checked_as(file, SizeRule::Portrait) else {
        return Vec::new();
    };
    file_findings(
        content,
        file,
        checked,
        &IssueScope::File(file.path.clone()),
        Disposition::DropFile,
        name,
    )
}

/// The portrait of the player folder `slot` maps in `export`, when one does and it holds one:
/// the `Portraits/` file of that slot is its second source. A folder several slots map stands
/// for each of them. A referee roster's slots are not a team's player slots, so they pair
/// with no `Portraits/` file.
pub(super) fn folder_portrait(
    export: &ValidatedAestheticsExport,
    slot: PlayerSlot,
) -> Option<&FileDescriptor> {
    let ValidatedRoster::Team(slots) = &export.roster else {
        return None;
    };
    let index = slots.get(&slot)?;
    export.players.get(index.0)?.portrait.as_ref()
}

/// `portrait_conflict` when `folder_portrait` and `portraits_file`, one slot's two portraits,
/// differ in bytes: the export is skipped, since the compiler cannot tell which one the
/// manager means. Byte-identical files are one portrait and no finding. The two files' own
/// findings do not matter here: a portrait of the wrong size is still compared.
pub(super) fn portrait_conflict(
    content: &mut ContentSource,
    folder_portrait: &FileDescriptor,
    portraits_file: &FileDescriptor,
) -> Option<ContentFinding> {
    let folder_bytes = content.read(folder_portrait.source.as_str());
    let portraits_bytes = content.read(portraits_file.source.as_str());
    let (folder_bytes, portraits_bytes) = match (folder_bytes, portraits_bytes) {
        (Ok(folder_bytes), Ok(portraits_bytes)) => (folder_bytes, portraits_bytes),
        // Each file's own check read it first and reported the failure as
        // `source_read_failed` on that file, which drops it: nothing is left to compare.
        (Err(failure), _) | (_, Err(failure)) => {
            log::debug!(
                "{}: portraits not compared: {}",
                failure.path,
                failure.error
            );
            return None;
        }
    };
    (folder_bytes != portraits_bytes).then(|| ContentFinding {
        code: Code::PortraitConflict.as_str(),
        scope: IssueScope::Export,
        context: vec![
            ("folder_portrait", folder_portrait.path.as_str().to_owned()),
            ("portraits_file", portraits_file.path.as_str().to_owned()),
        ],
        disposition: Disposition::DropExport,
        pass_through_eligible: false,
    })
}

#[cfg(test)]
mod tests {
    use aesthetics_export::{ContentFinding, Disposition, IssueScope};
    use pes_version::PesVersion;

    use crate::deep::tests::{
        bc1_dds, findings_for, findings_of, path, texture, texture_finding_on,
    };
    use crate::testing::scratch;

    #[test]
    fn a_portraits_file_whose_side_is_not_a_power_of_two_is_dropped_on_any_target() {
        let odd = |version| {
            let temp = scratch(&format!("deep_portrait_odd_{version:?}"));
            findings_for(
                version,
                temp.path(),
                &[("Portraits/player_05.png", texture("odd.png"))],
                &[],
                &[],
            )
        };
        let expected = [texture_finding_on(
            "texture_not_pow2",
            &IssueScope::File(path("Portraits/player_05.png")),
            "player_05.png",
            Disposition::DropFile,
            true,
        )];
        assert_eq!(odd(PesVersion::Pes21), expected, "PES 21");
        assert_eq!(odd(PesVersion::Pes17), expected, "PES 17");
        // A single-level DDS of that size, which passes anywhere else on Fox.
        let temp = scratch("deep_portrait_odd_dds");
        let findings = findings_of(
            temp.path(),
            &[("Portraits/player_05.dds", bc1_dds(300, 300))],
            &[],
        );
        assert_eq!(
            findings,
            [texture_finding_on(
                "texture_not_pow2",
                &IssueScope::File(path("Portraits/player_05.dds")),
                "player_05.dds",
                Disposition::DropFile,
                true,
            )]
        );
    }

    #[test]
    fn a_player_s_portrait_too_small_is_dropped_alone_on_its_file() {
        let temp = scratch("deep_texture_portrait");
        let findings = findings_of(
            temp.path(),
            &[("Players/03 - A/portrait.png", texture("tiny.png"))],
            &[],
        );
        assert_eq!(
            findings,
            [texture_finding_on(
                "texture_too_small",
                &IssueScope::File(path("Players/03 - A/portrait.png")),
                "portrait.png",
                Disposition::DropFile,
                true,
            )]
        );
    }

    #[test]
    fn a_renamed_portrait_is_a_type_mismatch_and_its_size_is_not_read() {
        let temp = scratch("deep_portrait_renamed");
        // 300x300 PNG bytes, which would be `texture_not_pow2` were their header read.
        let findings = findings_of(
            temp.path(),
            &[("Portraits/player_05.dds", texture("odd.png"))],
            &[],
        );
        assert_eq!(
            findings,
            [texture_finding_on(
                "texture_type_mismatch",
                &IssueScope::File(path("Portraits/player_05.dds")),
                "player_05.dds",
                Disposition::DropFile,
                false,
            )]
        );
    }

    /// `portrait_conflict` between slot 05's folder portrait `folder_portrait` and the
    /// `Portraits/` file `portraits_file`.
    fn portrait_conflict(folder_portrait: &str, portraits_file: &str) -> ContentFinding {
        ContentFinding {
            code: "portrait_conflict",
            scope: IssueScope::Export,
            context: vec![
                ("folder_portrait", folder_portrait.to_owned()),
                ("portraits_file", portraits_file.to_owned()),
            ],
            disposition: Disposition::DropExport,
            pass_through_eligible: false,
        }
    }

    #[test]
    fn a_slot_s_two_portraits_conflict_only_when_their_bytes_differ() {
        let temp = scratch("deep_portrait_conflict");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/05 - A/portrait.dds", bc1_dds(64, 64)),
                ("Portraits/player_05.dds", bc1_dds(128, 128)),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [portrait_conflict(
                "Players/05 - A/portrait.dds",
                "Portraits/player_05.dds"
            )]
        );
        let temp = scratch("deep_portrait_identical");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/05 - A/portrait.dds", bc1_dds(64, 64)),
                ("Portraits/player_05.dds", bc1_dds(64, 64)),
            ],
            &[],
        );
        assert_eq!(findings, []);
        // Another slot's file is no pair.
        let temp = scratch("deep_portrait_other_slot");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/05 - A/portrait.dds", bc1_dds(64, 64)),
                ("Portraits/player_07.dds", bc1_dds(128, 128)),
            ],
            &[],
        );
        assert_eq!(findings, []);
        // A file with a size finding is still compared.
        let temp = scratch("deep_portrait_conflict_small");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/05 - A/portrait.png", texture("tiny.png")),
                ("Portraits/player_05.png", texture("portrait.png")),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [
                texture_finding_on(
                    "texture_too_small",
                    &IssueScope::File(path("Players/05 - A/portrait.png")),
                    "portrait.png",
                    Disposition::DropFile,
                    true,
                ),
                portrait_conflict("Players/05 - A/portrait.png", "Portraits/player_05.png"),
            ]
        );
    }
}
