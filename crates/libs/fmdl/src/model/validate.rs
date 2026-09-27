//! The `Model` invariants `from_file` guarantees and `to_file` plus the
//! ops rely on: one list, so a hand-built model gets the same errors.
//! The format's size limits (32 bones, 65535 vertices, 21845 faces) are
//! *not* here — `ops::split` exists to take meshes over them.

use super::{Mesh, Model};
use crate::format::FmdlError;

/// Every parent chain must reach a root; a loop is `ParentCycle`.
fn check_cycles(parents: &[Option<usize>], what: &'static str) -> Result<(), FmdlError> {
    for start in 0..parents.len() {
        let mut seen = vec![false; parents.len()];
        let mut current = Some(start);
        while let Some(index) = current {
            if seen[index] {
                return Err(FmdlError::ParentCycle(what));
            }
            seen[index] = true;
            current = parents[index];
        }
    }
    Ok(())
}

impl Model {
    /// The invariants every op and `to_file` rely on; `from_file` output
    /// always passes.
    pub fn validate(&self) -> Result<(), FmdlError> {
        for bone in &self.bones {
            if let Some(parent) = bone.parent
                && parent >= self.bones.len()
            {
                return Err(FmdlError::BadReference {
                    what: "bone",
                    index: parent,
                });
            }
        }
        let bone_parents: Vec<Option<usize>> = self.bones.iter().map(|bone| bone.parent).collect();
        check_cycles(&bone_parents, "bone")?;

        for mesh in &self.meshes {
            if mesh.material >= self.materials.len() {
                return Err(FmdlError::BadReference {
                    what: "material instance",
                    index: mesh.material,
                });
            }
            for bone in &mesh.bone_group {
                if *bone >= self.bones.len() {
                    return Err(FmdlError::BadReference {
                        what: "bone",
                        index: *bone,
                    });
                }
            }
            mesh.validate_vertices()?;
        }

        for group in &self.mesh_groups {
            if let Some(parent) = group.parent
                && parent >= self.mesh_groups.len()
            {
                return Err(FmdlError::BadReference {
                    what: "mesh group",
                    index: parent,
                });
            }
        }
        let group_parents: Vec<Option<usize>> =
            self.mesh_groups.iter().map(|group| group.parent).collect();
        check_cycles(&group_parents, "mesh group")?;
        for group in &self.mesh_groups {
            for mesh in &group.meshes {
                if *mesh >= self.meshes.len() {
                    return Err(FmdlError::BadReference {
                        what: "mesh",
                        index: *mesh,
                    });
                }
            }
        }
        Ok(())
    }
}

impl Mesh {
    /// The mesh-local part of `Model::validate`: attribute lengths and
    /// face indices.
    pub(crate) fn validate_vertices(&self) -> Result<(), FmdlError> {
        let vertices = &self.vertices;
        let count = vertices.positions.len();
        for length in [
            vertices.normals.as_ref().map(Vec::len),
            vertices.tangents.as_ref().map(Vec::len),
            vertices.colors.as_ref().map(Vec::len),
            vertices.bone_weights.as_ref().map(Vec::len),
            vertices.bone_indices.as_ref().map(Vec::len),
        ]
        .into_iter()
        .flatten()
        .chain(vertices.uvs.iter().map(Vec::len))
        {
            if length != count {
                return Err(FmdlError::VertexMismatch("attribute count mismatch"));
            }
        }
        if vertices.uvs.len() > 4 {
            return Err(FmdlError::InvalidVertexFormat("more than four uv maps"));
        }
        if vertices.uv_high_precision.len() != vertices.uvs.len() {
            return Err(FmdlError::InvalidVertexFormat(
                "uv precision flags do not match uv maps",
            ));
        }
        if vertices.bone_weights.is_some() != vertices.bone_indices.is_some() {
            return Err(FmdlError::InvalidVertexFormat(
                "bone weights and bone indices must come together",
            ));
        }
        for face in &self.faces {
            for index in face {
                if usize::from(*index) >= count {
                    return Err(FmdlError::BadReference {
                        what: "face vertex",
                        index: usize::from(*index),
                    });
                }
            }
        }
        Ok(())
    }
}
