//! Fox Engine FTEX texture container: FTEX <-> DDS.
//!
//! An FTEX is a 64-byte little-endian header, one 16-byte mip record per frame
//! (six frames per mip level for cube maps), then the frame data: either raw
//! or zlib-compressed in 16 KiB chunks. The DDS side is the classic 128-byte
//! header plus the 20-byte DX10 extension for the DXGI-only formats.
//! `ftex_to_dds` reproduces pes-file-tools' output byte for byte;
//! `dds_to_ftex` matches its header and mip-table fields while its zlib chunks
//! differ in bytes (miniz_oxide vs Python zlib), so write-parity is checked by
//! converting back.

mod dds;
mod format;
mod from_dds;
mod to_dds;

pub use format::{mip_size, ColorSpace, FtexInfo, PixelFormat};
pub use from_dds::dds_to_ftex;
pub use to_dds::{ftex_to_dds, info};

/// Why an FTEX or DDS buffer could not be converted.
#[derive(Debug, thiserror::Error)]
pub enum FtexError {
    /// The buffer ends before a structure that points past it.
    #[error("buffer is truncated")]
    Truncated,
    /// Wrong `FTEX` or `DDS ` signature.
    #[error("invalid magic")]
    BadMagic,
    /// The FTEX version is outside the accepted 2.025..=2.045 range.
    #[error("unsupported ftex version {0}")]
    UnsupportedVersion(f32),
    /// An FTEX variant field carries a value this code does not handle (says
    /// which: ftexs count, zero mipmaps, cube map with depth).
    #[error("unsupported ftex variant: {0}")]
    UnsupportedVariant(&'static str),
    /// An FTEX pixel-format id outside the known set.
    #[error("unsupported ftex pixel format id {0}")]
    UnsupportedFormat(u16),
    /// A DDS field the converter does not accept (says which).
    #[error("unsupported dds: {0}")]
    UnsupportedDds(&'static str),
    /// A zlib stream failed to decompress or compress.
    #[error("zlib error: {0}")]
    Zlib(#[from] std::io::Error),
    /// A mip record's index does not match its position in the mip list.
    #[error("unexpected mipmap index: expected {expected}, found {found}")]
    UnexpectedMipmap {
        /// The mip level the record should describe.
        expected: u8,
        /// The index the record carries.
        found: u8,
    },
}
