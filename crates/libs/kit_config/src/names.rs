//! Texture-name derivation: the five 16-byte fields are derived at compile
//! time from the textures actually present, never authored.

/// Which of a kit's five textures exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TexturePresence {
    /// `kit.dds`.
    pub kit: bool,
    /// `kit_back.dds`.
    pub back: bool,
    /// `kit_chest.dds`.
    pub chest: bool,
    /// `kit_leg.dds`.
    pub leg: bool,
    /// `kit_name.dds`.
    pub name: bool,
}

/// One of the game's ten kit slots: nine player kits and the goalkeeper kit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KitSlot {
    /// Player kit 1 (`p1`, entry suffix `1st`).
    P1,
    /// Player kit 2 (`p2`, entry suffix `2nd`).
    P2,
    /// Player kit 3 (`p3`, entry suffix `3rd`).
    P3,
    /// Player kit 4 (`p4`, entry suffix `4th`).
    P4,
    /// Player kit 5 (`p5`, entry suffix `5th`).
    P5,
    /// Player kit 6 (`p6`, entry suffix `6th`).
    P6,
    /// Player kit 7 (`p7`, entry suffix `7th`).
    P7,
    /// Player kit 8 (`p8`, entry suffix `8th`).
    P8,
    /// Player kit 9 (`p9`, entry suffix `9th`).
    P9,
    /// The goalkeeper kit (`g1`, entry suffix `GK1st`).
    G1,
}

impl KitSlot {
    const ALL: [KitSlot; 10] = [
        KitSlot::P1,
        KitSlot::P2,
        KitSlot::P3,
        KitSlot::P4,
        KitSlot::P5,
        KitSlot::P6,
        KitSlot::P7,
        KitSlot::P8,
        KitSlot::P9,
        KitSlot::G1,
    ];

    /// The slot's texture-name spelling and its kit-config entry suffix.
    fn names(self) -> (&'static str, &'static str) {
        match self {
            KitSlot::P1 => ("p1", "1st"),
            KitSlot::P2 => ("p2", "2nd"),
            KitSlot::P3 => ("p3", "3rd"),
            KitSlot::P4 => ("p4", "4th"),
            KitSlot::P5 => ("p5", "5th"),
            KitSlot::P6 => ("p6", "6th"),
            KitSlot::P7 => ("p7", "7th"),
            KitSlot::P8 => ("p8", "8th"),
            KitSlot::P9 => ("p9", "9th"),
            KitSlot::G1 => ("g1", "GK1st"),
        }
    }

    /// Reads `p1`–`p9` or `g1`, ASCII-case-insensitively; anything else is `None`.
    pub fn parse(name: &str) -> Option<KitSlot> {
        KitSlot::ALL
            .iter()
            .find(|slot| slot.as_str().eq_ignore_ascii_case(name))
            .copied()
    }

    /// The lowercase slot name the texture names carry: `p1` … `p9`, `g1`.
    pub fn as_str(self) -> &'static str {
        self.names().0
    }

    /// The slot's kit-config entry name: `{team_id:03}_DEF_{1st…9th|GK1st}_realUni.bin`.
    pub fn config_name(self, team_id: u16) -> String {
        format!("{team_id:03}_DEF_{}_realUni.bin", self.names().1)
    }
}

/// The five 16-byte texture-name fields for a kit: `u0{team_id:03}{slot}` for
/// the kit itself, `..._back` / `..._chest` / `..._leg` / `..._name` for the
/// others, ASCII and NUL-padded, all zeros where the texture is absent.
/// `team_id` in 701..=920 is the caller's job.
pub fn texture_names(team_id: u16, slot: KitSlot, present: TexturePresence) -> [[u8; 16]; 5] {
    let base = format!("u0{team_id:03}{}", slot.as_str());
    let fields = [
        (present.kit, base.clone()),
        (present.back, format!("{base}_back")),
        (present.chest, format!("{base}_chest")),
        (present.leg, format!("{base}_leg")),
        (present.name, format!("{base}_name")),
    ];
    fields.map(|(present, name)| {
        let mut field = [0u8; 16];
        if present {
            let bytes = name.as_bytes();
            let len = bytes.len().min(16);
            field[..len].copy_from_slice(&bytes[..len]);
        }
        field
    })
}
