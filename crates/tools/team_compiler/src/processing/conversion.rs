//! A selected model of the other engine's format converted for the target
//! (`team_compiler/pipeline.md` step 3 "Format conversion") through `model_convert::convert`,
//! which also moves it onto the target version's skeleton: on PES 15-17 an `.fmdl`, with its
//! paired `.skl` as the bind pose, written as a `.model` and its material set.

use model_convert::{Converted, NativeModelBundle, convert, loss};
use pes_model::format::mtl::MaterialSet;
use pes_version::PesVersion;
use studio_core::Disposition;

use super::{CompileContext, Finding, TaskFailure};
use crate::messages::Code;

/// An FMDL converted for a PES 15-17 target.
pub(super) struct PreFoxConversion {
    /// The `.model` written, packed in the FMDL's place.
    pub(super) model: Vec<u8>,
    /// Its material set, packed as `<stem>.mtl` beside it.
    pub(super) materials: MaterialSet,
}

/// The FMDL `name` (its file name), `bytes`, converted for `ctx.version`, a PES 15-17 target,
/// `skeleton` being the bytes of its paired `.skl`, its bind pose, when its folder holds one.
/// The conversion's parsed forms are charged to the run's memory budget at the source's size
/// while they live. What the conversion reports that the member is told is noted in
/// `findings` naming the model (`reported`). A model or skeleton the conversion cannot read,
/// convert or write fails the task with `model_conversion_failed`, naming the model.
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
        converted_fmdl(bytes, skeleton, ctx.version).map_err(|error| TaskFailure {
            code: Code::ModelConversionFailed,
            context: vec![("model", name.to_owned()), ("error", format!("{error:#}"))],
        })?
    };
    findings.extend(losses.iter().filter_map(|loss| reported(name, loss)));
    Ok(converted)
}

/// The FMDL `bytes` converted for the PES 15-17 `version`, with the skeleton `skeleton` as its
/// bind pose, and the conversion's loss findings.
fn converted_fmdl(
    bytes: &[u8],
    skeleton: Option<&[u8]>,
    version: PesVersion,
) -> anyhow::Result<(PreFoxConversion, Vec<loss::Finding>)> {
    let model = fmdl::Model::from_file(&fmdl::FmdlFile::read(bytes)?)?;
    let skl = skeleton.map(fmdl::SklFile::read).transpose()?;
    let Converted { bundle, findings } = convert(NativeModelBundle::Fox { model, skl }, version)?;
    match bundle {
        NativeModelBundle::PreFox { model, mtl } => Ok((
            PreFoxConversion {
                model: model.to_file()?.write()?,
                materials: mtl,
            },
            findings,
        )),
        NativeModelBundle::Fox { .. } => {
            unreachable!("`convert` returns the target's format, a `.model` for PES 15-17")
        }
    }
}

/// The finding the conversion's `loss` of the model `name` (its file name) is reported as: an
/// Info on the folder naming the model and, for `bone_folded_for_version`, the bone folded and
/// the one it folded onto (`bone`, `<bone> -> <target>`), for `skeleton_retargeted` how many
/// bones moved (`bones`). Any other loss is not reported yet: the conversion's remaining codes
/// are mapped by a later step, for every converted model.
fn reported(name: &str, loss: &loss::Finding) -> Option<Finding> {
    let (code, key) = match loss.code {
        "bone_folded_for_version" => (Code::BoneFoldedForVersion, "bone"),
        "skeleton_retargeted" => (Code::SkeletonRetargeted, "bones"),
        _ => return None,
    };
    Some((
        code,
        Disposition::Keep,
        vec![("model", name.to_owned()), (key, loss.detail.clone())],
    ))
}

#[cfg(test)]
mod tests {
    use model_convert::Subject;

    use super::*;

    fn loss(code: &'static str, subject: Subject, detail: &str) -> loss::Finding {
        loss::Finding {
            code,
            subject,
            detail: detail.to_owned(),
        }
    }

    #[test]
    fn a_fold_and_a_retargeted_skeleton_are_reported_and_the_other_losses_not_yet() {
        // A PES 2017 model folded for PES 2015, as `model_convert`'s retargeting reports it.
        assert_eq!(
            reported(
                "boots.fmdl",
                &loss(
                    "bone_folded_for_version",
                    Subject::Bone(3),
                    "dsk_deltoid_l -> dsk_upperarm_l"
                )
            ),
            Some((
                Code::BoneFoldedForVersion,
                Disposition::Keep,
                vec![
                    ("model", "boots.fmdl".to_owned()),
                    ("bone", "dsk_deltoid_l -> dsk_upperarm_l".to_owned()),
                ]
            ))
        );
        assert_eq!(
            reported(
                "boots.fmdl",
                &loss("skeleton_retargeted", Subject::Model, "7")
            ),
            Some((
                Code::SkeletonRetargeted,
                Disposition::Keep,
                vec![
                    ("model", "boots.fmdl".to_owned()),
                    ("bones", "7".to_owned()),
                ]
            ))
        );
        // What the tracer's models report on PES 2017, none of it shown yet.
        for (code, subject, detail) in [
            ("native_field_dropped", Subject::Model, "bone_matrices"),
            ("native_field_dropped", Subject::Bone(8), "skl_parent"),
            (
                "native_field_dropped",
                Subject::Material(1),
                "no_shadow_cast",
            ),
        ] {
            assert_eq!(
                reported("fcl_hair.fmdl", &loss(code, subject, detail)),
                None
            );
        }
    }
}
