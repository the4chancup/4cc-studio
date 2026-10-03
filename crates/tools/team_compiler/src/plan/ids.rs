//! The per-team block of boots and gloves IDs (`aesthetics_export/player_folders.md` "At
//! compile time, the pipeline", step 2): every team owns a fixed block of 40 IDs, the first 23
//! being its players' exclusive ones, so planning computes every ID and processing never
//! allocates one, and the same export always compiles to the same IDs.

use aesthetics_export::PlayerSlot;
use teams_list::TeamId;

/// The first ID of the first team's block: IDs 0 to 100 are the stock band.
const FIRST_BLOCK: u16 = 101;

/// A team's block: the 23 player-exclusive IDs, then 17 for its shared folders.
const BLOCK_SIZE: u16 = 40;

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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
