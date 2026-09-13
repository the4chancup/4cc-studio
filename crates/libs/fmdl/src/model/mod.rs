//! `Model`: the semantic layer the ops work on. `from_file` resolves every
//! index through the file's tables (strings, bounding boxes, bone groups,
//! materials, textures, the shared texture/parameter assignment table,
//! mesh-group assignments) and reads the extension headers from the string
//! table's tail. Every dangling index is an error, never a panic.

mod from_file;
#[cfg(test)]
mod tests;
mod to_file;

use crate::format::MeshVertices;

/// A parsed FMDL at the semantic level: bones, materials, meshes, groups
/// and the extension header, with every index resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    /// Bones in file order; `Bone::parent` indexes this vector.
    pub bones: Vec<Bone>,
    /// Material instances in file order.
    pub materials: Vec<MaterialInstance>,
    /// Meshes in file order.
    pub meshes: Vec<Mesh>,
    /// Mesh groups in file order; `MeshGroup::parent` indexes this vector.
    pub mesh_groups: Vec<MeshGroup>,
    /// The `X-FMDL-Extensions` header: which encodings the file declares.
    pub extensions: Extensions,
    /// Section-1 block 1, 64 bytes per bone in Konami files; carried as is
    /// (the add-on writes it empty).
    pub bone_matrices: Option<Vec<u8>>,
}

/// A bone with its name and parents resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct Bone {
    /// The bone's name (e.g. `sk_head`).
    pub name: String,
    /// Index into `Model::bones`, `None` for a root.
    pub parent: Option<usize>,
    /// The bone's bounding box.
    pub bounding_box: BoundingBox,
    /// Position relative to the parent bone.
    pub local_position: [f32; 4],
    /// Position in model space.
    pub world_position: [f32; 4],
}

/// An axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    /// The maximum corner.
    pub max: [f32; 4],
    /// The minimum corner.
    pub min: [f32; 4],
}

/// A texture reference with its strings resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    /// The texture file name (kept verbatim, extension included).
    pub file_name: String,
    /// The texture directory.
    pub directory: String,
}

/// A material instance: material, texture and parameter runs resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct MaterialInstance {
    /// The instance's name.
    pub name: String,
    /// The material's shader name.
    pub shader: String,
    /// The material's technique name.
    pub technique: String,
    /// (sampler name, texture), in file order.
    pub textures: Vec<(String, Texture)>,
    /// (parameter name, four floats), in file order.
    pub parameters: Vec<(String, [f32; 4])>,
}

/// A mesh with its vertex data, faces and references resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    /// The decoded vertices; bone indices index `bone_group`.
    pub vertices: MeshVertices,
    /// Triangles as indices into `vertices`.
    pub faces: Vec<[u16; 3]>,
    /// Indices into `Model::bones`, at most 32; empty when the mesh is not
    /// skinned (the record's `bone_group_id` is ignored then).
    pub bone_group: Vec<usize>,
    /// Index into `Model::materials`.
    pub material: usize,
    /// Transparency draw flags.
    pub alpha_flags: u8,
    /// Shadow draw flags.
    pub shadow_flags: u8,
    /// The `Has-Antiblur-Meshes` extension header lists this mesh.
    pub has_antiblur_meshes: bool,
    /// The `Is-Antiblur-Meshes` extension header lists this mesh.
    pub is_antiblur_mesh: bool,
    /// The mesh's custom bounding box, when the `Custom-Bounding-Box-Meshes`
    /// header applies.
    pub custom_bounding_box: Option<BoundingBox>,
}

/// A mesh group with its meshes, parent and bounding box resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshGroup {
    /// The group's name.
    pub name: String,
    /// Index into `Model::mesh_groups`, `None` for a root.
    pub parent: Option<usize>,
    /// Indices into `Model::meshes`, in assignment order.
    pub meshes: Vec<usize>,
    /// The bounding box its assignment names; `None` for a group no
    /// assignment covers.
    pub bounding_box: Option<BoundingBox>,
    /// Whether the group is visible.
    pub visible: bool,
    /// The `Split-Mesh-Groups` extension header lists this group.
    pub split_mesh_group: bool,
}

/// The `X-FMDL-Extensions` flags the file declares.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Extensions {
    /// `mesh-splitting`: meshes may split for bone-group limits.
    pub mesh_splitting: bool,
    /// `antiblur`: the file carries anti-blur mesh data.
    pub antiblur: bool,
    /// `vertex-loop-preservation`: vertex order is meaningful.
    pub vertex_loop_preservation: bool,
    /// Extension values outside the known three, kept verbatim so a rewrite
    /// can emit them again.
    pub other: Vec<String>,
}
