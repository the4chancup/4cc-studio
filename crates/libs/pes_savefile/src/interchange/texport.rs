//! `Texport`: PES's own in-game team export/import format. Two eras:
//!
//! - 18-21 `.ted`: a 0x50 plaintext header, a 32-byte XOR key at 0x30, then
//!   the save's own team, roster, tactics and player records concatenated
//!   (sizes in `schema/texport.rs`, measured on real 18/19/21 files; PES 20's
//!   key index is the reference editor's untested guess).
//! - 15-17 `TEXPORT00000000`: a whole savefile container; the tactics and
//!   player records' payload offsets are `schema::texport::texport_old`'s
//!   (the team and roster records' positions are unmeasured, so the team
//!   carries only the id the tactics record holds and an empty roster).
//!
//! The module owns crypto and carry-through only: every record is decoded
//! and re-encoded by the schema codec, and unmodeled bytes ride along in the
//! retained plaintext/container.

use pes_version::PesVersion;

use crate::codec::{
    CodecError, read_player, read_player_into, read_roster_into, read_tactics_into, read_team,
    write_player, write_roster, write_tactics, write_team,
};
use crate::container::{ContainerError, SaveContainer};
use crate::model::player::PlayerEntry;
use crate::model::team::TeamEntry;
use crate::schema::texport::{TexportLayout, texport_layout, texport_old};
use crate::schema::{VersionSchema, schema_for};

/// The 18-21 key sits at 0x30; the body it encrypts starts at 0x50.
const KEY: usize = 0x30;
const HEADER: usize = 0x50;
/// The coach block's byte length and the unmodeled block's after tactics.
const COACH: usize = 88;
const BLOCK: usize = 660;
const TAIL: usize = 12;

/// An open texport: the retained ciphertext's plaintext (18-21) or container
/// (15-17) plus the decoded team and players.
#[derive(Debug)]
pub struct Texport {
    version: PesVersion,
    file: File,
    team: TeamEntry,
    /// Roster order, empty slots skipped.
    players: Vec<PlayerEntry>,
}

/// What a texport keeps for the write back.
#[derive(Debug)]
enum File {
    /// 18-21: the whole decrypted file (header, key and body).
    Ted(Vec<u8>),
    /// 15-17: the savefile container.
    Old(SaveContainer),
}

/// Why a buffer is not a texport, or the texport cannot be written.
#[derive(Debug, thiserror::Error)]
pub enum TexportError {
    /// The size or the record ids name no version.
    #[error("not a recognized texport")]
    UnknownVersion,
    /// `Some(version)` was passed a file of another size.
    #[error("a PES {version:?} texport is {expected} bytes, the input is {actual}")]
    WrongSize {
        /// The version asked for.
        version: PesVersion,
        /// That version's file size.
        expected: usize,
        /// The input's byte count.
        actual: usize,
    },
    /// The 15-17 container crypto failed.
    #[error(transparent)]
    Container(#[from] ContainerError),
    /// A record codec failed.
    #[error(transparent)]
    Codec(#[from] CodecError),
    /// A roster slot and its player record disagree.
    #[error("roster slot {slot} holds player {roster}, the record is {player}")]
    RosterMismatch {
        /// The roster slot.
        slot: usize,
        /// The roster's player id.
        roster: u32,
        /// The record's player id.
        player: u32,
    },
    /// `new` cannot synthesize a 15-17 file: there is no measured template.
    #[error("no texport template exists for {0:?}; write by editing a read file")]
    NoTemplate(PesVersion),
}

/// The little-endian u32 id at a record's start.
fn record_id(record: &[u8]) -> u32 {
    u32::from_le_bytes(record[..4].try_into().expect("an id field"))
}

/// The derived record offsets of an 18-21 file.
struct Offsets {
    team: usize,
    coach: usize,
    roster: usize,
    tactics: usize,
    players: usize,
    width: usize,
}

impl Offsets {
    /// Team at 0x50, then coach, roster, tactics, the 660-byte block, `width`
    /// player records and the tail; `width` must divide the remaining space
    /// exactly (the layouts are measured so it does).
    fn of(layout: &TexportLayout, schema: &VersionSchema) -> Offsets {
        let team = HEADER;
        let coach = team + schema.team.size;
        let roster = coach + COACH;
        let tactics = roster + schema.roster.size;
        let players = tactics + schema.tactic.size + BLOCK;
        let width = (layout.size - TAIL - players) / schema.player.size;
        debug_assert_eq!(
            players + width * schema.player.size + TAIL,
            layout.size,
            "the player records and tail fill the file exactly"
        );
        Offsets {
            team,
            coach,
            roster,
            tactics,
            players,
            width,
        }
    }
}

/// The 18-21 body XOR'd with the 32-byte key at 0x30, starting at the
/// layout's key index and wrapping at 32. The same function encrypts: the
/// key bytes sit below 0x50 and pass through.
fn crypt(bytes: &[u8], layout: &TexportLayout) -> Vec<u8> {
    let mut out = bytes.to_vec();
    let key = &bytes[KEY..HEADER];
    let mut k = layout.key_index;
    for b in out.iter_mut().skip(HEADER) {
        *b ^= key[k];
        k = (k + 1) % 0x20;
    }
    out
}

/// Whether the decrypted candidate's team, roster and tactics records carry
/// one id — the check that picks 20 or 21 out of a shared file size.
fn ids_agree(plaintext: &[u8], layout: &TexportLayout, version: PesVersion) -> bool {
    let schema = schema_for(version);
    let off = Offsets::of(layout, schema);
    let id = |at: usize| record_id(&plaintext[at..at + 4]);
    id(off.team) == id(off.roster) && id(off.roster) == id(off.tactics)
}

/// The 18-21 record writer shared by `new` and `to_bytes`: team, the coach
/// block's id, roster, tactics and each filled roster slot's player record
/// into `plain`; every other byte is kept as it was. `players` must be the
/// filled slots' records in roster order (`RosterMismatch` on an id or
/// count disagreement).
fn write_ted<'a>(
    plain: &mut [u8],
    off: &Offsets,
    schema: &VersionSchema,
    team: &TeamEntry,
    players: impl Iterator<Item = &'a PlayerEntry>,
) -> Result<(), TexportError> {
    write_team(
        team,
        &mut plain[off.team..off.team + schema.team.size],
        schema.team,
    )?;
    plain[off.coach..off.coach + 4].copy_from_slice(&team.id.to_le_bytes());
    write_roster(
        team,
        &mut plain[off.roster..off.roster + schema.roster.size],
        schema.roster,
    )?;
    write_tactics(
        team,
        &mut plain[off.tactics..off.tactics + schema.tactic.size],
        schema.tactic,
    )?;
    let mut players = players;
    for (slot, roster) in team.roster.iter().enumerate().take(off.width) {
        if roster.player_id == 0 {
            continue;
        }
        let player = players.next().ok_or(TexportError::RosterMismatch {
            slot,
            roster: roster.player_id,
            player: 0,
        })?;
        if player.id != roster.player_id {
            return Err(TexportError::RosterMismatch {
                slot,
                roster: roster.player_id,
                player: player.id,
            });
        }
        let at = off.players + slot * schema.player.size;
        write_player(
            player,
            &mut plain[at..at + schema.player.size],
            schema.player,
        )?;
    }
    if let Some(extra) = players.next() {
        return Err(TexportError::RosterMismatch {
            slot: off.width,
            roster: 0,
            player: extra.id,
        });
    }
    Ok(())
}

impl Texport {
    /// `version = None` detects: the container's version for 15-17; by size
    /// for 18/19; 20 and 21 share a size, so both key indices are tried and
    /// the one whose team, roster and tactics record ids agree wins
    /// (`UnknownVersion` when neither or both do). An explicit `version` for
    /// 15-17 defers to the container's own.
    pub fn from_bytes(bytes: &[u8], version: Option<PesVersion>) -> Result<Texport, TexportError> {
        match version {
            Some(version) => match texport_layout(version) {
                Some(layout) => {
                    if bytes.len() != layout.size {
                        return Err(TexportError::WrongSize {
                            version,
                            expected: layout.size,
                            actual: bytes.len(),
                        });
                    }
                    Self::open_ted(bytes, version, layout)
                }
                None => Self::open_old(SaveContainer::decrypt(bytes)?),
            },
            None => {
                let versions: Vec<PesVersion> = PesVersion::ALL
                    .iter()
                    .copied()
                    .filter(|v| {
                        texport_layout(*v).is_some_and(|layout| {
                            bytes.len() == layout.size
                                && ids_agree(&crypt(bytes, layout), layout, *v)
                        })
                    })
                    .collect();
                match versions[..] {
                    [version] => Self::open_ted(
                        bytes,
                        version,
                        texport_layout(version).expect("filtered on Some"),
                    ),
                    _ => match SaveContainer::decrypt(bytes) {
                        Ok(container) => Self::open_old(container),
                        Err(ContainerError::Unrecognized) => Err(TexportError::UnknownVersion),
                        Err(error) => Err(TexportError::Container(error)),
                    },
                }
            }
        }
    }

    /// An 18-21 file: decrypt, then read the records at the derived offsets.
    fn open_ted(
        bytes: &[u8],
        version: PesVersion,
        layout: &TexportLayout,
    ) -> Result<Texport, TexportError> {
        let schema = schema_for(version);
        let off = Offsets::of(layout, schema);
        let plain = crypt(bytes, layout);
        let mut team = read_team(&plain[off.team..off.team + schema.team.size], schema.team)?;
        read_roster_into(
            &mut team,
            &plain[off.roster..off.roster + schema.roster.size],
            schema.roster,
        )?;
        read_tactics_into(
            &mut team,
            &plain[off.tactics..off.tactics + schema.tactic.size],
            schema.tactic,
        )?;
        let mut players = Vec::new();
        for (slot, roster) in team.roster.iter().enumerate().take(off.width) {
            if roster.player_id == 0 {
                continue;
            }
            let at = off.players + slot * schema.player.size;
            let record = &plain[at..at + schema.player.size];
            let id = record_id(record);
            if id != roster.player_id {
                return Err(TexportError::RosterMismatch {
                    slot,
                    roster: roster.player_id,
                    player: id,
                });
            }
            players.push(read_player(record, schema.player)?);
        }
        Ok(Texport {
            version,
            file: File::Ted(plain),
            team,
            players,
        })
    }

    /// A 15-17 file: the container's payload holds the tactics record and the
    /// consecutive player records (15/16: a player record then its appearance
    /// record per player); the team and roster records are unmeasured, so the
    /// team carries the tactics record's id and an empty roster.
    fn open_old(container: SaveContainer) -> Result<Texport, TexportError> {
        let version = container.version();
        let schema = schema_for(version);
        let old = texport_old(version).ok_or(TexportError::UnknownVersion)?;
        let payload = &container.payload;
        let tactics = payload
            .get(old.tactics_at..old.tactics_at + schema.tactic.size)
            .ok_or(CodecError::RecordSize {
                expected: schema.tactic.size,
                got: payload.len().saturating_sub(old.tactics_at),
            })?;
        let mut team = TeamEntry {
            id: record_id(tactics),
            ..TeamEntry::default()
        };
        read_tactics_into(&mut team, tactics, schema.tactic)?;
        let stride = schema
            .appearance
            .as_ref()
            .map_or(schema.player.size, |(_, a)| schema.player.size + a.size);
        let mut players = Vec::new();
        for i in 0usize.. {
            let at = old.players_at + i * stride;
            let Some(record) = payload.get(at..at + schema.player.size) else {
                break;
            };
            let id = record_id(record);
            let Ok(i) = u32::try_from(i) else { break };
            if id == 0 || id != team.id * 100 + 1 + i {
                break;
            }
            let mut player = read_player(record, schema.player)?;
            if let Some((_, appearance)) = &schema.appearance {
                let start = at + schema.player.size;
                let record =
                    payload
                        .get(start..start + appearance.size)
                        .ok_or(CodecError::RecordSize {
                            expected: appearance.size,
                            got: payload.len().saturating_sub(start),
                        })?;
                read_player_into(&mut player, record, appearance)?;
            }
            players.push(player);
        }
        Ok(Texport {
            version,
            file: File::Old(container),
            team,
            players,
        })
    }

    /// A fresh 18-21 file from the layout's templates: header, coach (id
    /// patched), zero block, tail, a key from `salt[..32]`; `players` in
    /// roster order, ids checked against the roster (`RosterMismatch`).
    /// 15-17 is `NoTemplate`: write by reading a real file and replacing its
    /// team and players (`team_mut`/`players_mut`).
    pub fn new(
        version: PesVersion,
        team: &TeamEntry,
        players: &[&PlayerEntry],
        salt: &[u8; 320],
    ) -> Result<Texport, TexportError> {
        let schema = schema_for(version);
        let layout = texport_layout(version).ok_or(TexportError::NoTemplate(version))?;
        let off = Offsets::of(layout, schema);
        // The skeleton: header, key, the coach template (its id is patched by
        // write_ted), a zero 660-byte block and the tail; the record space is
        // zero-initialised so every unmodeled bit stays 0.
        let mut plain = vec![0u8; layout.size];
        plain[..KEY].copy_from_slice(&layout.header);
        plain[KEY..HEADER].copy_from_slice(&salt[..32]);
        plain[off.coach..off.coach + COACH].copy_from_slice(&layout.coach);
        plain[layout.size - TAIL..].copy_from_slice(&layout.tail);
        write_ted(&mut plain, &off, schema, team, players.iter().copied())?;
        Ok(Texport {
            version,
            file: File::Ted(plain),
            team: team.clone(),
            players: players.iter().map(|p| (*p).clone()).collect(),
        })
    }

    /// The game version this texport belongs to.
    pub fn version(&self) -> PesVersion {
        self.version
    }

    /// The team (team, roster and tactics joined on 18-21; the tactics
    /// record's id and contents alone on 15-17).
    pub fn team(&self) -> &TeamEntry {
        &self.team
    }

    /// Every player, in roster order, empty slots skipped.
    pub fn players(&self) -> &[PlayerEntry] {
        &self.players
    }

    /// The team for editing.
    pub fn team_mut(&mut self) -> &mut TeamEntry {
        &mut self.team
    }

    /// Every player for editing, in roster order.
    pub fn players_mut(&mut self) -> &mut [PlayerEntry] {
        &mut self.players
    }

    /// The records re-encoded into the plaintext (unmodeled bytes as read),
    /// then encrypted: XOR with the file's own key on 18-21 — `salt` is
    /// unused on that path — the container with `salt` on 15-17.
    pub fn to_bytes(&self, salt: &[u8; 320]) -> Result<Vec<u8>, TexportError> {
        match &self.file {
            File::Ted(plaintext) => {
                let layout = texport_layout(self.version).expect("a ted holds a layout");
                let schema = schema_for(self.version);
                let off = Offsets::of(layout, schema);
                let mut plain = plaintext.clone();
                write_ted(&mut plain, &off, schema, &self.team, self.players.iter())?;
                Ok(crypt(&plain, layout))
            }
            File::Old(container) => {
                let schema = schema_for(self.version);
                let old = texport_old(self.version).expect("an old holds offsets");
                let mut container = container.clone();
                write_tactics(
                    &self.team,
                    &mut container.payload[old.tactics_at..old.tactics_at + schema.tactic.size],
                    schema.tactic,
                )?;
                let stride = schema
                    .appearance
                    .as_ref()
                    .map_or(schema.player.size, |(_, a)| schema.player.size + a.size);
                for (i, player) in self.players.iter().enumerate() {
                    let at = old.players_at + i * stride;
                    write_player(
                        player,
                        &mut container.payload[at..at + schema.player.size],
                        schema.player,
                    )?;
                    if let Some((_, appearance)) = &schema.appearance {
                        let at = at + schema.player.size;
                        write_player(
                            player,
                            &mut container.payload[at..at + appearance.size],
                            appearance,
                        )?;
                    }
                }
                Ok(container.to_bytes(salt)?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::{MasterKey, Scheme};
    use crate::test_support::{open, test_salt};

    /// The fixture .ted files, one per version.
    const TEDS: [(PesVersion, &[u8]); 3] = [
        (
            PesVersion::Pes18,
            include_bytes!("../../tests/fixtures/pes18_texport.ted"),
        ),
        (
            PesVersion::Pes19,
            include_bytes!("../../tests/fixtures/pes19_texport.ted"),
        ),
        (
            PesVersion::Pes21,
            include_bytes!("../../tests/fixtures/pes21_texport.ted"),
        ),
    ];

    #[test]
    fn every_layouts_width_divides_exactly() {
        for version in PesVersion::ALL {
            let Some(layout) = texport_layout(version) else {
                continue;
            };
            let off = Offsets::of(layout, schema_for(version));
            assert_eq!(
                off.players + off.width * schema_for(version).player.size + TAIL,
                layout.size,
                "{version:?}"
            );
        }
    }

    #[test]
    fn new_synthesizes_a_file_the_records_round_trip_through() {
        for (version, bytes) in TEDS {
            let fixture = Texport::from_bytes(bytes, None).expect("opens");
            let players: Vec<&PlayerEntry> = fixture.players().iter().collect();
            // The fixture's own key, so the plaintexts compare byte for byte.
            let mut salt = test_salt();
            salt[..32].copy_from_slice(&bytes[KEY..HEADER]);
            let made = Texport::new(version, fixture.team(), &players, &salt).expect("synthesizes");
            let out = made.to_bytes(&test_salt()).expect("encodes");
            let layout = texport_layout(version).expect("a layout");
            assert_eq!(out.len(), layout.size, "{version:?}");
            let back = Texport::from_bytes(&out, Some(version)).expect("reopens");
            assert_eq!(back.team(), fixture.team(), "{version:?} team");
            assert_eq!(back.players(), fixture.players(), "{version:?} players");

            // The non-record regions equal the real file's: the header and
            // key, the coach block (id patched), the zero 660-byte block and
            // the tail.
            let made_plain = crypt(&out, layout);
            let fixture_plain = crypt(bytes, layout);
            let off = Offsets::of(layout, schema_for(version));
            assert_eq!(
                &made_plain[..HEADER],
                &fixture_plain[..HEADER],
                "{version:?} header and key"
            );
            assert_eq!(
                &made_plain[off.coach..off.coach + COACH],
                &fixture_plain[off.coach..off.coach + COACH],
                "{version:?} coach"
            );
            let block = off.tactics + schema_for(version).tactic.size;
            assert!(
                fixture_plain[block..block + BLOCK].iter().all(|&b| b == 0),
                "{version:?} fixture block"
            );
            assert!(
                made_plain[block..block + BLOCK].iter().all(|&b| b == 0),
                "{version:?} made block"
            );
            assert_eq!(
                &made_plain[layout.size - TAIL..],
                &fixture_plain[layout.size - TAIL..],
                "{version:?} tail"
            );
        }
    }

    #[test]
    fn new_refuses_players_out_of_roster_order() {
        let (_, bytes) = TEDS[1];
        let fixture = Texport::from_bytes(bytes, None).expect("opens");
        let mut players: Vec<&PlayerEntry> = fixture.players().iter().collect();
        players.swap(0, 1);
        let err = Texport::new(PesVersion::Pes19, fixture.team(), &players, &test_salt())
            .expect_err("order is the roster's");
        assert!(
            matches!(err, TexportError::RosterMismatch { slot: 0, .. }),
            "{err:?}"
        );
    }

    #[test]
    fn new_has_no_template_for_15_to_17() {
        let (_, bytes) = TEDS[1];
        let fixture = Texport::from_bytes(bytes, None).expect("opens");
        let players: Vec<&PlayerEntry> = fixture.players().iter().collect();
        assert!(matches!(
            Texport::new(PesVersion::Pes17, fixture.team(), &players, &test_salt()),
            Err(TexportError::NoTemplate(PesVersion::Pes17))
        ));
    }

    /// A 15-17 *save* container is shorter than the texport's record
    /// offsets: reading it must error, not slice past the payload.
    #[test]
    fn a_save_shorter_than_the_old_offsets_is_an_error_not_a_panic() {
        let container = crate::test_support::fixture_container(PesVersion::Pes17);
        let bytes = container.to_bytes(&test_salt()).expect("encodes");
        let err = Texport::from_bytes(&bytes, None).expect_err("not a texport");
        assert!(matches!(err, TexportError::Codec(_)), "{err:?}");

        // A valid container whose payload ends before the tactics record:
        // the same refusal, never a slice panic.
        let mut container = crate::test_support::fixture_container(PesVersion::Pes17);
        container.payload.truncate(0x10330);
        let bytes = container.to_bytes(&test_salt()).expect("encodes");
        let err = Texport::from_bytes(&bytes, None).expect_err("not a texport");
        assert!(
            matches!(err, TexportError::Codec(CodecError::RecordSize { .. })),
            "{err:?}"
        );
    }

    #[test]
    fn edits_through_team_mut_and_players_mut_survive_the_round_trip() {
        let (_, bytes) = TEDS[1];
        let original = Texport::from_bytes(bytes, None).expect("opens");
        let mut t = Texport::from_bytes(bytes, None).expect("opens");
        t.team_mut().name = "EDITED".to_string();
        t.players_mut()[0].stats.attacking_prowess = 99;
        let out = t.to_bytes(&test_salt()).expect("encodes");
        let back = Texport::from_bytes(&out, None).expect("reopens");
        assert_eq!(back.team().name, "EDITED");
        assert_eq!(back.players()[0].stats.attacking_prowess, 99);
        let mut expected_team = original.team().clone();
        expected_team.name = "EDITED".to_string();
        assert_eq!(back.team(), &expected_team);
        let mut expected_players = original.players().to_vec();
        expected_players[0].stats.attacking_prowess = 99;
        assert_eq!(back.players(), &expected_players[..]);
    }

    #[test]
    fn a_shared_size_with_disagreeing_ids_is_unknown() {
        let (_, bytes) = TEDS[2];
        let layout = texport_layout(PesVersion::Pes21).expect("a layout");
        let mut plain = crypt(bytes, layout);
        // Corrupt the roster record's id so no key index makes the three
        // records agree, then re-encrypt under the 21 index.
        let off = Offsets::of(layout, schema_for(PesVersion::Pes21));
        plain[off.roster] ^= 0xff;
        let corrupt = crypt(&plain, layout);
        assert!(matches!(
            Texport::from_bytes(&corrupt, None),
            Err(TexportError::UnknownVersion)
        ));

        // Corrupting only the team record's id still refuses: roster and
        // tactics agree with each other, so agreement must be of all three.
        let mut plain = crypt(bytes, layout);
        plain[off.team] ^= 0xff;
        let corrupt = crypt(&plain, layout);
        assert!(matches!(
            Texport::from_bytes(&corrupt, None),
            Err(TexportError::UnknownVersion)
        ));
    }

    /// A PES 16 texport built the way the reference reads one (tactics
    /// record, then player record + appearance record per rostered player)
    /// wrapped in the savefile container. Self-consistency against our own
    /// construction: no real PES 15/16 texport exists, so this pins the
    /// player+appearance stride and the consecutive-id scan, unverified
    /// against a real file.
    #[test]
    fn a_synthesized_pes16_texport_round_trips_through_the_old_path() {
        let schema = schema_for(PesVersion::Pes16);
        let old = texport_old(PesVersion::Pes16).expect("offsets");
        let (_, appearance) = schema.appearance.as_ref().expect("a split record");
        let stride = schema.player.size + appearance.size;
        let (file, _) = open(PesVersion::Pes16);
        let team = file.team(701).expect("team 701");
        let rostered: Vec<&PlayerEntry> = team
            .roster
            .iter()
            .filter(|slot| slot.player_id != 0)
            .map(|slot| file.player(slot.player_id).expect("rostered"))
            .collect();
        assert!(rostered.len() >= 3, "the team has players");

        let mut payload = vec![0u8; old.players_at + 40 * stride + 64];
        write_tactics(
            team,
            &mut payload[old.tactics_at..old.tactics_at + schema.tactic.size],
            schema.tactic,
        )
        .expect("tactics write");
        for (i, player) in rostered.iter().enumerate() {
            let at = old.players_at + i * stride;
            write_player(
                player,
                &mut payload[at..at + schema.player.size],
                schema.player,
            )
            .expect("player write");
            write_player(
                player,
                &mut payload[at + schema.player.size..at + stride],
                appearance,
            )
            .expect("appearance write");
        }
        // A trailing record whose id is not the next consecutive id: the
        // reader stops at the roster count.
        let at = old.players_at + rostered.len() * stride;
        write_player(
            rostered[0],
            &mut payload[at..at + schema.player.size],
            schema.player,
        )
        .expect("stray write");

        let mut description = b"Team Export Data 01".to_vec();
        description.resize(384, 0);
        let container = SaveContainer {
            scheme: Scheme::Keyed(MasterKey::Pes16),
            description,
            logo: Vec::new(),
            payload,
            identifier: vec![0; MasterKey::Pes16.header_size() - 80],
            serial: Vec::new(),
        };
        let bytes = container.to_bytes(&test_salt()).expect("encodes");
        let t = Texport::from_bytes(&bytes, None).expect("opens");
        assert_eq!(t.version(), PesVersion::Pes16);
        assert_eq!(t.team().id, 701);
        assert_eq!(t.players().len(), rostered.len());
        let expected: Vec<PlayerEntry> = rostered.iter().map(|p| (*p).clone()).collect();
        assert_eq!(t.players(), &expected[..]);
        assert_eq!(t.to_bytes(&test_salt()).expect("encodes"), bytes);
    }

    /// PES 20's layout is derived from its own record sizes (a 528-byte team
    /// record, not 21's 588), so a 21-sized file is `WrongSize` for it — the
    /// untested key index never gets to decode anything.
    #[test]
    fn pes20s_derived_size_refuses_the_21_fixture() {
        let (_, bytes) = TEDS[2];
        let err = Texport::from_bytes(bytes, Some(PesVersion::Pes20)).expect_err("refused");
        assert!(
            matches!(
                err,
                TexportError::WrongSize {
                    version: PesVersion::Pes20,
                    ..
                }
            ),
            "{err:?}"
        );
    }
}
