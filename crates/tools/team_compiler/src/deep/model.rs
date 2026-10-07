//! The deep pass's model checks (`team_compiler/messages.md` "Model checks"): which rules
//! the format crates' checks fire on a native model (`.fmdl`, `.model`) or a `.mtl`, one
//! entry per code, and whether an FMDL carries hand weights.

use fmdl::{FmdlFile, Model};
use model_convert::ops::hand_split::fox_has_hand_weights;
use pes_model::format::PreFoxModel;
use pes_model::format::mtl::MaterialSet;

/// The format crates' far-vertex codes, both reported as `vertex_too_far_from_origin`.
pub(crate) const FAR_VERTEX_CODES: [&str; 2] = [
    "fmdl_vertex_far_from_origin",
    "model_vertex_far_from_origin",
];

/// Which format crate checks a model or material set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModelKind {
    /// A Fox model, checked by `fmdl`.
    Fmdl,
    /// A pre-Fox model, checked by `pes_model`.
    PreFoxModel,
    /// A pre-Fox material set, checked by `pes_model`.
    Mtl,
}

/// One rule a format crate's check fired on a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Fired {
    /// The format crate's code.
    pub(super) code: &'static str,
    /// Whether the format crate rates it an Error, which drops what holds the file.
    pub(super) error: bool,
    /// How many items tripped the rule.
    pub(super) count: usize,
}

impl Fired {
    /// `fmdl`'s finding `found` as a rule fired: its code, whether it is an Error, its count.
    fn fox(found: fmdl::check::Finding) -> Fired {
        let error = match found.severity {
            fmdl::check::Severity::Error => true,
            fmdl::check::Severity::Warning | fmdl::check::Severity::Info => false,
        };
        Fired {
            code: found.code,
            error,
            count: found.count,
        }
    }

    /// `pes_model`'s finding `found` as a rule fired: its code, whether it is an Error, its
    /// count.
    fn pre_fox(found: pes_model::check::Finding) -> Fired {
        let error = match found.severity {
            pes_model::check::Severity::Error => true,
            pes_model::check::Severity::Warning | pes_model::check::Severity::Info => false,
        };
        Fired {
            code: found.code,
            error,
            count: found.count,
        }
    }
}

/// What the deep pass learns from one parsed model or material set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ModelRead {
    /// The rules the format crate's check fired on it.
    pub(super) fired: Vec<Fired>,
    /// An FMDL whose vertices carry a positive weight on a hand-skeleton bone (`skh_*_l`,
    /// `skh_*_r`): planning gives a player folder holding it as a face part a gloves task, and
    /// the face and gloves tasks split it (`pipeline.md` "2. Per-export serial steps", step 6).
    /// Always `false` for a pre-Fox model or a material set.
    pub(super) hand_weighted: bool,
}

/// What `bytes`, read as `kind`, tells the deep pass (`ModelRead`), or the reader's error text
/// when they do not parse. The model is parsed once for both questions.
pub(super) fn fired(kind: ModelKind, bytes: &[u8]) -> Result<ModelRead, String> {
    let (fired, hand_weighted) = match kind {
        ModelKind::Fmdl => {
            let model = FmdlFile::read(bytes)
                .and_then(|file| Model::from_file(&file))
                .map_err(|error| error.to_string())?;
            let fired = fmdl::check::check(&model)
                .into_iter()
                .map(Fired::fox)
                .collect();
            (fired, fox_has_hand_weights(&model))
        }
        ModelKind::PreFoxModel => {
            let model = PreFoxModel::read(bytes)
                .and_then(|file| pes_model::model::Model::from_file(&file))
                .map_err(|error| error.to_string())?;
            let fired = pes_model::check::check(&model)
                .into_iter()
                .map(Fired::pre_fox)
                .collect();
            (fired, false)
        }
        ModelKind::Mtl => {
            let set = MaterialSet::read(bytes).map_err(|error| error.to_string())?;
            let fired = pes_model::check::check_materials(&set)
                .into_iter()
                .map(Fired::pre_fox)
                .collect();
            (fired, false)
        }
    };
    Ok(ModelRead {
        fired,
        hand_weighted,
    })
}

/// `fired` with one entry per code, in the order each code first fired, its counts summed:
/// the checks report per mesh or per material, a member reads one line per file.
pub(super) fn summed(fired: Vec<Fired>) -> Vec<Fired> {
    let mut summed: Vec<Fired> = Vec::new();
    for rule in fired {
        match summed.iter_mut().find(|known| known.code == rule.code) {
            Some(known) => known.count += rule.count,
            None => summed.push(rule),
        }
    }
    summed
}

#[cfg(test)]
mod tests {
    use aesthetics_export::{ContentFinding, Disposition};

    use super::*;
    use crate::deep::tests::{
        counted, edited, far_boots, findings_of, folder, glove_over_the_face_limit,
        pre_fox_fixture, tracer_boots,
    };
    use crate::testing::scratch;

    /// `code` on the folder `scope`: the file `file` does not parse, for `error`.
    fn broken(code: &'static str, scope: &str, file: &str, error: String) -> ContentFinding {
        ContentFinding {
            code,
            scope: folder(scope),
            context: vec![("file", file.to_owned()), ("error", error)],
            disposition: Disposition::DropFolder,
            pass_through_eligible: false,
        }
    }

    #[test]
    fn far_vertices_in_two_meshes_are_one_finding_with_their_summed_count() {
        let temp = scratch("deep_two_meshes");
        let boots = edited(&far_boots(), |model| {
            model.meshes[1].vertices.positions[0] = [0.0, 0.0, 7000.0];
        });
        let findings = findings_of(temp.path(), &[("Players/03 - A/boots.fmdl", boots)], &[]);
        let a = folder("Players/03 - A");
        assert_eq!(
            findings,
            [
                counted(
                    "vertex_too_far_from_origin",
                    &a,
                    "boots.fmdl",
                    2,
                    Disposition::DropFolder,
                    false
                ),
                counted(
                    "fmdl_weights_not_normalized",
                    &a,
                    "boots.fmdl",
                    1662,
                    Disposition::Keep,
                    false
                ),
            ]
        );
    }

    #[test]
    fn a_format_error_keeps_its_code_drops_the_folder_and_may_pass_through() {
        let temp = scratch("deep_format_error");
        let findings = findings_of(
            temp.path(),
            &[("Players/03 - A/glove_r.fmdl", glove_over_the_face_limit())],
            &[],
        );
        assert_eq!(
            findings,
            [counted(
                "fmdl_mesh_over_face_limit",
                &folder("Players/03 - A"),
                "glove_r.fmdl",
                21_846,
                Disposition::DropFolder,
                true
            )]
        );
    }

    #[test]
    fn the_tracer_s_boots_keep_their_folder_and_a_model_that_does_not_parse_drops_its_own() {
        let temp = scratch("deep_clean");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/boots.fmdl", tracer_boots()),
                ("Players/05 - B/boots.fmdl", b"not a model".to_vec()),
            ],
            &[],
        );
        let error = FmdlFile::read(b"not a model").unwrap_err().to_string();
        assert_eq!(
            findings,
            [
                counted(
                    "fmdl_weights_not_normalized",
                    &folder("Players/03 - A"),
                    "boots.fmdl",
                    1662,
                    Disposition::Keep,
                    false
                ),
                broken("model_broken", "Players/05 - B", "boots.fmdl", error),
            ]
        );
    }

    #[test]
    fn pre_fox_models_and_material_sets_are_checked() {
        let temp = scratch("deep_pre_fox");
        let findings = findings_of(
            temp.path(),
            &[
                (
                    "Players/03 - A/face_high.model",
                    pre_fox_fixture("konami_card.model"),
                ),
                (
                    "Players/03 - A/materials.mtl",
                    pre_fox_fixture("cardhead_materials.mtl"),
                ),
                (
                    "Players/05 - B/boots.model",
                    pre_fox_fixture("community_empty_geometry_boots.wesys.model"),
                ),
                (
                    "Players/05 - B/boots.mtl",
                    pre_fox_fixture("konami_shadow.mtl"),
                ),
            ],
            &[],
        );
        let b = folder("Players/05 - B");
        // The boots' two meshes are empty; the shadow's one material names no state.
        assert_eq!(
            findings,
            [
                counted(
                    "model_mesh_empty",
                    &b,
                    "boots.model",
                    2,
                    Disposition::Keep,
                    false
                ),
                counted(
                    "mtl_state_missing",
                    &b,
                    "boots.mtl",
                    7,
                    Disposition::Keep,
                    false
                ),
            ]
        );
    }

    #[test]
    fn a_pre_fox_model_s_far_vertex_is_vertex_too_far_from_origin() {
        let temp = scratch("deep_pre_fox_far");
        // Konami's referee card, clean, with its first vertex 6000 units from the origin.
        let file = PreFoxModel::read(&pre_fox_fixture("konami_card.model")).unwrap();
        let mut model = pes_model::model::Model::from_file(&file).unwrap();
        model.meshes[0].vertices.positions[0] = [6000.0, 0.0, 0.0];
        let card = model.to_file().unwrap().write().unwrap();
        // With a material set beside it, so the model's one finding is its far vertex.
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/card.model", card),
                (
                    "Players/03 - A/card.mtl",
                    pre_fox_fixture("cardhead_materials.mtl"),
                ),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [counted(
                "vertex_too_far_from_origin",
                &folder("Players/03 - A"),
                "card.model",
                1,
                Disposition::DropFolder,
                false
            )]
        );
    }

    #[test]
    fn a_pre_fox_error_keeps_its_code_drops_the_folder_and_may_pass_through() {
        let temp = scratch("deep_pre_fox_error");
        // The card head's material set, clean, with its one material listed twice.
        let mut set = MaterialSet::read(&pre_fox_fixture("cardhead_materials.mtl")).unwrap();
        set.materials.push(set.materials[0].clone());
        let findings = findings_of(
            temp.path(),
            &[("Players/03 - A/materials.mtl", set.write())],
            &[],
        );
        assert_eq!(
            findings,
            [counted(
                "mtl_material_duplicate",
                &folder("Players/03 - A"),
                "materials.mtl",
                1,
                Disposition::DropFolder,
                true
            )]
        );
    }

    #[test]
    fn a_pre_fox_model_or_material_set_that_does_not_parse_drops_its_folder() {
        let temp = scratch("deep_pre_fox_broken");
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/boots.model", b"not a model".to_vec()),
                ("Players/05 - B/boots.mtl", b"not a material set".to_vec()),
            ],
            &[],
        );
        let model_error = PreFoxModel::read(b"not a model").unwrap_err().to_string();
        let mtl_error = MaterialSet::read(b"not a material set")
            .unwrap_err()
            .to_string();
        // No `model_material_undefined` for the model with no `.mtl` beside it: the target is
        // PES 21, where a `.model` is not compiled yet.
        assert_eq!(
            findings,
            [
                broken("model_broken", "Players/03 - A", "boots.model", model_error),
                broken("mtl_broken", "Players/05 - B", "boots.mtl", mtl_error),
            ]
        );
    }
}
