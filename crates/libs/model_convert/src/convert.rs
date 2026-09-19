//! Conversion routing: `convert` takes a PES *version*, not a format — the format follows
//! from the engine, and the skeleton retargeting pass runs inside the same IR round trip,
//! so every model the compiler emits is on the target's skeleton without a second pass.
//! `needs_conversion` is the native pre-check: it reads bone names and poses only, never
//! geometry.

use std::collections::BTreeSet;

use pes_version::{Engine, PesVersion};

use crate::affine::Affine;
use crate::formats::{ConvertError, Imported};
use crate::loss::Finding;
use crate::skeletons::retarget::{bone_moved, retarget};
use crate::skeletons::{is_standard, skeletons, version_bone};

/// A model in one of the native formats, at the semantic layer of its format crate.
pub enum NativeModelBundle {
    /// FMDL plus its companion `.skl` when one exists (PES 2018–2021).
    Fox {
        /// The FMDL model.
        model: ::fmdl::Model,
        /// The companion skeleton, where the bind pose comes from when present.
        skl: Option<::fmdl::SklFile>,
    },
    /// `.model` plus its `.mtl` (PES 2015–2017).
    PreFox {
        /// The `.model`.
        model: ::pes_model::model::Model,
        /// The `.mtl`.
        mtl: ::pes_model::format::mtl::MaterialSet,
    },
    // Gltf(GltfModel) joins in Phase 7
}

/// `convert`'s result: the converted bundle and every finding the round trip produced.
pub struct Converted {
    /// The bundle in the target's format.
    pub bundle: NativeModelBundle,
    /// What the target could not keep, import → retarget → export in order.
    pub findings: Vec<Finding>,
}

/// The bundle in `target`'s format (`target.engine()`), on `target`'s skeleton: the input
/// itself when it already is both, otherwise source → IR → `retarget` → target.
/// `Converted::findings` lists what the target could not keep (`loss.rs`), so the caller
/// reports rather than the user discovers.
pub fn convert(bundle: NativeModelBundle, target: PesVersion) -> Result<Converted, ConvertError> {
    if !needs_conversion(&bundle, target) {
        return Ok(Converted {
            bundle,
            findings: vec![],
        });
    }
    let Imported {
        model,
        mut findings,
    } = match &bundle {
        NativeModelBundle::Fox { model, skl } => {
            crate::formats::fmdl::fmdl_to_ir(model, skl.as_ref())?
        }
        NativeModelBundle::PreFox { model, mtl } => {
            crate::formats::pes_model::model_to_ir(model, mtl)?
        }
    };
    let mut ir = model;
    findings.extend(retarget(&mut ir, target)?);
    let bundle = match target.engine() {
        Engine::Fox => {
            let exported = crate::formats::fmdl::ir_to_fmdl(&ir)?;
            findings.extend(exported.findings);
            NativeModelBundle::Fox {
                model: exported.model,
                skl: exported.skl,
            }
        }
        Engine::PreFox => {
            let exported = crate::formats::pes_model::ir_to_model(&ir)?;
            findings.extend(exported.findings);
            NativeModelBundle::PreFox {
                model: exported.model,
                mtl: exported.mtl,
            }
        }
    };
    Ok(Converted { bundle, findings })
}

/// Whether `convert` would go through the IR: another engine, or a bone the bundle's groups
/// use that `target` lacks or poses differently (bone tables only, no geometry read).
/// A group entry past the bone table counts as needing conversion: such a bundle cannot be
/// proven native here, and `convert`'s importer reports the bad index.
pub fn needs_conversion(bundle: &NativeModelBundle, target: PesVersion) -> bool {
    match bundle {
        NativeModelBundle::Fox { model, skl } => {
            if target.engine() != Engine::Fox {
                return true;
            }
            let tables = skeletons(target);
            let pes21 = skeletons(PesVersion::Pes21);
            for index in used_bones(model.meshes.iter().map(|m| &m.bone_group)) {
                let Some(bone) = model.bones.get(index) else {
                    return true;
                };
                if !is_standard(&bone.name) {
                    continue;
                }
                let Some(table_bone) = version_bone(tables, &bone.name) else {
                    return true;
                };
                // The source pose: the SKL bone of that name, else PES21's tables; an
                // unknown pose cannot be compared, so the bone is skipped.
                let Some(source) = skl
                    .as_ref()
                    .and_then(|skl| skl.bones.iter().find(|b| b.name == bone.name))
                    .map(|b| Affine::from_rotation_translation(b.rotation, b.translation))
                    .or_else(|| version_bone(pes21, &bone.name).map(|b| b.matrix))
                else {
                    continue;
                };
                if bone_moved(&source, &table_bone.matrix) != Some(false) {
                    return true;
                }
            }
            false
        }
        NativeModelBundle::PreFox { model, .. } => {
            if target.engine() != Engine::PreFox {
                return true;
            }
            let tables = skeletons(target);
            for index in used_bones(model.meshes.iter().map(|m| &m.bone_group)) {
                let Some(bone) = model.bones.get(index) else {
                    return true;
                };
                if !is_standard(&bone.name) {
                    continue;
                }
                let Some(table_bone) = version_bone(tables, &bone.name) else {
                    return true;
                };
                // The source pose is the inline inverse bind matrix, inverted; a singular
                // one counts as needing conversion.
                let Some(source) = Affine(bone.matrix).inverse() else {
                    return true;
                };
                if bone_moved(&source, &table_bone.matrix) != Some(false) {
                    return true;
                }
            }
            false
        }
    }
}

/// The bone indices any mesh's bone group references.
fn used_bones<'a>(groups: impl Iterator<Item = &'a Vec<usize>>) -> BTreeSet<usize> {
    groups.flatten().copied().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{CanonicalModel, Material, Mesh, MeshGroup, SourceFormat, Vertices};
    use crate::loss::Subject;
    use crate::materials::MaterialFamily;

    const HIGHNECK: &[u8] = include_bytes!("../tests/fixtures/konami_highneck.fmdl");
    const ORAL: &[u8] = include_bytes!("../tests/fixtures/addon_oral.fmdl");
    const CAP_M: &[u8] = include_bytes!("../tests/fixtures/konami_modD_cap.model");
    const CAP_T: &[u8] = include_bytes!("../tests/fixtures/konami_modD_cap.mtl");
    const LEGACY_M: &[u8] = include_bytes!("../tests/fixtures/legacy19to16_oral.model");
    const LEGACY_T: &[u8] = include_bytes!("../tests/fixtures/legacy19to16_oral.mtl");

    fn fox(bytes: &[u8]) -> NativeModelBundle {
        NativeModelBundle::Fox {
            model: ::fmdl::Model::from_file(&::fmdl::FmdlFile::read(bytes).expect("fmdl"))
                .expect("model"),
            skl: None,
        }
    }

    fn prefox(model: &[u8], mtl: &[u8]) -> NativeModelBundle {
        NativeModelBundle::PreFox {
            model: ::pes_model::model::Model::from_file(
                &::pes_model::format::PreFoxModel::read(model).expect("model"),
            )
            .expect("from_file"),
            mtl: ::pes_model::format::mtl::MaterialSet::read(mtl).expect("mtl"),
        }
    }

    /// `(code, subject, detail)` triples of `findings`.
    fn codes(findings: &[Finding]) -> Vec<(&'static str, Subject, &str)> {
        findings
            .iter()
            .map(|f| (f.code, f.subject.clone(), f.detail.as_str()))
            .collect()
    }

    #[test]
    fn same_version_same_engine_is_the_input() {
        let NativeModelBundle::Fox { model: fmdl, .. } = fox(HIGHNECK) else {
            unreachable!()
        };
        assert!(!needs_conversion(&fox(HIGHNECK), PesVersion::Pes21));
        let converted = convert(fox(HIGHNECK), PesVersion::Pes21).expect("convert");
        let NativeModelBundle::Fox { model, .. } = converted.bundle else {
            panic!("expected a Fox bundle");
        };
        assert_eq!(model, fmdl);
        assert_eq!(converted.findings, vec![]);

        let NativeModelBundle::PreFox { model: cap, .. } = prefox(CAP_M, CAP_T) else {
            unreachable!()
        };
        assert!(!needs_conversion(&prefox(CAP_M, CAP_T), PesVersion::Pes17));
        let converted = convert(prefox(CAP_M, CAP_T), PesVersion::Pes17).expect("convert");
        let NativeModelBundle::PreFox { model, .. } = converted.bundle else {
            panic!("expected a pre-Fox bundle");
        };
        assert_eq!(model, cap);
        assert_eq!(converted.findings, vec![]);
    }

    #[test]
    fn same_engine_other_version_checks_the_tables() {
        // Highneck's seven bones all exist in PES18 within tolerance.
        assert!(!needs_conversion(&fox(HIGHNECK), PesVersion::Pes18));
        // The cap's `dsk_deltoid_l`/`dsk_upperarm_long_l` are absent from PES15.
        assert!(needs_conversion(&prefox(CAP_M, CAP_T), PesVersion::Pes15));
    }

    #[test]
    fn prefox_to_fox_goes_through_the_ir() {
        let converted = convert(prefox(CAP_M, CAP_T), PesVersion::Pes21).expect("convert");
        let NativeModelBundle::Fox { model, skl } = converted.bundle else {
            panic!("expected a Fox bundle");
        };
        assert_eq!(
            model
                .bones
                .iter()
                .map(|b| b.name.as_str())
                .collect::<Vec<_>>(),
            [
                "sk_shoulder_l",
                "sk_upperarm_l",
                "dsk_deltoid_l",
                "dsk_upperarm_long_l",
            ]
        );
        assert!(skl.is_none(), "every bone is in PES21's tables");
        assert_eq!(
            codes(&converted.findings),
            [
                ("native_field_dropped", Subject::Mesh(0), "tags"),
                ("skeleton_retargeted", Subject::Model, "1"),
                (
                    "dummy_texture_added",
                    Subject::Material(0),
                    "NormalMap_Tex_NRM"
                ),
                (
                    "dummy_texture_added",
                    Subject::Material(0),
                    "SpecularMap_Tex_LIN"
                ),
                ("vertex_bitangents_dropped", Subject::Mesh(0), ""),
            ]
        );
    }

    /// The (position, normal xyz, uv0) multiset comparison the brief specifies; every
    /// component must match within `eps`.
    fn vertex_multiset(
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        uvs: &[[f32; 2]],
    ) -> Vec<([f32; 3], [f32; 3], [f32; 2])> {
        let mut rows: Vec<_> = positions
            .iter()
            .zip(normals)
            .zip(uvs)
            .map(|((p, n), uv)| (*p, *n, *uv))
            .collect();
        rows.sort_by(|a, b| {
            (a.0, a.1, a.2)
                .partial_cmp(&(b.0, b.1, b.2))
                .expect("no NaN")
        });
        rows
    }

    /// Faces as position triples rotated smallest-first, then sorted.
    fn face_set(positions: &[[f32; 3]], faces: &[[u16; 3]]) -> Vec<[[f32; 3]; 3]> {
        let mut out: Vec<[[f32; 3]; 3]> = faces
            .iter()
            .map(|face| {
                let pts = face.map(|i| positions[usize::from(i)]);
                let start = (0..3)
                    .min_by(|&a, &b| pts[a].partial_cmp(&pts[b]).expect("no NaN"))
                    .expect("three corners");
                [pts[start], pts[(start + 1) % 3], pts[(start + 2) % 3]]
            })
            .collect();
        out.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
        out
    }

    #[test]
    fn fox_oral_to_pes16_matches_the_legacy_reference() {
        let converted = convert(fox(ORAL), PesVersion::Pes16).expect("convert");
        let NativeModelBundle::PreFox { model, mtl } = converted.bundle else {
            panic!("expected a pre-Fox bundle");
        };
        let legacy = ::pes_model::model::Model::from_file(
            &::pes_model::format::PreFoxModel::read(LEGACY_M).expect("legacy model"),
        )
        .expect("legacy from_file");
        let legacy_mtl = ::pes_model::format::mtl::MaterialSet::read(LEGACY_T).expect("legacy mtl");

        assert_eq!(
            model
                .bones
                .iter()
                .map(|b| b.name.as_str())
                .collect::<Vec<_>>(),
            ["sk_head"]
        );
        let delta =
            Affine(model.bones[0].matrix).max_component_delta(&Affine(legacy.bones[0].matrix));
        assert!(delta < 1e-5, "sk_head matrix delta {delta}");

        assert_eq!(model.meshes.len(), 1);
        let (ours, theirs) = (&model.meshes[0], &legacy.meshes[0]);
        assert_eq!(
            ours.vertices.positions.len(),
            theirs.vertices.positions.len()
        );
        assert_eq!(
            face_set(&ours.vertices.positions, &ours.faces),
            face_set(&theirs.vertices.positions, &theirs.faces)
        );
        let our_normals: Vec<[f32; 3]> = ours
            .vertices
            .normals
            .as_ref()
            .expect("normals")
            .iter()
            .map(|n| [n[0], n[1], n[2]])
            .collect();
        let a = vertex_multiset(
            &ours.vertices.positions,
            &our_normals,
            &ours.vertices.uvs[0],
        );
        let b = vertex_multiset(
            &theirs.vertices.positions,
            theirs.vertices.normals.as_ref().expect("normals"),
            &theirs.vertices.uvs[0],
        );
        assert_eq!(a.len(), b.len());
        for ((pa, na, ua), (pb, nb, ub)) in a.iter().zip(&b) {
            for (x, y) in pa
                .iter()
                .chain(na)
                .chain(ua)
                .zip(pb.iter().chain(nb).chain(ub))
            {
                assert!((x - y).abs() < 1e-6, "{x} vs {y}");
            }
        }

        // The material: the `Basic_*` ladder follows the roles present — `Basic_CNS`
        // because the FMDL's Normal/Specular maps are kept (the legacy wrote `Basic_C`
        // because its run had no texture files to resolve; tests/fixtures/README.md). The
        // seven states and DiffuseMap's three attributes equal the legacy's; the path is
        // excluded by design.
        let mat = mtl.materials.first().expect("a material");
        let legacy_mat = legacy_mtl.materials.first().expect("a legacy material");
        assert_eq!(mat.shader, "Basic_CNS");
        fn sampler<'a>(
            m: &'a ::pes_model::format::mtl::Material,
            name: &str,
        ) -> Option<&'a ::pes_model::format::mtl::Sampler> {
            m.entries.iter().find_map(|e| match e {
                ::pes_model::format::mtl::MaterialEntry::Sampler(s) if s.name == name => Some(s),
                _ => None,
            })
        }
        for (name, file) in [
            ("DiffuseMap", "dummy_bsm.dds"),
            ("NormalMap", "dummy_nrm.dds"),
            ("SpecularMap", "dummy_srm.dds"),
        ] {
            let s = sampler(mat, name).expect(name);
            assert!(s.path.ends_with(file), "{name}: {}", s.path);
        }
        let states = |m: &::pes_model::format::mtl::Material| -> BTreeSet<(String, u32)> {
            m.entries
                .iter()
                .filter_map(|e| match e {
                    ::pes_model::format::mtl::MaterialEntry::State(s) => {
                        Some((s.name.clone(), s.value))
                    }
                    _ => None,
                })
                .collect()
        };
        assert_eq!(states(mat), states(legacy_mat));
        let (s, ls) = (
            sampler(mat, "DiffuseMap").expect("DiffuseMap"),
            sampler(legacy_mat, "DiffuseMap").expect("DiffuseMap"),
        );
        assert_eq!(s.srgb, ls.srgb);
        assert_eq!(s.minfilter, ls.minfilter);
        assert_eq!(s.magfilter, ls.magfilter);
    }

    #[test]
    fn unskinned_mesh_gets_the_static_bone() {
        let ir = CanonicalModel {
            bones: vec![],
            meshes: vec![Mesh {
                vertices: Vertices {
                    positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                    ..Vertices::default()
                },
                faces: vec![[0, 1, 2]],
                bone_group: vec![],
                material: 0,
                extension_headers: Default::default(),
                custom_bounding_box: None,
            }],
            mesh_groups: vec![MeshGroup {
                name: "group".to_string(),
                parent: None,
                meshes: vec![0],
                visible: true,
            }],
            materials: vec![Material {
                name: "mat".to_string(),
                family: MaterialFamily::Shaded,
                two_sided: None,
                transparent: None,
                antiblur: None,
                textures: vec![],
                parameters: vec![],
                fox: None,
                prefox: None,
            }],
            textures: vec![],
            extension_headers: Default::default(),
            source_format: SourceFormat::PreFox,
        };
        let exported = crate::formats::fmdl::ir_to_fmdl(&ir).expect("export");
        assert_eq!(
            exported
                .model
                .bones
                .iter()
                .map(|b| b.name.as_str())
                .collect::<Vec<_>>(),
            ["static"]
        );
        assert_eq!(exported.model.bones[0].world_position, [0.2, 0.0, 0.0, 1.0]);
        assert_eq!(exported.model.bones[0].local_position, [0.0; 4]);
        assert_eq!(exported.model.meshes[0].bone_group, vec![0]);
        assert_eq!(
            exported.model.meshes[0].vertices.bone_weights,
            Some(vec![[255, 0, 0, 0]; 3])
        );
        let skl = exported.skl.expect("an SKL for the unknown bone");
        assert_eq!(skl.bones.len(), 1);
        assert_eq!(skl.bones[0].name, "static");
        assert_eq!(
            skl.bones[0].rotation,
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
        );
        assert_eq!(skl.bones[0].translation, [0.0; 3]);
        assert_eq!(
            codes(&exported.findings),
            vec![
                (
                    "dummy_texture_added",
                    Subject::Material(0),
                    "NormalMap_Tex_NRM"
                ),
                (
                    "dummy_texture_added",
                    Subject::Material(0),
                    "SpecularMap_Tex_LIN"
                ),
                ("static_bone_added", Subject::Mesh(0), ""),
            ]
        );

        // The SKL passed back gives the bind pose; nothing is unknown.
        let Imported { model, findings } =
            crate::formats::fmdl::fmdl_to_ir(&exported.model, Some(&skl)).expect("import");
        assert_eq!(findings, vec![]);
        let mesh = &model.meshes[0];
        assert_eq!(model.bones[0].name, "static");
        assert_eq!(mesh.bone_group, vec![0]);
        assert_eq!(
            mesh.vertices.bone_weights,
            Some(vec![[1.0, 0.0, 0.0, 0.0]; 3])
        );
    }

    #[test]
    fn an_undefined_material_is_an_error_not_a_bundle() {
        let NativeModelBundle::PreFox { mut model, mtl } = prefox(CAP_M, CAP_T) else {
            unreachable!()
        };
        model.materials = vec!["nope".to_string()];
        assert!(matches!(
            convert(
                NativeModelBundle::PreFox { model, mtl },
                PesVersion::Pes15
            ),
            Err(ConvertError::MaterialUndefined(name)) if name == "nope"
        ));
    }

    #[test]
    fn a_bone_group_entry_past_the_bone_table_needs_conversion() {
        let NativeModelBundle::Fox { mut model, skl } = fox(ORAL) else {
            unreachable!()
        };
        model.meshes[0].bone_group = vec![9];
        assert!(needs_conversion(
            &NativeModelBundle::Fox { model, skl },
            PesVersion::Pes21
        ));

        let NativeModelBundle::PreFox { mut model, mtl } = prefox(CAP_M, CAP_T) else {
            unreachable!()
        };
        model.meshes[0].bone_group = vec![9];
        assert!(needs_conversion(
            &NativeModelBundle::PreFox { model, mtl },
            PesVersion::Pes17
        ));
    }
}
