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

/// The five 16-byte texture-name fields for a kit: `u0{team_id:03}{slot}` for
/// the kit itself, `..._back` / `..._chest` / `..._leg` / `..._name` for the
/// others, ASCII and NUL-padded, all zeros where the texture is absent.
/// `team_id` in 701..=920 is the caller's job.
pub fn texture_names(team_id: u16, slot: &str, present: TexturePresence) -> [[u8; 16]; 5] {
    let base = format!("u0{team_id:03}{slot}");
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
