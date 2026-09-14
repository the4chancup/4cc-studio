//! What a conversion could not carry, or had to decide: findings with stable codes, mirroring
//! `fmdl::check`'s shape minus severity — every loss is informational; the consuming tool's
//! message catalog decides.
//!
//! Codes, in the order the importers and exporters can emit them:
//!
//! - `bone_matrix_unknown` (Bone): no companion SKL and the name is in none of PES21's
//!   template tables; the identity matrix is used.
//! - `bone_slot_dropped` (Mesh): a weighted slot past the mesh's bone group was zeroed and
//!   the vertex renormalized; `detail` is how many slots were dropped.
//! - `material_family_approximated` (Material): `InferredFamily::approximate` — no shader
//!   rule named the shader and the family is the closest fit.
//! - `material_split_by_flags` (Material): meshes of one FMDL material instance carried
//!   different flags, so the instance split; `detail` is the new material's name.
//! - `material_texture_unused` (Material): a canonical role the target format has no sampler
//!   for; `detail` is the role.
//! - `vertex_bitangents_dropped` (Mesh): FMDL has no bitangent attribute.
//! - `dummy_texture_added` (Material): a `Shaded`/`Metal` material missing its normal or
//!   specular map got the game's dummy; `detail` is the sampler name.
//! - `native_field_dropped` (Model or Mesh): a `.model` field the IR has no home for was
//!   non-default; `detail` names it.
//! - `bone_folded_for_version` (Bone): a bone the target version lacks folded onto the bone
//!   the fold table names; `detail` is `"<bone> -> <target>"`.
//! - `bone_folded_by_position` (Bone): no fold-table entry, so the bone folded onto the
//!   nearest target body bone by rest position — a guess the user should hear about;
//!   `detail` is `"<bone> -> <target>"`.
//! - `skeleton_retargeted` (Model): bones re-bound to the target version's rest pose;
//!   `detail` is how many bones moved.

/// What a finding is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Subject {
    /// The model as a whole.
    Model,
    /// The mesh at this index.
    Mesh(usize),
    /// The material at this index.
    Material(usize),
    /// The bone at this index.
    Bone(usize),
}

/// Something a conversion could not carry, or had to decide; the consuming tool maps `code`
/// to its message catalog. Never an error: the output is valid, just not identical.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The stable finding code.
    pub code: &'static str,
    /// What it is about.
    pub subject: Subject,
    /// The name or count the message needs (a bone name, a material name, a vertex count);
    /// empty when the subject says it all.
    pub detail: String,
}
