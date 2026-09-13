//! The test fixtures; see `tests/fixtures/README.md` for what each file is.

pub(crate) const CARD: &[u8] = include_bytes!("../../tests/fixtures/konami_card.model");
pub(crate) const CARDHEAD: &[u8] = include_bytes!("../../tests/fixtures/cardhead_face_high.model");
pub(crate) const CARDHEAD_DOUBLESIDED: &[u8] =
    include_bytes!("../../tests/fixtures/cardhead_doublesided_face_high.model");
pub(crate) const CAP: &[u8] = include_bytes!("../../tests/fixtures/konami_modD_cap.model");
pub(crate) const HAIR_D: &[u8] = include_bytes!("../../tests/fixtures/konami_hair_d_win32.model");
pub(crate) const FLAG: &[u8] = include_bytes!("../../tests/fixtures/konami_flag_close.wesys.model");
pub(crate) const GLASSES: &[u8] =
    include_bytes!("../../tests/fixtures/konami_glasses_02.wesys.model");
pub(crate) const HEAD_HI: &[u8] = include_bytes!("../../tests/fixtures/konami_headHi.wesys.model");
pub(crate) const TAPING: &[u8] = include_bytes!("../../tests/fixtures/konami_taping.wesys.model");
pub(crate) const HAIR_HIGH: &[u8] =
    include_bytes!("../../tests/fixtures/konami_hair_high_sp_ty004.wesys.model");
pub(crate) const COLLAR: &[u8] =
    include_bytes!("../../tests/fixtures/konami_collar_052.wesys.model");
pub(crate) const SHADOW: &[u8] =
    include_bytes!("../../tests/fixtures/konami_shadow_win32.wesys.model");

/// All twelve fixtures.
pub(crate) const ALL: &[&[u8]] = &[
    CARD,
    CARDHEAD,
    CARDHEAD_DOUBLESIDED,
    CAP,
    HAIR_D,
    FLAG,
    GLASSES,
    HEAD_HI,
    TAPING,
    HAIR_HIGH,
    COLLAR,
    SHADOW,
];

pub(crate) const CARDHEAD_MTL: &[u8] =
    include_bytes!("../../tests/fixtures/cardhead_materials.mtl");
pub(crate) const CARD_RED_MTL: &[u8] = include_bytes!("../../tests/fixtures/konami_card_red.mtl");
pub(crate) const CAP_MTL: &[u8] = include_bytes!("../../tests/fixtures/konami_modD_cap.mtl");
pub(crate) const SHADOW_MTL: &[u8] = include_bytes!("../../tests/fixtures/konami_shadow.mtl");
pub(crate) const ACCESSORY_MTL: &[u8] = include_bytes!("../../tests/fixtures/konami_accessory.mtl");
pub(crate) const HEAD_HI_MTL: &[u8] = include_bytes!("../../tests/fixtures/konami_headHi.mtl");
pub(crate) const HAIR_MTL: &[u8] = include_bytes!("../../tests/fixtures/konami_hair.mtl");

/// All seven material-set fixtures.
pub(crate) const ALL_MTL: &[&[u8]] = &[
    CARDHEAD_MTL,
    CARD_RED_MTL,
    CAP_MTL,
    SHADOW_MTL,
    ACCESSORY_MTL,
    HEAD_HI_MTL,
    HAIR_MTL,
];
