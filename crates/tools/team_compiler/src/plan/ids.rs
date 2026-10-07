//! The per-team block of boots and gloves IDs (`aesthetics_export/player_folders.md` "At
//! compile time, the pipeline", step 2): every team owns a fixed block of 40 IDs, the first 23
//! being its players' exclusive ones and the other 17 its shared folders', so planning computes
//! every ID and processing never allocates one, and the same export always compiles to the same
//! IDs.

use aesthetics_export::{
    PlayerFolder, PlayerSlot, SharedKind, SharedModelFolder, ValidatedAestheticsExport,
};
use pes_version::Engine;
use teams_list::TeamId;

use super::mapped_players;
use super::subset::link_combines;

/// The first ID of the first team's block: IDs 0 to 100 are the stock band.
const FIRST_BLOCK: u16 = 101;

/// A team's block: the 23 player-exclusive IDs, then 17 for its shared folders.
const BLOCK_SIZE: u16 = 40;

/// The block's first IDs, one per roster slot.
const EXCLUSIVE_COUNT: u16 = 23;

/// The shared folders of one kind a team's block has IDs for: the block's remainder after the
/// exclusive ones. An export needing more is `boots_id_pool_exhausted` or
/// `gloves_id_pool_exhausted`.
pub(crate) const SHARED_COUNT: u16 = BLOCK_SIZE - EXCLUSIVE_COUNT;

/// One team's planned boots and gloves IDs. Boots and gloves are disjoint game namespaces
/// (`boots/k0625/`, `glove/g0625/`), so one block serves both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlannedModelIds {
    /// The block's first ID, slot 01's.
    block_start: u16,
}

impl PlannedModelIds {
    /// Team `team`'s block, `101 + (team - 701) * 40`: team 701 gets 101 to 140, and the last
    /// team, 920, ends at 8900, clear of the referees' `99XX` band.
    pub(crate) fn for_team(team: TeamId) -> PlannedModelIds {
        PlannedModelIds {
            block_start: FIRST_BLOCK + (team.get() - TeamId::MIN) * BLOCK_SIZE,
        }
    }

    /// The exclusive boots and gloves ID of the player at roster slot `slot`: the block's
    /// `slot`-th ID, so slot 01 gets the block's first.
    pub(crate) fn exclusive(self, slot: PlayerSlot) -> u16 {
        self.block_start + u16::from(slot.get()) - 1
    }

    /// The shared ID at `index` in the kind's alphabetical folder order: the block's IDs after
    /// the exclusive ones, so index 0 is the 24th. `None` past the pool, which planning never
    /// reaches: the structure pass drops an export needing more.
    pub(crate) fn shared(self, index: usize) -> Option<u16> {
        let index = u16::try_from(index)
            .ok()
            .filter(|index| *index < SHARED_COUNT)?;
        Some(self.block_start + EXCLUSIVE_COUNT + index)
    }
}

/// The shared folders of `kind` in `export` that take a shared ID when compiled for `engine`,
/// in ID order: the folders at least one roster-mapped player folder links plainly, by name
/// (case folded, ties by the plain spelling). A shared face never takes one (on Fox it merges
/// into the player's face; pre-Fox it is copied per player), and neither does a folder only
/// unmapped folders link, since they are not compiled.
pub(crate) fn shared_folders_taking_ids(
    export: &ValidatedAestheticsExport,
    engine: Engine,
    kind: SharedKind,
) -> Vec<&SharedModelFolder> {
    let folders = match kind {
        SharedKind::Face => return Vec::new(),
        SharedKind::Boots => &export.boots,
        SharedKind::Gloves => &export.gloves,
    };
    let mapped = mapped_players(export);
    let mut taking: Vec<&SharedModelFolder> = folders
        .iter()
        .filter(|folder| {
            let name_key = vtree::fold_name(&folder.folder_name);
            mapped
                .iter()
                .any(|player| links_plainly(player, engine, kind, &name_key))
        })
        .collect();
    taking.sort_by_cached_key(|folder| {
        (
            vtree::fold_name(&folder.folder_name),
            folder.folder_name.clone(),
        )
    });
    taking
}

/// Whether `player` links the shared folder of `kind` whose folded name is `name_key` so that
/// the shared output is loaded as it is. On Fox a link that combines (`link_combines`) makes
/// the shared folder only a source of parts for the player's own package; pre-Fox has no
/// exclusive packages, so every link is plain.
fn links_plainly(player: &PlayerFolder, engine: Engine, kind: SharedKind, name_key: &str) -> bool {
    player.links.iter().any(|link| {
        link.kind == kind
            && vtree::fold_name(&link.name) == name_key
            && match engine {
                Engine::PreFox => true,
                Engine::Fox => !link_combines(player, link),
            }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{resolved, resolved_with_issues};

    fn slot(number: u8) -> PlayerSlot {
        PlayerSlot::new(number).unwrap()
    }

    fn team(id: u16) -> TeamId {
        TeamId::new(id).unwrap()
    }

    #[test]
    fn each_slot_gets_its_team_block_s_id_in_slot_order() {
        let co = PlannedModelIds::for_team(team(714));
        assert_eq!(co.exclusive(slot(5)), 625);
        assert_eq!(co.exclusive(slot(23)), 643);
        assert_eq!(
            PlannedModelIds::for_team(team(792)).exclusive(slot(5)),
            3745
        );
        assert_eq!(PlannedModelIds::for_team(team(701)).exclusive(slot(1)), 101);
        assert_eq!(
            PlannedModelIds::for_team(team(TeamId::MAX)).exclusive(slot(23)),
            8883,
            "the last team's last exclusive ID stays below the referee band"
        );
    }

    #[test]
    fn the_shared_ids_follow_the_exclusive_ones_and_stop_at_seventeen() {
        let co = PlannedModelIds::for_team(team(714));
        assert_eq!(co.shared(0), Some(644));
        assert_eq!(co.shared(16), Some(660));
        assert_eq!(co.shared(17), None);
        assert_eq!(SHARED_COUNT, 17);
        assert_eq!(
            PlannedModelIds::for_team(team(TeamId::MAX)).shared(16),
            Some(8900),
            "the last team's block ends at 8900"
        );
    }

    /// The folder names of `shared_folders_taking_ids` over the export `co Midcup Shared`.
    fn taking(
        files: &[&str],
        players_txt: Option<&[u8]>,
        engine: Engine,
        kind: SharedKind,
    ) -> Vec<String> {
        let files: Vec<(&str, u64)> = files.iter().map(|path| (*path, 1)).collect();
        let export = resolved("co Midcup Shared", &files, &[], players_txt).export;
        shared_folders_taking_ids(&export, engine, kind)
            .iter()
            .map(|folder| folder.folder_name.clone())
            .collect()
    }

    #[test]
    fn plainly_linked_folders_take_ids_in_case_folded_name_order() {
        let files = [
            "Players/03 - A/Zebra.boots",
            "Players/07 - B/apple.boots",
            "Players/11 - C/Mango.boots",
            "Players/11 - C/Grip.gloves",
            "Boots/Zebra/boots.fmdl",
            "Boots/apple/boots.fmdl",
            "Boots/Mango/boots.fmdl",
            "Gloves/Grip/glove_l.fmdl",
        ];
        assert_eq!(
            taking(&files, None, Engine::Fox, SharedKind::Boots),
            ["apple", "Mango", "Zebra"]
        );
        assert_eq!(
            taking(&files, None, Engine::Fox, SharedKind::Gloves),
            ["Grip"]
        );
        assert_eq!(
            taking(&files, None, Engine::Fox, SharedKind::Face),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_folder_linked_by_a_combining_player_takes_an_id_only_pre_fox() {
        // Slot 03 holds boots of its own, so on Fox its link combines; slot 07's gloves do
        // not touch its boots link.
        let files = [
            "Players/03 - A/Crocs.boots",
            "Players/03 - A/kit_boots.fmdl",
            "Players/07 - B/Mud.boots",
            "Players/07 - B/glove_l.fmdl",
            "Boots/Crocs/boots.fmdl",
            "Boots/Mud/boots.fmdl",
        ];
        assert_eq!(
            taking(&files, None, Engine::Fox, SharedKind::Boots),
            ["Mud"]
        );
        assert_eq!(
            taking(&files, None, Engine::PreFox, SharedKind::Boots),
            ["Crocs", "Mud"]
        );
    }

    #[test]
    fn a_folder_only_an_unmapped_folder_links_takes_no_id() {
        let (export, issues) = resolved_with_issues(
            "co Midcup Shared",
            &[
                ("Players/A/Crocs.boots", 1),
                ("Players/Unlisted/Mud.boots", 1),
                ("Boots/Crocs/boots.fmdl", 1),
                ("Boots/Mud/boots.fmdl", 1),
            ],
            &[],
            Some(b"03 A\n"),
        );
        assert_eq!(issues, ["player_unlisted", "shared_folder_orphaned"]);
        let names: Vec<&str> =
            shared_folders_taking_ids(&export.export, Engine::Fox, SharedKind::Boots)
                .iter()
                .map(|folder| folder.folder_name.as_str())
                .collect();
        assert_eq!(names, ["Crocs"]);
    }
}
