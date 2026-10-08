//! A selected model of the other engine's format converted for the target
//! (`team_compiler/pipeline.md` step 3 "Format conversion") through `model_convert::convert`,
//! which also moves it onto the target version's skeleton: on PES 15-17 an `.fmdl`, with its
//! paired `.skl` as the bind pose, written as a `.model` and its material set; on PES 18-21 a
//! `.model`, with the `.mtl` its search finds, written as an FMDL and, when the conversion makes
//! one, a skeleton. Every converted model is checked in its target form before it is written
//! (`messages.md` `vertex_too_far_from_origin`): what the source passed, the conversion may
//! still have moved or split past a limit.

use model_convert::{Converted, NativeModelBundle, Subject, convert, loss};
use pes_model::format::PreFoxModel;
use pes_model::format::mtl::MaterialSet;
use pes_version::PesVersion;
use studio_core::Disposition;

use super::{CompileContext, Finding, TaskFailure};
use crate::deep::{FAR_VERTEX_CODES, Fired, summed};
use crate::messages::Code;

/// An FMDL converted for a PES 15-17 target.
pub(super) struct PreFoxConversion {
    /// The `.model` written, packed in the FMDL's place.
    pub(super) model: Vec<u8>,
    /// Its material set, packed as `<stem>.mtl` beside it.
    pub(super) materials: MaterialSet,
}

/// A `.model` converted for a PES 18-21 target.
pub(super) struct FoxConversion {
    /// The FMDL written, a part in the `.model`'s place.
    pub(super) model: Vec<u8>,
    /// The skeleton the conversion writes when a bone the model keeps is outside the game's
    /// skeleton tables (`ExportedFox::skl`): the part's skeleton, as a member's `.skl` of the
    /// model's stem would be. `None` when every bone is the game's own.
    pub(super) skeleton: Option<Vec<u8>>,
}

/// The FMDL `name` (its file name), `bytes`, converted for `ctx.version`, a PES 15-17 target,
/// `skeleton` being the bytes of its paired `.skl`, its bind pose, when its folder holds one.
/// The conversion's parsed forms are charged to the run's memory budget at the source's size
/// while they live. What the conversion reports that the member is told is noted in
/// `findings` naming the model (`reported`). A model or skeleton the conversion cannot read,
/// convert or write fails the task with `model_conversion_failed`, naming the model, and so
/// does a `.model` written that `pes_model`'s check finds an Error in (`target_form_failure`).
pub(super) fn fmdl_for_pre_fox(
    name: &str,
    bytes: &[u8],
    skeleton: Option<&[u8]>,
    ctx: &CompileContext,
    findings: &mut Vec<Finding>,
) -> Result<PreFoxConversion, TaskFailure> {
    let (converted, losses) = {
        // The FMDL, its IR and the `.model` written, charged at the source's size, an estimate
        // of each form, while they are built.
        let _conversion_charge = ctx.budget.charge(bytes.len());
        let (model, materials, losses) =
            converted_fmdl(bytes, skeleton, ctx.version).map_err(|error| failed(name, error))?;
        let fired = pes_model::check::check(&model)
            .into_iter()
            .map(Fired::pre_fox);
        target_form_failure(name, fired.collect())?;
        let written = model.to_file().and_then(|file| file.write());
        let model = written.map_err(|error| failed(name, error.into()))?;
        (PreFoxConversion { model, materials }, losses)
    };
    findings.extend(losses.iter().filter_map(|loss| reported(name, loss)));
    Ok(converted)
}

/// The FMDL `bytes` converted for the PES 15-17 `version`, with the skeleton `skeleton` as its
/// bind pose: the `.model`, its material set and the conversion's loss findings.
fn converted_fmdl(
    bytes: &[u8],
    skeleton: Option<&[u8]>,
    version: PesVersion,
) -> anyhow::Result<(pes_model::model::Model, MaterialSet, Vec<loss::Finding>)> {
    let model = fmdl::Model::from_file(&fmdl::FmdlFile::read(bytes)?)?;
    let skl = skeleton.map(fmdl::SklFile::read).transpose()?;
    let Converted { bundle, findings } = convert(NativeModelBundle::Fox { model, skl }, version)?;
    match bundle {
        NativeModelBundle::PreFox { model, mtl } => Ok((model, mtl, findings)),
        NativeModelBundle::Fox { .. } => {
            unreachable!("`convert` returns the target's format, a `.model` for PES 15-17")
        }
    }
}

/// The `.model` `name` (its file name), `bytes`, converted for `ctx.version`, a PES 18-21
/// target, `mtl` being the bytes of the `.mtl` its search found (`mtl_search::mtl_for`), which
/// defines its materials. Charged, reported and failed as `fmdl_for_pre_fox` is, the FMDL
/// written checked by `fmdl`'s check.
pub(super) fn model_for_fox(
    name: &str,
    bytes: &[u8],
    mtl: &[u8],
    ctx: &CompileContext,
    findings: &mut Vec<Finding>,
) -> Result<FoxConversion, TaskFailure> {
    let (converted, losses) = {
        // The `.model`, its IR and the FMDL written, charged at the source's size, an estimate
        // of each form, while they are built.
        let _conversion_charge = ctx.budget.charge(bytes.len());
        let (model, skeleton, losses) =
            converted_model(bytes, mtl, ctx.version).map_err(|error| failed(name, error))?;
        let fired = fmdl::check::check(&model).into_iter().map(Fired::fox);
        target_form_failure(name, fired.collect())?;
        let file = model
            .to_file()
            .map_err(|error| failed(name, error.into()))?;
        let converted = FoxConversion {
            model: file.write(),
            skeleton: skeleton.map(|skeleton| skeleton.write()),
        };
        (converted, losses)
    };
    findings.extend(losses.iter().filter_map(|loss| reported(name, loss)));
    Ok(converted)
}

/// The `.model` `bytes` converted for the PES 18-21 `version`, `mtl` defining its materials:
/// the FMDL, the skeleton the conversion writes when it writes one, and the conversion's loss
/// findings.
fn converted_model(
    bytes: &[u8],
    mtl: &[u8],
    version: PesVersion,
) -> anyhow::Result<(fmdl::Model, Option<fmdl::SklFile>, Vec<loss::Finding>)> {
    let model = pes_model::model::Model::from_file(&PreFoxModel::read(bytes)?)?;
    let mtl = MaterialSet::read(mtl)?;
    let Converted { bundle, findings } =
        convert(NativeModelBundle::PreFox { model, mtl }, version)?;
    match bundle {
        NativeModelBundle::Fox { model, skl } => Ok((model, skl, findings)),
        NativeModelBundle::PreFox { .. } => {
            unreachable!("`convert` returns the target's format, an FMDL for PES 18-21")
        }
    }
}

/// `model_conversion_failed` for the model `name` (its file name), carrying `error`'s chain.
fn failed(name: &str, error: anyhow::Error) -> TaskFailure {
    TaskFailure {
        code: Code::ModelConversionFailed,
        context: vec![("model", name.to_owned()), ("error", format!("{error:#}"))],
    }
}

/// The task failure the converted model `name` (its source's file name) is, `fired` being the
/// rules its format crate's check fired on it in its target form (`messages.md` "Model
/// checks"); `Ok` when none of them is an Error. A far vertex is
/// `vertex_too_far_from_origin`, naming the model and how many vertices are far, summed over
/// the model as the deep pass sums a source's (`deep::summed`); any other Error is
/// `model_conversion_failed` with the rule's code as the error: the source passed that check,
/// so the conversion made the model the game cannot load. A Warning or an Info is not
/// reported. Either failure leaves the model's package out, `pass_through` or not, as every
/// task failure does (`messages.md` `model_conversion_failed`): a failed task commits nothing,
/// and the folder's other packages and its textures stand.
fn target_form_failure(name: &str, fired: Vec<Fired>) -> Result<(), TaskFailure> {
    let errors: Vec<Fired> = summed(fired)
        .into_iter()
        .filter(|rule| rule.error)
        .collect();
    // The far vertex first: it names what the member can fix in the source.
    if let Some(far) = errors
        .iter()
        .find(|rule| FAR_VERTEX_CODES.contains(&rule.code))
    {
        return Err(TaskFailure {
            code: Code::VertexTooFarFromOrigin,
            context: vec![("model", name.to_owned()), ("count", far.count.to_string())],
        });
    }
    match errors.first() {
        Some(rule) => Err(TaskFailure {
            code: Code::ModelConversionFailed,
            context: vec![("model", name.to_owned()), ("error", rule.code.to_owned())],
        }),
        None => Ok(()),
    }
}

/// The finding the conversion's `loss` of the model `name` (its file name) is reported as, on
/// the folder, at its catalog row's severity (`messages.md`, the conversion rows): its context
/// is the model, then the index of the mesh, material or bone the loss is about, then the
/// loss's detail under the row's key when the row names one. A fold's detail,
/// `<bone> -> <target>`, names the bone itself and is given in the index's place.
/// `native_field_dropped` of a Fox mesh flag a `.mtl` cannot express (`invisible`,
/// `no_shadow_cast`) is `mesh_flags_dropped`, a Warning: the mesh shows where the source hid
/// it. `None` for a code `model_convert::loss` does not document.
fn reported(name: &str, loss: &loss::Finding) -> Option<Finding> {
    let (code, detail_key) = match loss.code {
        "bone_matrix_unknown" => (Code::BoneMatrixUnknown, Some("name")),
        "bone_slot_dropped" => (Code::BoneSlotDropped, Some("count")),
        "material_family_approximated" => (Code::MaterialFamilyApproximated, Some("name")),
        "material_split_by_flags" => (Code::MaterialSplitByFlags, Some("name")),
        "material_texture_unused" => (Code::MaterialTextureUnused, Some("texture")),
        "material_parameter_dropped" => (Code::MaterialParameterDropped, Some("parameter")),
        "sampler_settings_defaulted" => (Code::SamplerSettingsDefaulted, Some("sampler")),
        "vertex_bitangents_dropped" => (Code::VertexBitangentsDropped, None),
        "dummy_texture_added" => (Code::DummyTextureAdded, Some("sampler")),
        "native_field_dropped" => match loss.detail.as_str() {
            "invisible" | "no_shadow_cast" => (Code::MeshFlagsDropped, Some("field")),
            _ => (Code::NativeFieldDropped, Some("field")),
        },
        "bone_folded_for_version" => (Code::BoneFoldedForVersion, Some("bone")),
        "bone_folded_by_position" => (Code::BoneFoldedByPosition, Some("bone")),
        "skeleton_retargeted" => (Code::SkeletonRetargeted, Some("bones")),
        "static_bone_added" => (Code::StaticBoneAdded, None),
        "weight_clamped" => (Code::WeightClamped, Some("count")),
        "empty_mesh_bone_group_dropped" => (Code::EmptyMeshBoneGroupDropped, Some("count")),
        // `loss.code` is a string `model_convert` documents rather than an enum: a code it
        // adds is unknown here until the catalog gives it a row.
        _ => return None,
    };
    let mut context = vec![("model", name.to_owned())];
    let subject = match loss.subject {
        Subject::Model => None,
        Subject::Mesh(index) => Some(("mesh", index)),
        Subject::Material(index) => Some(("material", index)),
        Subject::Bone(index) => Some(("bone", index)),
    };
    if let Some((key, index)) = subject
        && detail_key != Some(key)
    {
        context.push((key, index.to_string()));
    }
    if let Some(key) = detail_key {
        context.push((key, loss.detail.clone()));
    }
    Some((code, Disposition::Keep, context))
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use pipeline::MemoryBudget;

    use super::*;
    use crate::bins::installed::InstalledPaths;
    use crate::processing::EntryTarget;
    use crate::templates::Templates;

    /// A run's context for `version`, with nothing installed and no memory limit.
    fn context(version: PesVersion) -> CompileContext {
        CompileContext::new(
            version,
            1,
            Templates::embedded(),
            InstalledPaths::Unknown,
            EntryTarget::GamePaths {
                engine: version.engine(),
            },
            MemoryBudget::new(usize::MAX),
            false,
        )
    }

    /// The bytes of `pes_model`'s fixture `name`: real pre-Fox models and material sets.
    fn pre_fox_fixture(name: &str) -> Vec<u8> {
        std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../libs/pes_model/tests/fixtures")
                .join(name),
        )
        .unwrap()
    }

    /// Konami's referee card, a clean one-bone `.model` whose material `konami_card_red.mtl`
    /// defines, with `edit` applied.
    fn card(edit: impl FnOnce(&mut pes_model::model::Model)) -> Vec<u8> {
        let file = PreFoxModel::read(&pre_fox_fixture("konami_card.model")).unwrap();
        let mut model = pes_model::model::Model::from_file(&file).unwrap();
        edit(&mut model);
        model.to_file().unwrap().write().unwrap()
    }

    /// The failure's code and context, or `None` when the conversion succeeded.
    fn failure<T>(result: Result<T, TaskFailure>) -> Option<(Code, Vec<(&'static str, String)>)> {
        result.err().map(|failure| (failure.code, failure.context))
    }

    #[test]
    fn a_model_converts_for_pes_21_to_an_fmdl_fmdl_reads_on_the_game_s_skeleton() {
        let source = card(|_| {});
        let mtl = pre_fox_fixture("konami_card_red.mtl");
        let mut findings = Vec::new();
        let Ok(converted) = model_for_fox(
            "card.model",
            &source,
            &mtl,
            &context(PesVersion::Pes21),
            &mut findings,
        ) else {
            panic!("the card converts");
        };
        let model =
            fmdl::Model::from_file(&fmdl::FmdlFile::read(&converted.model).unwrap()).unwrap();
        let source_model =
            pes_model::model::Model::from_file(&PreFoxModel::read(&source).unwrap()).unwrap();
        assert_eq!(
            (source_model.meshes.len(), model.meshes.len()),
            (1, 1),
            "the card's meshes, then the FMDL's"
        );
        // The card's one bone is the game's own, at its pose: no skeleton of its own, and no
        // bone moved. What the member is told is what the FMDL adds and drops: the dummy
        // maps its `Basic_C` material lacks, and the bitangents an FMDL has no place for.
        assert!(converted.skeleton.is_none());
        let model_named = |rest: &[(&'static str, &str)]| {
            let mut context = vec![("model", "card.model".to_owned())];
            context.extend(rest.iter().map(|(key, value)| (*key, (*value).to_owned())));
            context
        };
        assert_eq!(
            findings,
            [
                (
                    Code::DummyTextureAdded,
                    Disposition::Keep,
                    model_named(&[("material", "0"), ("sampler", "NormalMap_Tex_NRM")])
                ),
                (
                    Code::DummyTextureAdded,
                    Disposition::Keep,
                    model_named(&[("material", "0"), ("sampler", "SpecularMap_Tex_LIN")])
                ),
                (
                    Code::VertexBitangentsDropped,
                    Disposition::Keep,
                    model_named(&[("mesh", "0")])
                ),
            ]
        );
    }

    #[test]
    fn a_far_vertex_in_the_converted_form_is_vertex_too_far_from_origin_in_both_directions() {
        let far_card = card(|model| model.meshes[0].vertices.positions[0] = [6000.0, 0.0, 0.0]);
        let mtl = pre_fox_fixture("konami_card_red.mtl");
        let mut findings = Vec::new();
        assert_eq!(
            failure(model_for_fox(
                "card.model",
                &far_card,
                &mtl,
                &context(PesVersion::Pes21),
                &mut findings
            )),
            Some((
                Code::VertexTooFarFromOrigin,
                vec![
                    ("model", "card.model".to_owned()),
                    ("count", "1".to_owned())
                ]
            ))
        );
        // The tracer's boots with one vertex 6000 units from the origin, for PES 17.
        let far_boots = std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/deep/boots_far.fmdl"),
        )
        .unwrap();
        assert_eq!(
            failure(fmdl_for_pre_fox(
                "boots.fmdl",
                &far_boots,
                None,
                &context(PesVersion::Pes17),
                &mut findings
            )),
            Some((
                Code::VertexTooFarFromOrigin,
                vec![
                    ("model", "boots.fmdl".to_owned()),
                    ("count", "1".to_owned())
                ]
            ))
        );
        assert_eq!(findings, [], "a failed conversion reports nothing else");
    }

    #[test]
    fn any_other_error_in_the_converted_form_is_model_conversion_failed_and_the_rest_nothing() {
        let rule = |code: &'static str, error: bool, count: usize| Fired { code, error, count };
        assert_eq!(
            failure(target_form_failure(
                "boots.model",
                vec![
                    rule("fmdl_weights_not_normalized", false, 1662),
                    rule("fmdl_mesh_over_face_limit", true, 1),
                ]
            )),
            Some((
                Code::ModelConversionFailed,
                vec![
                    ("model", "boots.model".to_owned()),
                    ("error", "fmdl_mesh_over_face_limit".to_owned())
                ]
            ))
        );
        // The far vertex is named first, its count summed over the model's meshes.
        assert_eq!(
            failure(target_form_failure(
                "boots.model",
                vec![
                    rule("fmdl_mesh_over_face_limit", true, 1),
                    rule("fmdl_vertex_far_from_origin", true, 2),
                    rule("fmdl_vertex_far_from_origin", true, 3),
                ]
            )),
            Some((
                Code::VertexTooFarFromOrigin,
                vec![
                    ("model", "boots.model".to_owned()),
                    ("count", "5".to_owned())
                ]
            ))
        );
        assert_eq!(
            failure(target_form_failure(
                "boots.fmdl",
                vec![
                    rule("model_weights_not_normalized", false, 4),
                    rule("model_mesh_empty", false, 1),
                ]
            )),
            None
        );
    }

    fn loss(code: &'static str, subject: Subject, detail: &str) -> loss::Finding {
        loss::Finding {
            code,
            subject,
            detail: detail.to_owned(),
        }
    }

    /// A loss's code, subject and detail, and the code and context after the model's name
    /// that `reported` gives it.
    type ReportedCase = (
        &'static str,
        Subject,
        &'static str,
        Code,
        &'static [(&'static str, &'static str)],
    );

    #[test]
    fn every_documented_loss_code_is_reported() {
        // Each code `model_convert::loss` documents, with a subject and a detail of the kind
        // its importers and exporters give it, and the finding the member is told.
        let cases: [ReportedCase; 17] = [
            (
                "bone_matrix_unknown",
                Subject::Bone(4),
                "my_bone",
                Code::BoneMatrixUnknown,
                &[("bone", "4"), ("name", "my_bone")],
            ),
            (
                "bone_slot_dropped",
                Subject::Mesh(2),
                "3",
                Code::BoneSlotDropped,
                &[("mesh", "2"), ("count", "3")],
            ),
            (
                "material_family_approximated",
                Subject::Material(1),
                "shirt",
                Code::MaterialFamilyApproximated,
                &[("material", "1"), ("name", "shirt")],
            ),
            (
                "material_split_by_flags",
                Subject::Material(0),
                "kit_1",
                Code::MaterialSplitByFlags,
                &[("material", "0"), ("name", "kit_1")],
            ),
            (
                "material_texture_unused",
                Subject::Material(2),
                "Tex_Sampler",
                Code::MaterialTextureUnused,
                &[("material", "2"), ("texture", "Tex_Sampler")],
            ),
            (
                "material_parameter_dropped",
                Subject::Material(0),
                "SpecularColor",
                Code::MaterialParameterDropped,
                &[("material", "0"), ("parameter", "SpecularColor")],
            ),
            (
                "sampler_settings_defaulted",
                Subject::Material(1),
                "DiffuseMap",
                Code::SamplerSettingsDefaulted,
                &[("material", "1"), ("sampler", "DiffuseMap")],
            ),
            (
                "vertex_bitangents_dropped",
                Subject::Mesh(0),
                "",
                Code::VertexBitangentsDropped,
                &[("mesh", "0")],
            ),
            (
                "dummy_texture_added",
                Subject::Material(0),
                "NormalMap_Tex_NRM",
                Code::DummyTextureAdded,
                &[("material", "0"), ("sampler", "NormalMap_Tex_NRM")],
            ),
            (
                "native_field_dropped",
                Subject::Model,
                "bone_matrices",
                Code::NativeFieldDropped,
                &[("field", "bone_matrices")],
            ),
            (
                "native_field_dropped",
                Subject::Material(1),
                "invisible",
                Code::MeshFlagsDropped,
                &[("material", "1"), ("field", "invisible")],
            ),
            // A PES 2017 model folded for PES 2015, as `model_convert`'s retargeting reports
            // it: the detail names the bone, in the index's place.
            (
                "bone_folded_for_version",
                Subject::Bone(3),
                "dsk_deltoid_l -> dsk_upperarm_l",
                Code::BoneFoldedForVersion,
                &[("bone", "dsk_deltoid_l -> dsk_upperarm_l")],
            ),
            (
                "bone_folded_by_position",
                Subject::Bone(9),
                "dsk_back -> sk_chest",
                Code::BoneFoldedByPosition,
                &[("bone", "dsk_back -> sk_chest")],
            ),
            (
                "skeleton_retargeted",
                Subject::Model,
                "7",
                Code::SkeletonRetargeted,
                &[("bones", "7")],
            ),
            (
                "static_bone_added",
                Subject::Mesh(1),
                "",
                Code::StaticBoneAdded,
                &[("mesh", "1")],
            ),
            (
                "weight_clamped",
                Subject::Mesh(0),
                "12",
                Code::WeightClamped,
                &[("mesh", "0"), ("count", "12")],
            ),
            (
                "empty_mesh_bone_group_dropped",
                Subject::Mesh(3),
                "175",
                Code::EmptyMeshBoneGroupDropped,
                &[("mesh", "3"), ("count", "175")],
            ),
        ];
        for (code, subject, detail, expected, context) in cases {
            let mut expected_context = vec![("model", "boots.fmdl".to_owned())];
            expected_context.extend(
                context
                    .iter()
                    .map(|(key, value)| (*key, (*value).to_owned())),
            );
            assert_eq!(
                reported("boots.fmdl", &loss(code, subject, detail)),
                Some((expected, Disposition::Keep, expected_context)),
                "{code} {detail}"
            );
        }
        // The other Fox mesh flag, and a `native_field_dropped` on a bone.
        assert_eq!(
            reported(
                "boots.fmdl",
                &loss(
                    "native_field_dropped",
                    Subject::Material(1),
                    "no_shadow_cast"
                )
            )
            .map(|(code, _, _)| code),
            Some(Code::MeshFlagsDropped)
        );
        assert_eq!(
            reported(
                "fcl_hair.fmdl",
                &loss("native_field_dropped", Subject::Bone(8), "skl_parent")
            ),
            Some((
                Code::NativeFieldDropped,
                Disposition::Keep,
                vec![
                    ("model", "fcl_hair.fmdl".to_owned()),
                    ("bone", "8".to_owned()),
                    ("field", "skl_parent".to_owned()),
                ]
            ))
        );
        // A code `loss.rs` does not document is no finding.
        assert_eq!(
            reported("boots.fmdl", &loss("no_such_loss", Subject::Model, "")),
            None
        );
    }
}
