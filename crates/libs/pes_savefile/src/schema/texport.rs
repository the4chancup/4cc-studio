//! The Texport layout tables. Two eras: the 18-21 `.ted` (a 0x50 header, a
//! 32-byte XOR key at 0x30, then the save's own records concatenated) and the
//! 15-17 `TEXPORT00000000` (a whole savefile container; the table names where
//! the tactics and player records sit in its payload).
//!
//! Provenance: the 18/19/21 constants are measured on real files (four 18,
//! eleven 19, five 21 texports; counts in `operations.md` "Interchange
//! formats"). No PES 15/16/20 texport exists: 15/16's offsets are the
//! reference editor's, unverified, and PES 20's key index `0x14` is the
//! reference editor's untested guess over PES 21's templates.

use pes_version::PesVersion;

/// An 18-21 `.ted` layout. The record offsets are derived, not stored: the
/// team record sits at 0x50, the coach block after it, then roster, tactics,
/// a 660-byte block, `width` player records and the 12-byte tail.
pub struct TexportLayout {
    /// First key byte used.
    pub key_index: usize,
    /// Whole file.
    pub size: usize,
    /// The fixture's header, the writer's template.
    pub header: [u8; 0x30],
    /// The fixture's coach block minus the id, the writer's template.
    pub coach: [u8; 88],
    /// The file's last 12 bytes.
    pub tail: [u8; 12],
}

/// Where a 15-17 texport's readable records sit in the container's payload
/// (the team and roster records' positions are unmeasured).
pub struct TexportOld {
    /// Byte offset of the tactics record.
    pub tactics_at: usize,
    /// Byte offset of the first player record.
    pub players_at: usize,
}

/// `s` as a `[u8; N]`, so the measured hex strings stay verbatim.
const fn hex<const N: usize>(s: &str) -> [u8; N] {
    let bytes = s.as_bytes();
    assert!(bytes.len() == N * 2, "a hex pair per byte");
    let mut out = [0u8; N];
    let mut i = 0;
    while i < N {
        let hi = digit(bytes[2 * i]);
        let lo = digit(bytes[2 * i + 1]);
        out[i] = hi << 4 | lo;
        i += 1;
    }
    out
}

const fn digit(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        _ => panic!("a lower-case hex digit"),
    }
}

/// The 88-byte coach block `team id (u32) | 4 constant bytes | flag |
/// "MANAGER" | zeros`, the id left zero (patched at write).
const fn coach(flag: u8) -> [u8; 88] {
    let mut out = [0u8; 88];
    out[4] = 0xe7;
    out[6] = 0xff;
    out[7] = 0xff;
    out[8] = flag;
    let mut i = 0;
    while i < 7 {
        out[9 + i] = b"MANAGER"[i];
        i += 1;
    }
    out
}

static PES18: TexportLayout = TexportLayout {
    key_index: 0x12,
    size: 0x1FC0,
    header: hex(
        "1700010050000000701f0000062900000000000000000000000000000000000038000000000000000000000000000000",
    ),
    coach: coach(0x61),
    tail: hex("000000000000000001000000"),
};

static PES19: TexportLayout = TexportLayout {
    key_index: 0x13,
    size: 0x25B0,
    header: hex(
        "170001005000000060250000682900000400000004000000000000000000000027000000000000000000000000000000",
    ),
    coach: coach(0x60),
    tail: hex("000000000400000001000000"),
};

/// PES 20: no file exists to measure. The key index is the reference
/// editor's untested guess and the templates are PES 21's, but PES 20's team
/// record is 60 bytes shorter than 21's, so the size is the derived one
/// (0x39A8), not 21's 0x39E4.
static PES20: TexportLayout = TexportLayout {
    key_index: 0x14,
    size: 0x39A8,
    header: hex(
        "180001005000000094390000742700000000000000000000000000000000000019000000000000000000000000000000",
    ),
    coach: coach(0x00),
    tail: hex("000000000000000001000000"),
};

static PES21: TexportLayout = TexportLayout {
    key_index: 0x15,
    size: 0x39E4,
    header: hex(
        "180001005000000094390000742700000000000000000000000000000000000019000000000000000000000000000000",
    ),
    coach: coach(0x00),
    tail: hex("000000000000000001000000"),
};

// PES 15/16's offsets are the reference editor's, unverified.
static PES15_OLD: TexportOld = TexportOld {
    tactics_at: 0x10298,
    players_at: 0x51065C,
};

static PES16_OLD: TexportOld = TexportOld {
    tactics_at: 0x10298,
    players_at: 0x510660,
};

static PES17_OLD: TexportOld = TexportOld {
    tactics_at: 0x10330,
    players_at: 0x510840,
};

/// The 18-21 `.ted` layout; `None` on 15-17.
pub fn texport_layout(version: PesVersion) -> Option<&'static TexportLayout> {
    match version {
        PesVersion::Pes18 => Some(&PES18),
        PesVersion::Pes19 => Some(&PES19),
        PesVersion::Pes20 => Some(&PES20),
        PesVersion::Pes21 => Some(&PES21),
        PesVersion::Pes15 | PesVersion::Pes16 | PesVersion::Pes17 => None,
    }
}

/// The 15-17 payload offsets; `None` on 18-21.
pub fn texport_old(version: PesVersion) -> Option<&'static TexportOld> {
    match version {
        PesVersion::Pes15 => Some(&PES15_OLD),
        PesVersion::Pes16 => Some(&PES16_OLD),
        PesVersion::Pes17 => Some(&PES17_OLD),
        PesVersion::Pes18 | PesVersion::Pes19 | PesVersion::Pes20 | PesVersion::Pes21 => None,
    }
}
