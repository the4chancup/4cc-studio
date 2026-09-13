//! Anti-blur encoding: PES duplicates flagged meshes with a fuzzblock
//! material so motion-blur passes can draw them separately. `encode` adds
//! the duplicates, `decode` strips them again.

use std::collections::HashMap;

use crate::model::{MaterialInstance, Model, Texture};

/// The directory the fuzzblock materials' dummy normal and specular maps
/// live in.
const TEXTURE_DIRECTORY: &str = "/Assets/pes16/model/character/common/sourceimages/";

/// Adds the anti-blur duplicate of every mesh flagged `has_antiblur_meshes`
/// right after it, with a fuzzblock material, and sets
/// `extensions.antiblur`. Idempotent on a model already encoded (a mesh
/// flagged `is_antiblur_mesh` is never duplicated, and a source mesh
/// followed by its duplicate is skipped).
pub fn encode(model: &mut Model) {
    let old_meshes = std::mem::take(&mut model.meshes);
    let mut meshes = Vec::with_capacity(old_meshes.len());
    // Old mesh index to its index in the new list, and to its duplicate's.
    let mut old_to_new = Vec::with_capacity(old_meshes.len());
    let mut duplicates: HashMap<usize, usize> = HashMap::new();
    let mut antiblur_materials: HashMap<usize, usize> = HashMap::new();
    for (index, mesh) in old_meshes.iter().enumerate() {
        old_to_new.push(meshes.len());
        meshes.push(mesh.clone());
        let already_duplicated = old_meshes
            .get(index + 1)
            .is_some_and(|next| next.is_antiblur_mesh);
        if mesh.has_antiblur_meshes && !mesh.is_antiblur_mesh && !already_duplicated {
            let material = *antiblur_materials.entry(mesh.material).or_insert_with(|| {
                let index = model.materials.len();
                model
                    .materials
                    .push(antiblur_material(&model.materials[mesh.material]));
                index
            });
            let mut duplicate = mesh.clone();
            duplicate.material = material;
            duplicate.alpha_flags = 128 | (mesh.alpha_flags & 32);
            duplicate.shadow_flags = 1;
            duplicate.has_antiblur_meshes = false;
            duplicate.is_antiblur_mesh = true;
            duplicate.custom_bounding_box = None;
            duplicates.insert(index, meshes.len());
            meshes.push(duplicate);
        }
    }
    model.meshes = meshes;
    for group in &mut model.mesh_groups {
        let mut list = Vec::with_capacity(group.meshes.len());
        for mesh in &group.meshes {
            list.push(old_to_new[*mesh]);
            if let Some(duplicate) = duplicates.get(mesh) {
                list.push(*duplicate);
            }
        }
        group.meshes = list;
    }
    model.extensions.antiblur = true;
}

/// Removes every mesh flagged `is_antiblur_mesh` and every material only
/// those meshes used, renumbers group mesh lists and mesh material indices,
/// and clears `extensions.antiblur`.
pub fn decode(model: &mut Model) {
    let mut kept = Vec::with_capacity(model.meshes.len());
    let mut old_to_new = vec![usize::MAX; model.meshes.len()];
    let mut removable = vec![false; model.materials.len()];
    for (index, mesh) in model.meshes.iter().enumerate() {
        if mesh.is_antiblur_mesh {
            removable[mesh.material] = true;
        } else {
            old_to_new[index] = kept.len();
            kept.push(mesh.clone());
        }
    }
    // A material still used by a surviving mesh is not removable.
    for mesh in &kept {
        removable[mesh.material] = false;
    }

    let mut material_map = vec![usize::MAX; model.materials.len()];
    let mut materials = Vec::with_capacity(model.materials.len());
    for (index, material) in model.materials.drain(..).enumerate() {
        if !removable[index] {
            material_map[index] = materials.len();
            materials.push(material);
        }
    }
    model.materials = materials;

    for mesh in &mut kept {
        mesh.material = material_map[mesh.material];
    }
    model.meshes = kept;
    for group in &mut model.mesh_groups {
        group.meshes.retain(|mesh| old_to_new[*mesh] != usize::MAX);
        for mesh in &mut group.meshes {
            *mesh = old_to_new[*mesh];
        }
    }
    model.extensions.antiblur = false;
}

/// The fuzzblock material for `source`: `<name> antiblur`, the uvscroll
/// variant when the source scrolls its base texture, and the source's base
/// color texture plus the dummy normal and specular maps.
fn antiblur_material(source: &MaterialInstance) -> MaterialInstance {
    let scrolls = source
        .parameters
        .iter()
        .any(|(name, _)| name == "UV0_Speed_U" || name == "UV0_Speed_V");
    let mut parameters = vec![("MatParamIndex_0".to_owned(), [0.0; 4])];
    let (shader, technique) = if scrolls {
        for (name, values) in &source.parameters {
            if name == "UV0_Speed_U" || name == "UV0_Speed_V" || name == "Offset" {
                parameters.push((name.clone(), *values));
            }
        }
        (
            "fox3ddf_blin_fuzzblock_uvscroll",
            "fox3DDF_Blin_Fuzzblock_UVScroll",
        )
    } else {
        ("fox3ddf_blin_fuzzblock", "fox3DDF_Blin_Fuzzblock")
    };

    let mut textures = Vec::new();
    let base = source
        .textures
        .iter()
        .find(|(sampler, _)| sampler == "Base_Tex_SRGB")
        .or_else(|| {
            source
                .textures
                .iter()
                .find(|(sampler, _)| sampler.to_lowercase().contains("base"))
        })
        .or_else(|| source.textures.first());
    if let Some((_, texture)) = base {
        textures.push(("Base_Tex_SRGB".to_owned(), texture.clone()));
    }
    textures.push((
        "NormalMap_Tex_NRM".to_owned(),
        Texture {
            file_name: "dummy_nrm.ftex".to_owned(),
            directory: TEXTURE_DIRECTORY.to_owned(),
        },
    ));
    textures.push((
        "SpecularMap_Tex_LIN".to_owned(),
        Texture {
            file_name: "dummy_srm.ftex".to_owned(),
            directory: TEXTURE_DIRECTORY.to_owned(),
        },
    ));

    MaterialInstance {
        name: format!("{} antiblur", source.name),
        shader: shader.to_owned(),
        technique: technique.to_owned(),
        textures,
        parameters,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::FmdlFile;

    const ORAL: &[u8] = include_bytes!("../../tests/fixtures/addon_oral.fmdl");
    const AU_LOW: &[u8] = include_bytes!("../../tests/fixtures/konami_au_Low_parts.fmdl");

    fn load(bytes: &[u8]) -> Model {
        Model::from_file(&FmdlFile::read(bytes).unwrap()).unwrap()
    }

    #[test]
    fn encode_flagged_oral() {
        let mut model = load(ORAL);
        model.meshes[0].has_antiblur_meshes = true;
        encode(&mut model);

        assert_eq!(model.meshes.len(), 2);
        assert_eq!(model.materials.len(), 2);
        let duplicate = &model.meshes[1];
        assert!(duplicate.is_antiblur_mesh);
        assert!(!duplicate.has_antiblur_meshes);
        assert_eq!(duplicate.alpha_flags, 128);
        assert_eq!(duplicate.shadow_flags, 1);
        assert_eq!(duplicate.custom_bounding_box, None);
        assert_eq!(duplicate.vertices, model.meshes[0].vertices);
        assert_eq!(duplicate.faces, model.meshes[0].faces);
        assert_eq!(duplicate.bone_group, model.meshes[0].bone_group);

        let material = &model.materials[1];
        assert_eq!(material.name, "Material antiblur");
        assert_eq!(material.shader, "fox3ddf_blin_fuzzblock");
        assert_eq!(material.technique, "fox3DDF_Blin_Fuzzblock");
        assert_eq!(
            material
                .textures
                .iter()
                .map(|(sampler, t)| (sampler.as_str(), t.directory.as_str(), t.file_name.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (
                    "Base_Tex_SRGB",
                    "/Assets/pes16/model/character/common/sourceimages/",
                    "dummy_bsm.dds"
                ),
                (
                    "NormalMap_Tex_NRM",
                    "/Assets/pes16/model/character/common/sourceimages/",
                    "dummy_nrm.ftex"
                ),
                (
                    "SpecularMap_Tex_LIN",
                    "/Assets/pes16/model/character/common/sourceimages/",
                    "dummy_srm.ftex"
                ),
            ]
        );
        assert_eq!(
            material.parameters,
            vec![("MatParamIndex_0".to_owned(), [0.0; 4])]
        );
        assert_eq!(model.mesh_groups[0].meshes, [0, 1]);
        assert!(model.extensions.antiblur);
    }

    #[test]
    fn encode_decode_round_trip() {
        // The flag the encode sets is cleared by decode, so the source model
        // starts with it unset.
        let mut oral = load(ORAL);
        oral.meshes[0].has_antiblur_meshes = true;
        oral.extensions.antiblur = false;
        let mut encoded = oral.clone();
        encode(&mut encoded);
        decode(&mut encoded);
        assert_eq!(encoded, oral);

        let mut au_low = load(AU_LOW);
        au_low.meshes[0].has_antiblur_meshes = true;
        au_low.meshes[2].has_antiblur_meshes = true;
        let mut encoded = au_low.clone();
        encode(&mut encoded);
        assert_eq!(encoded.meshes.len(), 6);
        for group in &encoded.mesh_groups {
            for mesh in &group.meshes {
                assert!(*mesh < encoded.meshes.len());
            }
        }
        decode(&mut encoded);
        assert_eq!(encoded, au_low);
    }

    #[test]
    fn encode_is_idempotent() {
        let mut model = load(ORAL);
        model.meshes[0].has_antiblur_meshes = true;
        encode(&mut model);
        let once = model.clone();
        encode(&mut model);
        assert_eq!(model, once);
    }

    #[test]
    fn uvscroll_source_gets_uvscroll_material() {
        let mut model = load(ORAL);
        model.meshes[0].has_antiblur_meshes = true;
        model.materials[0]
            .parameters
            .push(("UV0_Speed_U".to_owned(), [0.5, 0.0, 0.0, 0.0]));
        model.materials[0]
            .parameters
            .push(("Offset".to_owned(), [1.0; 4]));
        encode(&mut model);

        let material = &model.materials[1];
        assert_eq!(material.shader, "fox3ddf_blin_fuzzblock_uvscroll");
        assert_eq!(material.technique, "fox3DDF_Blin_Fuzzblock_UVScroll");
        assert_eq!(
            material.parameters,
            vec![
                ("MatParamIndex_0".to_owned(), [0.0; 4]),
                ("UV0_Speed_U".to_owned(), [0.5, 0.0, 0.0, 0.0]),
                ("Offset".to_owned(), [1.0; 4]),
            ]
        );
    }

    #[test]
    fn encoded_model_survives_a_file_round_trip() {
        let mut model = load(ORAL);
        model.meshes[0].has_antiblur_meshes = true;
        encode(&mut model);
        let again = Model::from_file(&model.to_file().unwrap()).unwrap();
        assert_eq!(again, model);
    }

    #[test]
    fn decode_keeps_a_shared_material() {
        let mut model = load(ORAL);
        model.meshes[0].has_antiblur_meshes = true;
        encode(&mut model);
        // Point the anti-blur mesh at the material its source also uses.
        model.meshes[1].material = 0;
        decode(&mut model);
        assert_eq!(model.meshes.len(), 1);
        // Material 0 was shared and stays; the orphaned fuzzblock material
        // is not one only the removed meshes used, so it stays too.
        assert_eq!(model.materials.len(), 2);
        assert_eq!(model.meshes[0].material, 0);
        assert_eq!(model.materials[0].name, "Material");
        assert!(!model.extensions.antiblur);
    }
}
