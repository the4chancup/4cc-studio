//! The hand auto-split of a pre-Fox face model (`model_conversion/hand_split.md` "Pipeline
//! integration"): a `.model` whose vertices weigh on hand-skeleton bones is split at the wrists
//! through `model_convert`'s IR, and each part written back as a `.model`. The face packs the
//! body in the model's place and the hands as two more models of its own; all three name the
//! source model's `.mtl`, since each part keeps a subset of its materials under their names.

use model_convert::formats::pes_model::{ir_to_model, model_to_ir};
use model_convert::ir::CanonicalModel;
use model_convert::ops::hand_split::split_by_skeleton_group;
use pes_model::format::PreFoxModel;
use pes_model::format::mtl::MaterialSet;
use pes_model::model::Model;
use studio_core::Disposition;

use super::{CompileContext, Finding, TaskFailure};
use crate::messages::Code;

/// A `.model` after the hand auto-split, each part written back as a `.model`.
pub(super) struct SplitModel {
    /// The model without its hands; `None` when no face is left.
    pub(super) body: Option<Vec<u8>>,
    /// The left hand, when the model weighs vertices on an `skh_*_l` bone.
    pub(super) glove_l: Option<Vec<u8>>,
    /// The right hand, when the model weighs vertices on an `skh_*_r` bone.
    pub(super) glove_r: Option<Vec<u8>>,
}

/// The face model `name` (its file name), `bytes`, split at the wrists, `mtl` being the bytes
/// of the `.mtl` its search found, and noted in `findings` as `model_hand_split` naming the
/// model and the gloves made (`glove_l`, `glove_r` or both). The split's parsed forms are
/// charged to the run's memory budget at the source's size while they live. A model or `.mtl`
/// the split cannot read or write fails the task with `model_conversion_failed`, naming the
/// model.
pub(super) fn split_face_model(
    name: &str,
    bytes: &[u8],
    mtl: &[u8],
    ctx: &CompileContext,
    findings: &mut Vec<Finding>,
) -> Result<SplitModel, TaskFailure> {
    let split = {
        // The split's model, its IR and its written parts, charged at the source's size, an
        // estimate of each form, while they are built.
        let _split_charge = ctx.budget.charge(bytes.len());
        split_model(bytes, mtl).map_err(|error| TaskFailure {
            code: Code::ModelConversionFailed,
            context: vec![("model", name.to_owned()), ("error", format!("{error:#}"))],
        })?
    };
    let made: Vec<&str> = [("glove_l", &split.glove_l), ("glove_r", &split.glove_r)]
        .into_iter()
        .filter(|(_, part)| part.is_some())
        .map(|(hand, _)| hand)
        .collect();
    findings.push((
        Code::ModelHandSplit,
        Disposition::Keep,
        vec![("model", name.to_owned()), ("gloves", made.join(", "))],
    ));
    Ok(split)
}

/// The `.model` `bytes` split at the wrists (`model_convert`'s `split_by_skeleton_group`),
/// through `model_convert`'s IR and back, `mtl` defining its materials.
fn split_model(bytes: &[u8], mtl: &[u8]) -> anyhow::Result<SplitModel> {
    let model = Model::from_file(&PreFoxModel::read(bytes)?)?;
    let set = MaterialSet::read(mtl)?;
    // The import's and the export's findings, `model_convert`'s loss codes, are left
    // unreported on purpose, as the Fox split leaves them: cross-format conversion, a later
    // step, maps those codes for every converted model.
    let ir = model_to_ir(&model, &set)?.model;
    let split = split_by_skeleton_group(&ir);
    // Each part's exported `.mtl` is not packed: every material a part keeps is the source's,
    // under its name, so the source `.mtl` the three entries name defines it.
    let write = |part: &CanonicalModel| -> anyhow::Result<Vec<u8>> {
        Ok(ir_to_model(part)?.model.to_file()?.write()?)
    };
    let body = if split.body.meshes.is_empty() {
        None
    } else {
        Some(write(&split.body)?)
    };
    Ok(SplitModel {
        body,
        glove_l: split.glove_l.as_ref().map(write).transpose()?,
        glove_r: split.glove_r.as_ref().map(write).transpose()?,
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use pes_version::PesVersion;
    use pipeline::MemoryBudget;

    use super::*;
    use crate::bins::installed::InstalledPaths;
    use crate::processing::EntryTarget;
    use crate::templates::Templates;

    /// The bytes of `tests/fixtures/hand_split/<name>`: the full-body strip as a pre-Fox pair
    /// (`tests/fixtures/hand_split/README.md`).
    fn fixture(name: &str) -> Vec<u8> {
        std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/hand_split")
                .join(name),
        )
        .unwrap()
    }

    fn parsed(bytes: &[u8]) -> Model {
        Model::from_file(&PreFoxModel::read(bytes).unwrap()).unwrap()
    }

    /// Every face of `model` as its three corner positions, rotated so the smallest comes
    /// first (the winding kept), the whole list sorted.
    fn faces(model: &Model) -> Vec<[[f32; 3]; 3]> {
        let mut out: Vec<[[f32; 3]; 3]> = model
            .meshes
            .iter()
            .flat_map(|mesh| {
                mesh.faces.iter().map(|face| {
                    let corners = face.map(|i| mesh.vertices.positions[usize::from(i)]);
                    let start = (0..3)
                        .min_by(|&a, &b| corners[a].partial_cmp(&corners[b]).unwrap())
                        .unwrap();
                    [
                        corners[start],
                        corners[(start + 1) % 3],
                        corners[(start + 2) % 3],
                    ]
                })
            })
            .collect();
        out.sort_by(|a, b| a.partial_cmp(b).unwrap());
        out
    }

    fn vertex_count(model: &Model) -> usize {
        model
            .meshes
            .iter()
            .map(|mesh| mesh.vertices.positions.len())
            .sum()
    }

    #[test]
    fn the_fixture_splits_into_a_body_and_two_gloves_naming_the_source_s_materials() {
        let source = fixture("body.model");
        let split = split_model(&source, &fixture("body.mtl")).unwrap();
        let body = parsed(&split.body.unwrap());
        let glove_l = parsed(&split.glove_l.unwrap());
        let glove_r = parsed(&split.glove_r.unwrap());

        let counts = |model: &Model| (vertex_count(model), faces(model).len());
        assert_eq!(counts(&body), (21, 24));
        assert_eq!(counts(&glove_l), (9, 8));
        assert_eq!(counts(&glove_r), (9, 8));
        // Every face of the source is in exactly one part.
        let mut parts: Vec<[[f32; 3]; 3]> = [&body, &glove_l, &glove_r]
            .into_iter()
            .flat_map(faces)
            .collect();
        parts.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(parts, faces(&parsed(&source)));
        // The source `.mtl` defines every material a part names.
        let source_materials = parsed(&source).materials;
        for part in [&body, &glove_l, &glove_r] {
            assert!(!part.materials.is_empty());
            for material in &part.materials {
                assert!(source_materials.contains(material), "{material}");
            }
        }
    }

    #[test]
    fn a_model_that_does_not_read_fails_the_task_naming_it() {
        let ctx = CompileContext::new(
            PesVersion::Pes17,
            1,
            Templates::embedded(),
            InstalledPaths::Unknown,
            EntryTarget::GamePaths {
                engine: PesVersion::Pes17.engine(),
            },
            MemoryBudget::new(usize::MAX),
            false,
        );
        let mut findings = Vec::new();
        let Err(failure) = split_face_model(
            "body.model",
            b"not a model",
            &fixture("body.mtl"),
            &ctx,
            &mut findings,
        ) else {
            panic!("a model that does not read cannot be split");
        };
        let error = PreFoxModel::read(b"not a model").unwrap_err().to_string();
        assert_eq!(failure.code, Code::ModelConversionFailed);
        assert_eq!(
            failure.context,
            [("model", "body.model".to_owned()), ("error", error)]
        );
        assert_eq!(findings, []);
    }
}
