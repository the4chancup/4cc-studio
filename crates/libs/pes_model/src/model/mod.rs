//! `Model`: the semantic layer the ops work on. `from_file` decodes the
//! vertices, splits the face stream into level 0 and the lower LOD levels,
//! resolves each mesh's bone group to `bones` indices, and sorts the
//! annotations by meaning: the add-on's kind 128 is the mesh name and kind
//! 129 a per-mesh extension header; every other kind is a Konami tag kept as
//! `(kind, text)`. `to_file` writes it back the add-on's way;
//! `from_file(to_file(m)) == m`.
//!
//! One place is not lossless at the format level, and the plan accepts it:
//! a tag annotation's section-3 record index and `unknown` word are dropped
//! (the record is the constant `0 0 0 2 2 2 0` and `unknown` is 7, both of
//! which `to_file` rewrites).

#[cfg(test)]
mod tests;

use crate::format::{
    Annotation, Bone, BoundingBox, EditorItem, FaceStream, Geometry as FileGeometry, LodRecord,
    Mesh as FileMesh, MeshVertices, ModelError, PreFoxModel,
};

/// A `.model` as the ops see it: bones, material names, meshes with decoded vertices. Built from
/// a `PreFoxModel` and written back to a fresh one; `from_file(to_file(m)) == m`.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    /// Header flags word, carried (0 in every player part).
    pub flags: u32,
    /// Bone names and inverse bind matrices, in file order.
    pub bones: Vec<Bone>,
    /// Material names, in order; the definitions live in the sibling `.mtl`.
    pub materials: Vec<String>,
    /// Meshes in file order.
    pub meshes: Vec<Mesh>,
    /// Annotation strings no mesh refers to: the add-on's model-level headers
    /// (`Skeleton-Type: Simplified`).
    pub extension_headers: Vec<String>,
    /// The model's bounds, as read; recomputed by ops that move vertices.
    pub bounds: BoundingBox,
    /// Section 7's LOD record, as read; `LodRecord::for_levels` for a new model.
    pub lod: LodRecord,
}

/// A mesh with its vertices decoded, faces split by LOD level and every
/// reference resolved.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    /// The add-on's mesh name (annotation kind 128).
    pub name: Option<String>,
    /// The add-on's per-mesh extension headers (annotation kind 129), in file order.
    pub extension_headers: Vec<String>,
    /// Konami's annotations, `(kind, text)` in file order: 1 the part name, 2 `DSpecularS`, 7 a
    /// normal-map name, 10 the part name again (meanings as observed, not documented).
    pub tags: Vec<(u32, String)>,
    /// The decoded vertices; bone indices index `bone_group`.
    pub vertices: MeshVertices,
    /// Level-0 faces; vertex indices index `vertices`.
    pub faces: Vec<[u16; 3]>,
    /// Lower LOD levels, decreasing detail, each a face list over the same vertices.
    pub lower_lods: Vec<Vec<[u16; 3]>>,
    /// Indices into `Model::bones`; `vertices.bone_indices` index this list. Empty when the
    /// mesh has no bone group.
    pub bone_group: Vec<usize>,
    /// Index into `Model::materials`.
    pub material: usize,
    /// The mesh's bounds, as read.
    pub bounds: BoundingBox,
    /// The geometry extras' order word (0 in every player part; `None` only in the version-17
    /// layout, which reads as 0 here).
    pub order: u32,
    /// Editor-data items (none in any player part).
    pub editor_data: Vec<EditorItem>,
}

/// Checks that every face index is below `count`.
fn check_indices<'a>(
    faces: impl Iterator<Item = &'a [u16; 3]>,
    count: usize,
) -> Result<(), ModelError> {
    for face in faces {
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

/// `text`'s index in `strings`, appending it when new (the add-on's
/// de-duplication).
fn intern(strings: &mut Vec<String>, text: &str) -> usize {
    match strings.iter().position(|entry| entry == text) {
        Some(index) => index,
        None => {
            strings.push(text.to_owned());
            strings.len() - 1
        }
    }
}

impl Model {
    /// Builds a `Model` from a `PreFoxModel`, resolving every index through
    /// the file's tables.
    pub fn from_file(file: &PreFoxModel) -> Result<Model, ModelError> {
        let mut meshes = Vec::with_capacity(file.meshes.len());
        let mut referenced = vec![false; file.annotation_strings.len()];
        for mesh in &file.meshes {
            let geometry = file
                .geometries
                .get(mesh.geometry)
                .ok_or(ModelError::BadReference {
                    what: "geometry",
                    offset: mesh.geometry as i64,
                })?;
            let vertices = geometry.decode_vertices()?;
            let faces = geometry.faces.level(0)?;
            let mut lower_lods = Vec::with_capacity(geometry.faces.lod_ranges.len());
            for level in 1..geometry.faces.lod_ranges.len() {
                lower_lods.push(geometry.faces.level(level)?);
            }
            check_indices(
                faces.iter().chain(lower_lods.iter().flatten()),
                vertices.len(),
            )?;
            let bone_group = match mesh.bone_group {
                Some(index) => file
                    .bone_groups
                    .get(index)
                    .ok_or(ModelError::BadReference {
                        what: "bone group",
                        offset: index as i64,
                    })?
                    .iter()
                    .map(|bone| usize::from(*bone))
                    .collect(),
                None => Vec::new(),
            };
            if mesh.material >= file.material_names.len() {
                return Err(ModelError::BadReference {
                    what: "material",
                    offset: mesh.material as i64,
                });
            }
            let mut name = None;
            let mut extension_headers = Vec::new();
            let mut tags = Vec::new();
            for annotation in &mesh.annotations {
                let text = file.annotation_strings.get(annotation.string).ok_or(
                    ModelError::BadReference {
                        what: "annotation string",
                        offset: annotation.string as i64,
                    },
                )?;
                referenced[annotation.string] = true;
                match annotation.kind {
                    Annotation::MESH_NAME => {
                        if name.is_some() {
                            return Err(ModelError::InvalidModel("two mesh names"));
                        }
                        name = Some(text.clone());
                    }
                    Annotation::EXTENSION_HEADER => extension_headers.push(text.clone()),
                    _ => tags.push((annotation.kind, text.clone())),
                }
            }
            // The version-17 layout has no order word; it reads as 0.
            let order = geometry.order.unwrap_or(0);
            // Konami meshes carry the array; a version-17 mesh has none.
            let editor_data = mesh.editor_data.clone().unwrap_or_default();
            meshes.push(Mesh {
                name,
                extension_headers,
                tags,
                vertices,
                faces,
                lower_lods,
                bone_group,
                material: mesh.material,
                bounds: geometry.bounds.clone(),
                order,
                editor_data,
            });
        }
        let extension_headers = file
            .annotation_strings
            .iter()
            .enumerate()
            .filter(|(index, _)| !referenced[*index])
            .map(|(_, text)| text.clone())
            .collect();
        Ok(Model {
            flags: file.flags,
            bones: file.bones.clone(),
            materials: file.material_names.clone(),
            meshes,
            extension_headers,
            bounds: file.bounds.clone(),
            lod: file.lod.clone(),
        })
    }

    /// Writes the model out as a fresh `PreFoxModel`, laid out the add-on's way.
    pub fn to_file(&self) -> Result<PreFoxModel, ModelError> {
        let mut bone_groups = Vec::new();
        let mut mesh_bone_group = Vec::with_capacity(self.meshes.len());
        for mesh in &self.meshes {
            if mesh.bone_group.is_empty() {
                mesh_bone_group.push(None);
                continue;
            }
            let mut group = Vec::with_capacity(mesh.bone_group.len());
            for bone in &mesh.bone_group {
                if *bone >= self.bones.len() {
                    return Err(ModelError::BadReference {
                        what: "bone",
                        offset: *bone as i64,
                    });
                }
                group.push(
                    u16::try_from(*bone)
                        .map_err(|_| ModelError::WriteLayout("a bone index beyond u16"))?,
                );
            }
            mesh_bone_group.push(Some(bone_groups.len()));
            bone_groups.push(group);
        }

        // One annotation string per text, de-duplicated; one section-3 record
        // per Konami tag.
        let mut annotation_strings = Vec::new();
        let mut annotation_records = Vec::new();
        let mut annotations = Vec::with_capacity(self.meshes.len());
        for mesh in &self.meshes {
            let mut list = Vec::new();
            if let Some(name) = &mesh.name {
                list.push(Annotation {
                    string: intern(&mut annotation_strings, name),
                    record: None,
                    unknown: 7,
                    kind: Annotation::MESH_NAME,
                });
            }
            for header in &mesh.extension_headers {
                list.push(Annotation {
                    string: intern(&mut annotation_strings, header),
                    record: None,
                    unknown: 7,
                    kind: Annotation::EXTENSION_HEADER,
                });
            }
            for (kind, text) in &mesh.tags {
                let record = annotation_records.len();
                annotation_records.push([0, 0, 0, 2, 2, 2, 0]);
                list.push(Annotation {
                    string: intern(&mut annotation_strings, text),
                    record: Some(record),
                    unknown: 7,
                    kind: *kind,
                });
            }
            annotations.push(list);
        }
        for header in &self.extension_headers {
            intern(&mut annotation_strings, header);
        }

        let mut geometries = Vec::with_capacity(self.meshes.len());
        for mesh in &self.meshes {
            check_indices(
                mesh.faces.iter().chain(mesh.lower_lods.iter().flatten()),
                mesh.vertices.len(),
            )?;
            let mut indices = Vec::new();
            for face in mesh.faces.iter().chain(mesh.lower_lods.iter().flatten()) {
                indices.extend_from_slice(face);
            }
            // The LOD table partitions the stream, level 0 first; with no
            // lower levels the format carries no table at all.
            let mut lod_ranges = Vec::with_capacity(mesh.lower_lods.len() + 1);
            if !mesh.lower_lods.is_empty() {
                let mut start = 0usize;
                for level in std::iter::once(&mesh.faces).chain(&mesh.lower_lods) {
                    let end = start + level.len() * 3;
                    lod_ranges.push((start as u32, end as u32));
                    start = end;
                }
            }
            geometries.push(FileGeometry {
                vertex_fields: mesh.vertices.to_fields()?,
                faces: FaceStream {
                    indices,
                    lod_ranges,
                },
                bounds: mesh.bounds.clone(),
                order: Some(mesh.order),
            });
        }

        let mut meshes = Vec::with_capacity(self.meshes.len());
        for (index, mesh) in self.meshes.iter().enumerate() {
            if mesh.material >= self.materials.len() {
                return Err(ModelError::BadReference {
                    what: "material",
                    offset: mesh.material as i64,
                });
            }
            meshes.push(FileMesh {
                geometry: index,
                material: mesh.material,
                bone_group: mesh_bone_group[index],
                annotations: annotations[index].clone(),
                // Konami writes the array present but empty.
                editor_data: Some(mesh.editor_data.clone()),
            });
        }

        Ok(PreFoxModel {
            version: 19,
            flags: self.flags,
            bones: self.bones.clone(),
            bone_groups,
            material_names: self.materials.clone(),
            annotation_strings,
            annotation_records,
            geometries,
            meshes,
            bounds: self.bounds.clone(),
            lod: self.lod.clone(),
        })
    }
}
