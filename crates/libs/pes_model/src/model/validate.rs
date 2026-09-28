//! The `Model` invariants `from_file` guarantees and `to_file` plus the
//! ops rely on: one list, so a hand-built model gets the same errors.
//! The format's size limits (64 bones, 65535 vertices, 21845 faces) are
//! *not* here — `ops::split` exists to take meshes over them.

use super::{Mesh, Model};
use crate::format::ModelError;

impl Model {
    /// The invariants every op and `to_file` rely on; `from_file` output
    /// always passes.
    pub fn validate(&self) -> Result<(), ModelError> {
        for mesh in &self.meshes {
            if mesh.material >= self.materials.len() {
                return Err(ModelError::BadReference {
                    what: "material",
                    offset: mesh.material as i64,
                });
            }
            for bone in &mesh.bone_group {
                if *bone >= self.bones.len() {
                    return Err(ModelError::BadReference {
                        what: "bone",
                        offset: *bone as i64,
                    });
                }
            }
            mesh.validate()?;
        }
        Ok(())
    }
}

impl Mesh {
    /// The mesh-local half of `Model::validate`: attribute lengths, uv
    /// map and weight-field rules, and face indices.
    pub(crate) fn validate(&self) -> Result<(), ModelError> {
        let vertices = &self.vertices;
        let count = vertices.positions.len();
        for length in [
            vertices.normals.as_ref().map(Vec::len),
            vertices.tangents.as_ref().map(Vec::len),
            vertices.bitangents.as_ref().map(Vec::len),
            vertices.colors.as_ref().map(Vec::len),
            vertices.bone_indices.as_ref().map(Vec::len),
            vertices.bone_weights.as_ref().map(Vec::len),
        ]
        .into_iter()
        .flatten()
        .chain(vertices.uvs.iter().map(Vec::len))
        {
            if length != count {
                return Err(ModelError::VertexMismatch("attribute count mismatch"));
            }
        }
        if vertices.uvs.len() > 4 {
            return Err(ModelError::InvalidVertexFormat("more than four uv maps"));
        }
        if vertices.bone_weights.is_some() && vertices.bone_indices.is_none() {
            return Err(ModelError::InvalidVertexFormat(
                "bone weights without bone indices",
            ));
        }
        if vertices.bone_weights.is_some() && !matches!(vertices.bone_weight_width, 2..=4) {
            return Err(ModelError::VertexMismatch("bone weight width"));
        }
        for face in self.faces.iter().chain(self.lower_lods.iter().flatten()) {
            for index in face {
                if usize::from(*index) >= count {
                    return Err(ModelError::BadReference {
                        what: "vertex",
                        offset: i64::from(*index),
                    });
                }
            }
        }
        Ok(())
    }
}
