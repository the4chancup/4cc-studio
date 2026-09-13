//! The vertex-field vocabulary: what a field holds ([`DatumType`]) and how
//! each vertex's value is stored ([`DatumFormat`]), as the field
//! descriptor's type and format words name them.

/// What a vertex field holds, as the field descriptor's type word names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatumType {
    /// Vertex position.
    Position = 2,
    /// Vertex normal.
    Normal = 3,
    /// Vertex color.
    Color = 4,
    /// The first uv map.
    Uv0 = 7,
    /// The second uv map.
    Uv1 = 8,
    /// The third uv map.
    Uv2 = 9,
    /// The fourth uv map.
    Uv3 = 10,
    /// Vertex tangent.
    Tangent = 15,
    /// Vertex bitangent.
    Bitangent = 16,
    /// Per-vertex bone weights.
    BoneWeights = 17,
    /// Per-vertex bone indices into the mesh's bone group.
    BoneIndices = 18,
}

/// How a vertex field stores each vertex's value, as the descriptor's format
/// word names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatumFormat {
    /// One `u16`.
    Uint16 = 1,
    /// One `u32`.
    Uint32 = 2,
    /// One `f32`.
    Float32 = 3,
    /// Two `f32`.
    DoubleFloat32 = 4,
    /// Three `f32`.
    TripleFloat32 = 5,
    /// Four `f32`.
    QuadFloat32 = 6,
    /// A 3x4 `f32` matrix (bone data's inverse bind matrices).
    Float32Matrix34 = 7,
    /// Four `i8`.
    QuadInt8 = 8,
    /// Four `u8` fixed-point fractions.
    QuadFloat8 = 9,
}

impl DatumType {
    /// The type a word names, when the format defines one.
    pub fn from_word(word: u32) -> Option<Self> {
        match word {
            2 => Some(DatumType::Position),
            3 => Some(DatumType::Normal),
            4 => Some(DatumType::Color),
            7 => Some(DatumType::Uv0),
            8 => Some(DatumType::Uv1),
            9 => Some(DatumType::Uv2),
            10 => Some(DatumType::Uv3),
            15 => Some(DatumType::Tangent),
            16 => Some(DatumType::Bitangent),
            17 => Some(DatumType::BoneWeights),
            18 => Some(DatumType::BoneIndices),
            _ => None,
        }
    }

    /// The word naming this type.
    pub fn word(self) -> u32 {
        self as u32
    }
}

impl DatumFormat {
    /// The format a word names, when the format defines one.
    pub fn from_word(word: u32) -> Option<Self> {
        match word {
            1 => Some(DatumFormat::Uint16),
            2 => Some(DatumFormat::Uint32),
            3 => Some(DatumFormat::Float32),
            4 => Some(DatumFormat::DoubleFloat32),
            5 => Some(DatumFormat::TripleFloat32),
            6 => Some(DatumFormat::QuadFloat32),
            7 => Some(DatumFormat::Float32Matrix34),
            8 => Some(DatumFormat::QuadInt8),
            9 => Some(DatumFormat::QuadFloat8),
            _ => None,
        }
    }

    /// The word naming this format.
    pub fn word(self) -> u32 {
        self as u32
    }

    /// Bytes per vertex.
    pub fn size(self) -> usize {
        match self {
            DatumFormat::Uint16 => 2,
            DatumFormat::Uint32
            | DatumFormat::Float32
            | DatumFormat::QuadInt8
            | DatumFormat::QuadFloat8 => 4,
            DatumFormat::DoubleFloat32 => 8,
            DatumFormat::TripleFloat32 => 12,
            DatumFormat::QuadFloat32 => 16,
            DatumFormat::Float32Matrix34 => 48,
        }
    }
}
