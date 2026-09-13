//! Mesh splitting: FMDL meshes have hard limits on vertices, faces and
//! bone-group size that real models hit. Splitting cuts an oversized mesh
//! into component meshes under the soft limits that together contain the
//! same geometry; the split source mesh is replaced in its group by a
//! child group named `split-mesh` (flagged `split_mesh_group`) holding the
//! components, and the `mesh-splitting` extension flag marks the file.
//!
//! A vertex may occur in several components. Two component vertices X and
//! Y describe the same source vertex iff they share the same stored byte
//! encoding (modulo bone-index differences between different bone
//! groups) *and* X and Y are both the Nth vertex of that encoding in
//! their component. To keep that convention decodable, every vertex that
//! shares a source vertex's *split key* (topological key: position, bone
//! weights and bone indices as stored) travels with it into every
//! component that takes any of them, in the same relative order; this is
//! also what keeps `vertex-loop-preservation` loops intact across a
//! split.

mod build;
mod combine;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use crate::format::{FmdlError, MeshVertices};
use crate::model::{Mesh, MeshGroup, Model};

/// The largest bone group an FMDL mesh may reference.
pub const BONE_LIMIT_HARD: usize = 32;
/// The bone-group size a split component tries to stay below.
pub const BONE_LIMIT_SOFT: usize = 30;
/// The largest vertex count an FMDL mesh may have.
pub const VERTEX_LIMIT_HARD: usize = 65535;
/// The vertex count a split component tries to stay below.
pub const VERTEX_LIMIT_SOFT: usize = 63000;
/// The largest face count an FMDL mesh may have.
pub const FACE_LIMIT_HARD: usize = 21845;
/// The face count a split component tries to stay below.
pub const FACE_LIMIT_SOFT: usize = 20000;

/// Bone names tried first as a fragment's base bone, in order.
const PREFERRED_BASE_BONES: [&str; 5] = [
    "sk_foot_l",
    "sk_foot_r",
    "sk_hand_l",
    "sk_hand_r",
    "skf_jaw",
];

/// The name of the child group a split mesh is replaced by.
const SPLIT_GROUP_NAME: &str = "split-mesh";

/// A vertex's bone mapping: `(model bone index, weight)` for every weight
/// greater than zero. Compared across components whose bone groups differ,
/// so it uses model bone indices, not bone-group slot numbers.
pub(super) type BoneMapping = Vec<(usize, u8)>;

/// Whether a mesh exceeds a hard limit (bones in its group, vertices,
/// faces).
pub fn needs_splitting(mesh: &Mesh) -> bool {
    mesh.bone_group.len() > BONE_LIMIT_HARD
        || mesh.vertices.positions.len() > VERTEX_LIMIT_HARD
        || mesh.faces.len() > FACE_LIMIT_HARD
}

/// The effective parent of every bone for splitting: the model's parents,
/// cycles broken, and the chest/belly/hip chain inverted so `sk_chest` is
/// the root when that chain is intact.
pub fn effective_parents(model: &Model) -> Vec<Option<usize>> {
    let mut parents = Vec::with_capacity(model.bones.len());
    for (index, bone) in model.bones.iter().enumerate() {
        let mut parent = bone.parent;
        if let Some(candidate) = parent {
            // Walking up through already-decided parents, reaching `index`
            // again would be a parent loop: cut it.
            let mut ancestor = Some(candidate);
            while let Some(bone_index) = ancestor {
                if bone_index == index {
                    parent = None;
                    break;
                }
                ancestor = parents.get(bone_index).copied().flatten();
            }
        }
        parents.push(parent);
    }

    // Invert `sk_chest -> sk_belly -> dsk_hip` when the intact chain runs
    // that way (belly's parent is the hip), so the chest is the effective
    // root. This avoids fragments that split a shirt off its torso.
    let index_of = |name: &str| model.bones.iter().position(|bone| bone.name == name);
    if let (Some(chest), Some(belly), Some(hip)) = (
        index_of("sk_chest"),
        index_of("sk_belly"),
        index_of("dsk_hip"),
    ) && parents[chest] == Some(belly)
        && parents[belly] == Some(hip)
        && parents[hip].is_none()
    {
        parents[chest] = None;
        parents[belly] = Some(chest);
        parents[hip] = Some(belly);
    }

    parents
}

/// Splits every mesh over a hard limit into component meshes under the
/// soft limits. Each split mesh is replaced, in its group, by a child
/// group named `split-mesh` (same box and visibility, `split_mesh_group`
/// set) holding the components; sets `extensions.mesh_splitting`.
/// `parents` overrides `effective_parents` (the model converter passes a
/// skeleton-extended hierarchy). Returns whether anything was split.
pub fn encode(model: &mut Model, parents: Option<&[Option<usize>]>) -> Result<bool, FmdlError> {
    let effective = match parents {
        Some(given) => {
            if given.len() != model.bones.len() {
                return Err(FmdlError::VertexMismatch(
                    "parents length does not match bone count",
                ));
            }
            given.to_vec()
        }
        None => effective_parents(model),
    };

    let mut components: HashMap<usize, Vec<Mesh>> = HashMap::new();
    for (index, mesh) in model.meshes.iter().enumerate() {
        if needs_splitting(mesh) {
            components.insert(index, build::split_mesh(model, mesh, &effective)?);
        }
    }
    if components.is_empty() {
        return Ok(false);
    }

    let old_meshes = std::mem::take(&mut model.meshes);
    let mut new_index = vec![usize::MAX; old_meshes.len()];
    let mut split_size: HashMap<usize, usize> = HashMap::new();
    for (index, mesh) in old_meshes.into_iter().enumerate() {
        new_index[index] = model.meshes.len();
        match components.remove(&index) {
            None => model.meshes.push(mesh),
            Some(component_meshes) => {
                split_size.insert(index, component_meshes.len());
                model.meshes.extend(component_meshes);
            }
        }
    }

    let mut new_groups = Vec::new();
    for (group_index, group) in model.mesh_groups.iter_mut().enumerate() {
        let mut meshes = Vec::with_capacity(group.meshes.len());
        for &mesh_index in &group.meshes {
            match split_size.get(&mesh_index) {
                None => meshes.push(new_index[mesh_index]),
                Some(&size) => {
                    let start = new_index[mesh_index];
                    new_groups.push(MeshGroup {
                        name: SPLIT_GROUP_NAME.to_owned(),
                        parent: Some(group_index),
                        meshes: (start..start + size).collect(),
                        bounding_box: group.bounding_box,
                        visible: group.visible,
                        split_mesh_group: true,
                    });
                }
            }
        }
        group.meshes = meshes;
    }
    model.mesh_groups.extend(new_groups);
    model.extensions.mesh_splitting = true;
    Ok(true)
}

/// Reassembles every `split_mesh_group`'s components into one mesh placed
/// in the parent group where the split group sat among its meshes
/// (appended if the parent had none), removes the split groups, renumbers
/// mesh and group indices, clears `extensions.mesh_splitting`. A split
/// group with a parent that is `None`, or with children, is
/// `FmdlError::BadMeshGroupAssignment`.
pub fn decode(model: &mut Model) -> Result<(), FmdlError> {
    let split_groups: Vec<usize> = model
        .mesh_groups
        .iter()
        .enumerate()
        .filter(|(_, group)| group.split_mesh_group)
        .map(|(index, _)| index)
        .collect();
    for &group_index in &split_groups {
        if model.mesh_groups[group_index].parent.is_none() {
            return Err(FmdlError::BadMeshGroupAssignment(
                "a split-mesh group has no parent",
            ));
        }
        if model
            .mesh_groups
            .iter()
            .any(|group| group.parent == Some(group_index))
        {
            return Err(FmdlError::BadMeshGroupAssignment(
                "a split-mesh group has children",
            ));
        }
    }

    // mesh index -> the split group it is a component of.
    let mut mesh_group_of: Vec<Option<usize>> = vec![None; model.meshes.len()];
    for &group_index in &split_groups {
        for &mesh_index in &model.mesh_groups[group_index].meshes {
            mesh_group_of[mesh_index] = Some(group_index);
        }
    }

    // Combine each split group's components while the old mesh list is
    // still intact, then rebuild the list: components collapse to their
    // combined mesh at the first component's position.
    let old_meshes = std::mem::take(&mut model.meshes);
    let mut combined_meshes: HashMap<usize, Mesh> = HashMap::new();
    for &group_index in &split_groups {
        combined_meshes.insert(
            group_index,
            combine::combine(&old_meshes, &model.mesh_groups[group_index])?,
        );
    }
    let mut combined_index: HashMap<usize, usize> = HashMap::new();
    let mut new_index = vec![usize::MAX; old_meshes.len()];
    for (index, mesh) in old_meshes.into_iter().enumerate() {
        match mesh_group_of[index] {
            None => {
                new_index[index] = model.meshes.len();
                model.meshes.push(mesh);
            }
            Some(group_index) => match combined_index.get(&group_index) {
                Some(&target) => new_index[index] = target,
                None => {
                    let target = model.meshes.len();
                    combined_index.insert(group_index, target);
                    new_index[index] = target;
                    model
                        .meshes
                        .push(combined_meshes.remove(&group_index).ok_or(
                            FmdlError::VertexMismatch("split group lost its combined mesh"),
                        )?);
                }
            },
        }
    }

    // Rebuild the group list without the split groups, then fix parents.
    let mut group_map = vec![usize::MAX; model.mesh_groups.len()];
    let mut new_groups: Vec<MeshGroup> = Vec::new();
    for (index, group) in model.mesh_groups.iter().enumerate() {
        if group.split_mesh_group {
            continue;
        }
        group_map[index] = new_groups.len();
        let mut group = group.clone();
        group.meshes = group.meshes.iter().map(|mesh| new_index[*mesh]).collect();
        for &split_index in &split_groups {
            if model.mesh_groups[split_index].parent == Some(index)
                && let Some(&target) = combined_index.get(&split_index)
            {
                group.meshes.push(target);
            }
        }
        new_groups.push(group);
    }
    for group in &mut new_groups {
        if let Some(parent) = group.parent {
            group.parent = Some(group_map[parent]);
        }
    }
    model.mesh_groups = new_groups;
    model.extensions.mesh_splitting = false;
    Ok(())
}

/// The model bone mapping of vertex `index` of `mesh`: `(model bone index,
/// weight)` pairs for weights greater than zero.
pub(super) fn bone_mapping(mesh: &Mesh, index: usize) -> Result<BoneMapping, FmdlError> {
    let mut mapping = Vec::new();
    if let (Some(weights), Some(indices)) =
        (&mesh.vertices.bone_weights, &mesh.vertices.bone_indices)
    {
        for component in 0..4 {
            if weights[index][component] > 0 {
                let slot = usize::from(indices[index][component]);
                let bone = mesh
                    .bone_group
                    .get(slot)
                    .copied()
                    .ok_or(FmdlError::BadReference {
                        what: "bone",
                        index: slot,
                    })?;
                mapping.push((bone, weights[index][component]));
            }
        }
    }
    Ok(mapping)
}

/// Faces and loose vertex sets that contain a descendant of one bone.
#[derive(Default, Clone)]
pub(super) struct StorableItems {
    /// Face indices into the source mesh's `faces`.
    pub(super) faces: HashSet<usize>,
    /// Equipresent set indices whose vertices no face references.
    pub(super) loose: HashSet<usize>,
}

/// Vertex `index`'s split key: the stored bytes of position, weights and
/// bone indices (all vertices of one mesh share the bone group, so group
/// slot numbers compare correctly here).
/// group slot; unmapped slots write 0 (a bone a zero weight never loads).
pub(super) fn push_vertex(
    vertices: &mut MeshVertices,
    source_mesh: &Mesh,
    source: usize,
    index_of: &HashMap<usize, u8>,
) -> Result<(), FmdlError> {
    let source_vertices = &source_mesh.vertices;
    vertices.positions.push(source_vertices.positions[source]);
    if let (Some(target), Some(source_normals)) = (&mut vertices.normals, &source_vertices.normals)
    {
        target.push(source_normals[source]);
    }
    if let (Some(target), Some(source_tangents)) =
        (&mut vertices.tangents, &source_vertices.tangents)
    {
        target.push(source_tangents[source]);
    }
    if let (Some(target), Some(source_colors)) = (&mut vertices.colors, &source_vertices.colors) {
        target.push(source_colors[source]);
    }
    for (target, source_uvs) in vertices.uvs.iter_mut().zip(&source_vertices.uvs) {
        target.push(source_uvs[source]);
    }
    if let (Some(target), Some(source_weights)) =
        (&mut vertices.bone_weights, &source_vertices.bone_weights)
    {
        target.push(source_weights[source]);
    }
    if let (Some(target), Some(source_indices)) =
        (&mut vertices.bone_indices, &source_vertices.bone_indices)
    {
        let weights =
            source_vertices
                .bone_weights
                .as_ref()
                .ok_or(FmdlError::InvalidVertexFormat(
                    "bone indices without bone weights",
                ))?;
        let mut remapped = [0u8; 4];
        for component in 0..4 {
            let slot = usize::from(source_indices[source][component]);
            match source_mesh.bone_group.get(slot).copied() {
                Some(bone) => {
                    if let Some(&mapped) = index_of.get(&bone) {
                        remapped[component] = mapped;
                    }
                }
                // A slot a zero weight never loads may hold anything.
                None if weights[source][component] > 0 => {
                    return Err(FmdlError::BadReference {
                        what: "bone",
                        index: slot,
                    });
                }
                None => {}
            }
        }
        target.push(remapped);
    }
    Ok(())
}

/// An empty `MeshVertices` with `source`'s attribute layout.
pub(super) fn empty_like(source: &MeshVertices, capacity: usize) -> MeshVertices {
    MeshVertices {
        positions: Vec::with_capacity(capacity),
        normals: source
            .normals
            .as_ref()
            .map(|_| Vec::with_capacity(capacity)),
        tangents: source
            .tangents
            .as_ref()
            .map(|_| Vec::with_capacity(capacity)),
        colors: source.colors.as_ref().map(|_| Vec::with_capacity(capacity)),
        uvs: source
            .uvs
            .iter()
            .map(|_| Vec::with_capacity(capacity))
            .collect(),
        uv_high_precision: source.uv_high_precision.clone(),
        bone_weights: source
            .bone_weights
            .as_ref()
            .map(|_| Vec::with_capacity(capacity)),
        bone_indices: source
            .bone_indices
            .as_ref()
            .map(|_| Vec::with_capacity(capacity)),
    }
}
