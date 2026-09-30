//! The roster slot types: a folder's number becomes one of these only after
//! the range check, so the rest of the crate never sees an out-of-range slot.

use teams_list::TeamId;

/// A normal team's roster slot, `01`–`23`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerSlot(u8);

impl PlayerSlot {
    /// The slot's range, `1..=23`; anything else is `None`.
    pub fn new(slot: u8) -> Option<PlayerSlot> {
        (1..=23).contains(&slot).then_some(PlayerSlot(slot))
    }

    /// The bare number, `1..=23`.
    pub fn get(self) -> u8 {
        self.0
    }

    /// The player ID this slot belongs to, `team * 100 + slot` (92023 at
    /// most: past `u16`, so `u32`).
    pub fn player_id(self, team: TeamId) -> u32 {
        u32::from(team.get()) * 100 + u32::from(self.0)
    }
}

/// A referee roster slot, `01`–`35`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RefSlot(u8);

impl RefSlot {
    /// The slot's range, `1..=35`; anything else is `None`.
    pub fn new(slot: u8) -> Option<RefSlot> {
        (1..=35).contains(&slot).then_some(RefSlot(slot))
    }

    /// The bare number, `1..=35`.
    pub fn get(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_ranges_hold() {
        assert_eq!(PlayerSlot::new(0), None);
        assert_eq!(PlayerSlot::new(24), None);
        assert_eq!(PlayerSlot::new(1).map(PlayerSlot::get), Some(1));
        assert_eq!(PlayerSlot::new(23).map(PlayerSlot::get), Some(23));
        assert_eq!(RefSlot::new(0), None);
        assert_eq!(RefSlot::new(36), None);
        assert_eq!(RefSlot::new(1).map(RefSlot::get), Some(1));
        assert_eq!(RefSlot::new(35).map(RefSlot::get), Some(35));
    }

    #[test]
    fn player_id_is_team_times_hundred_plus_slot() {
        let team = TeamId::new(792).unwrap();
        assert_eq!(PlayerSlot::new(5).unwrap().player_id(team), 79205);
        let team = TeamId::new(920).unwrap();
        assert_eq!(PlayerSlot::new(23).unwrap().player_id(team), 92023);
    }
}
