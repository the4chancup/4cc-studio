//! The deep pass's model checks (`team_compiler/messages.md` "Model checks"): which rules
//! the format crates' checks fire on a native model (`.fmdl`, `.model`) or a `.mtl`, one
//! entry per code, and whether a model (`.fmdl` or `.model`) carries hand weights.

use fmdl::{FmdlFile, Model};
use model_convert::ops::hand_split::{fox_has_hand_weights, prefox_has_hand_weights};
use pes_model::format::PreFoxModel;
use pes_model::format::mtl::{Material, MaterialEntry, MaterialSet};

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

/// One rule a format crate's check fired on a file, or on a converted model in its target form
/// (`processing::conversion`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Fired {
    /// The format crate's code.
    pub(crate) code: &'static str,
    /// Whether the format crate rates it an Error, which drops what holds the file.
    pub(crate) error: bool,
    /// How many items tripped the rule.
    pub(crate) count: usize,
}

impl Fired {
    /// `fmdl`'s finding `found` as a rule fired: its code, whether it is an Error, its count.
    pub(crate) fn fox(found: fmdl::check::Finding) -> Fired {
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
    pub(crate) fn pre_fox(found: pes_model::check::Finding) -> Fired {
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
    /// A model whose vertices carry a positive weight on a hand-skeleton bone (`skh_*_l`,
    /// `skh_*_r`), an FMDL or a pre-Fox `.model`, each read by its own format's check: on Fox
    /// planning gives a player folder holding it as a face part a gloves task, and the face and
    /// gloves tasks split it (`pipeline.md` "2. Per-export serial steps", step 6); on pre-Fox
    /// the face task splits it into two more `face.xml` entries. Always `false` for a material
    /// set.
    pub(super) hand_weighted: bool,
    /// Its materials, in order: for a pre-Fox model the names it lists (`Model::materials`, as
    /// `pes_model::check::check_bundle` compares them, a name no mesh uses included), for a
    /// material set the ones it defines with their texture paths; empty for an FMDL, whose
    /// materials are not paired with a material set. The deep pass compares a model's names
    /// with its `.mtl`'s (`model_material_undefined`) and looks for the textures of the ones
    /// its meshes bind (`mtl_texture_not_found`).
    pub(super) materials: Vec<MaterialRead>,
}

/// One material a parsed pre-Fox file names (`ModelRead::materials`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MaterialRead {
    /// Its name: the one a model binds, or the one a material set defines.
    pub(super) name: String,
    /// The texture paths its samplers name, in order, as written; empty for a model.
    pub(super) paths: Vec<String>,
    /// For a model's, whether one of its meshes binds it: a name no mesh binds
    /// (`model_material_unused`) is never drawn, so its textures are never loaded. `false` for
    /// a material set's.
    pub(super) mesh_used: bool,
}

/// What `bytes`, read as `kind`, tells the deep pass (`ModelRead`), or the reader's error text
/// when they do not parse. The model is parsed once for both questions.
pub(super) fn fired(kind: ModelKind, bytes: &[u8]) -> Result<ModelRead, String> {
    let read = match kind {
        ModelKind::Fmdl => {
            let model = FmdlFile::read(bytes)
                .and_then(|file| Model::from_file(&file))
                .map_err(|error| error.to_string())?;
            ModelRead {
                fired: fmdl::check::check(&model)
                    .into_iter()
                    .map(Fired::fox)
                    .collect(),
                hand_weighted: fox_has_hand_weights(&model),
                materials: Vec::new(),
            }
        }
        ModelKind::PreFoxModel => {
            let model = PreFoxModel::read(bytes)
                .and_then(|file| pes_model::model::Model::from_file(&file))
                .map_err(|error| error.to_string())?;
            let materials = model
                .materials
                .iter()
                .enumerate()
                .map(|(index, name)| MaterialRead {
                    name: name.clone(),
                    paths: Vec::new(),
                    mesh_used: model.meshes.iter().any(|mesh| mesh.material == index),
                })
                .collect();
            ModelRead {
                fired: pes_model::check::check(&model)
                    .into_iter()
                    .map(Fired::pre_fox)
                    .collect(),
                hand_weighted: prefox_has_hand_weights(&model),
                materials,
            }
        }
        ModelKind::Mtl => {
            let set = MaterialSet::read(bytes).map_err(|error| error.to_string())?;
            ModelRead {
                fired: pes_model::check::check_materials(&set)
                    .into_iter()
                    .map(Fired::pre_fox)
                    .collect(),
                hand_weighted: false,
                materials: set.materials.into_iter().map(material_read).collect(),
            }
        }
    };
    Ok(read)
}

/// `material` of a material set as the deep pass keeps it: its name and every sampler's
/// texture path, a path two samplers name included.
fn material_read(material: Material) -> MaterialRead {
    let paths = material
        .entries
        .into_iter()
        .filter_map(|entry| match entry {
            MaterialEntry::Sampler(sampler) => Some(sampler.path),
            MaterialEntry::State(_) | MaterialEntry::Vector(_) => None,
        })
        .collect();
    MaterialRead {
        name: material.name,
        paths,
        mesh_used: false,
    }
}

/// `fired` with one entry per code, in the order each code first fired, its counts summed:
/// the checks report per mesh or per material, a member reads one line per file.
pub(crate) fn summed(fired: Vec<Fired>) -> Vec<Fired> {
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
    use aesthetics_export::{ContentFinding, Disposition, IssueScope};
    use pes_version::PesVersion;

    use super::*;
    use crate::deep::tests::{
        bc1_dds, counted, edited, far_boots, findings_for, findings_of, fixture, folder,
        glove_over_the_face_limit, mtl_texture, path, pre_fox_fixture, tracer_boots, tracer_file,
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
        // The boots' two meshes are empty; the shadow's one material names no state. On PES
        // 21 each `.model`, no FMDL beside it, is converted with the `.mtl` its search finds,
        // which here defines none of its materials, and whose texture paths are looked up: the
        // card head's `./texture.dds`, of a material no mesh binds, is missing.
        assert_eq!(
            findings,
            [
                undefined(
                    "Players/03 - A",
                    "face_high.model",
                    "materials.mtl",
                    "judge_card_red"
                ),
                mtl_texture(
                    "mtl_texture_unused_missing",
                    folder("Players/03 - A"),
                    "materials.mtl",
                    "./texture.dds",
                    "card"
                ),
                counted(
                    "model_mesh_empty",
                    &b,
                    "boots.model",
                    2,
                    Disposition::Keep,
                    false
                ),
                undefined(
                    "Players/05 - B",
                    "boots.model",
                    "boots.mtl",
                    "Boots_Game_mat, Boots_Game_Alpha_mat"
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
    fn a_pre_fox_model_s_mtl_may_be_reached_through_a_link_and_a_model_link_s_in_common() {
        let temp = scratch("deep_pre_fox_links");
        // The card head's model and its material set, which defines the model's one material
        // and names `./texture.dds`.
        let card = || pre_fox_fixture("cardhead_face_high.model");
        let materials = || pre_fox_fixture("cardhead_materials.mtl");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                // A model whose one `.mtl` is a link, its target present.
                ("Players/03 - A/hat.model", card()),
                ("Players/03 - A/hat.mtl.common", Vec::new()),
                ("Common/hat.mtl", materials()),
                // A model link whose `.mtl` is the Common one of its name.
                ("Players/05 - B/legs.model.common", Vec::new()),
                ("Common/legs.model", card()),
                ("Common/legs.mtl", materials()),
                ("Common/texture.dds", bc1_dds(4, 4)),
            ],
            &[],
            &[],
        );
        assert_eq!(findings, []);
        // A model link with no `.mtl` anywhere, `Common/` holding none.
        let temp = scratch("deep_pre_fox_link_undefined");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                ("Players/05 - B/legs.model.common", Vec::new()),
                ("Common/legs.model", card()),
            ],
            &[],
            &[],
        );
        assert_eq!(
            findings,
            [ContentFinding {
                code: "model_material_undefined",
                scope: folder("Players/05 - B"),
                context: vec![("file", "legs.model.common".to_owned())],
                disposition: Disposition::DropFolder,
                pass_through_eligible: false,
            }]
        );
    }

    #[test]
    fn under_ingame_face_a_model_link_with_no_mtl_anywhere_is_undefined() {
        // The link is a part of his boots, whose `boots.mtl` needs the Common model's `.mtl`.
        let temp = scratch("deep_pre_fox_marked_link_undefined");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                ("Players/05 - B/ingame_face", Vec::new()),
                ("Players/05 - B/kit_boots.model.common", Vec::new()),
                (
                    "Common/kit_boots.model",
                    pre_fox_fixture("cardhead_face_high.model"),
                ),
            ],
            &[],
            &[],
        );
        assert_eq!(
            findings,
            [ContentFinding {
                code: "model_material_undefined",
                scope: folder("Players/05 - B"),
                context: vec![("file", "kit_boots.model.common".to_owned())],
                disposition: Disposition::DropFolder,
                pass_through_eligible: false,
            }]
        );
    }

    #[test]
    fn a_model_link_s_search_sees_only_the_common_files_the_common_pass_kept() {
        let temp = scratch("deep_pre_fox_link_broken_mtl");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                ("Players/05 - B/legs.model.common", Vec::new()),
                ("Common/legs.model", pre_fox_fixture("konami_card.model")),
                ("Common/legs.mtl", b"not a material set".to_vec()),
            ],
            &[],
            &[],
        );
        let error = MaterialSet::read(b"not a material set")
            .unwrap_err()
            .to_string();
        assert_eq!(
            findings,
            [
                ContentFinding {
                    code: "model_material_undefined",
                    scope: folder("Players/05 - B"),
                    context: vec![("file", "legs.model.common".to_owned())],
                    disposition: Disposition::DropFolder,
                    pass_through_eligible: false,
                },
                ContentFinding {
                    code: "mtl_broken",
                    scope: IssueScope::File(path("Common/legs.mtl")),
                    context: vec![("file", "legs.mtl".to_owned()), ("error", error)],
                    disposition: Disposition::DropFile,
                    pass_through_eligible: false,
                },
            ]
        );
    }

    /// `model_material_undefined` on the folder `scope`: the model `file` uses `materials`,
    /// which the `.mtl` it is paired with, `mtl`, does not define.
    fn undefined(scope: &str, file: &str, mtl: &str, materials: &str) -> ContentFinding {
        ContentFinding {
            code: "model_material_undefined",
            scope: folder(scope),
            context: vec![
                ("file", file.to_owned()),
                ("mtl", mtl.to_owned()),
                ("materials", materials.to_owned()),
            ],
            disposition: Disposition::DropFolder,
            pass_through_eligible: true,
        }
    }

    #[test]
    fn a_pre_fox_model_material_its_mtl_lacks_is_undefined_and_may_pass_through() {
        // Konami's referee card binds `judge_card_red`; the card head's material set defines
        // only `card`, which the card head's model binds.
        let temp = scratch("deep_pre_fox_named_undefined");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                (
                    "Players/03 - A/face_high.model",
                    pre_fox_fixture("konami_card.model"),
                ),
                (
                    "Players/03 - A/face_high.mtl",
                    pre_fox_fixture("cardhead_materials.mtl"),
                ),
                ("Players/03 - A/texture.dds", bc1_dds(4, 4)),
                (
                    "Players/05 - B/face_high.model",
                    pre_fox_fixture("cardhead_face_high.model"),
                ),
                (
                    "Players/05 - B/face_high.mtl",
                    pre_fox_fixture("cardhead_materials.mtl"),
                ),
                ("Players/05 - B/texture.dds", bc1_dds(4, 4)),
            ],
            &[],
            &[],
        );
        assert_eq!(
            findings,
            [undefined(
                "Players/03 - A",
                "face_high.model",
                "face_high.mtl",
                "judge_card_red"
            )]
        );
    }

    #[test]
    fn on_pes_21_a_model_no_fmdl_beats_is_paired_and_a_beaten_one_and_its_mtl_read_by_nothing() {
        let temp = scratch("deep_fox_pairings");
        let findings = findings_of(
            temp.path(),
            &[
                // Selected, its `.mtl` defining only the card head's `card`.
                (
                    "Players/03 - A/face_high.model",
                    pre_fox_fixture("konami_card.model"),
                ),
                (
                    "Players/03 - A/face_high.mtl",
                    pre_fox_fixture("cardhead_materials.mtl"),
                ),
                // Selected, with no `.mtl` for its search to find.
                (
                    "Players/05 - B/face_high.model",
                    pre_fox_fixture("konami_card.model"),
                ),
                // Beaten by the FMDL of its stem (the tracer's right glove, which `fmdl`'s
                // check finds nothing in): neither it nor the `.mtl` beside it, neither of which
                // even parses, is read, compared or reported.
                ("Players/07 - C/face_high.fmdl", tracer_file("glove_r.fmdl")),
                ("Players/07 - C/face_high.model", b"not a model".to_vec()),
                (
                    "Players/07 - C/face_high.mtl",
                    b"not a material set".to_vec(),
                ),
            ],
            &[],
        );
        // The selected model's `.mtl` has its texture paths looked up: the card head's
        // `./texture.dds`, of a material no mesh binds, is missing.
        assert_eq!(
            findings,
            [
                undefined(
                    "Players/03 - A",
                    "face_high.model",
                    "face_high.mtl",
                    "judge_card_red"
                ),
                mtl_texture(
                    "mtl_texture_unused_missing",
                    folder("Players/03 - A"),
                    "face_high.mtl",
                    "./texture.dds",
                    "card"
                ),
                ContentFinding {
                    code: "model_material_undefined",
                    scope: folder("Players/05 - B"),
                    context: vec![("file", "face_high.model".to_owned())],
                    disposition: Disposition::DropFolder,
                    pass_through_eligible: false,
                },
            ]
        );
    }

    #[test]
    fn a_model_link_s_common_model_is_compared_with_the_mtl_its_search_finds() {
        let temp = scratch("deep_pre_fox_link_named_undefined");
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                // The folder's own `.mtl` of the link's name, which the card's material is
                // not in.
                ("Players/05 - B/legs.model.common", Vec::new()),
                (
                    "Players/05 - B/legs.mtl",
                    pre_fox_fixture("cardhead_materials.mtl"),
                ),
                ("Players/05 - B/texture.dds", bc1_dds(4, 4)),
                ("Common/legs.model", pre_fox_fixture("konami_card.model")),
                // A `Common/` `.mtl`, named by its export path.
                ("Players/07 - C/hat.model.common", Vec::new()),
                ("Common/hat.model", pre_fox_fixture("konami_card.model")),
                ("Common/hat.mtl", pre_fox_fixture("cardhead_materials.mtl")),
                ("Common/texture.dds", bc1_dds(4, 4)),
            ],
            &[],
            &[],
        );
        assert_eq!(
            findings,
            [
                undefined(
                    "Players/05 - B",
                    "legs.model.common",
                    "legs.mtl",
                    "judge_card_red"
                ),
                undefined(
                    "Players/07 - C",
                    "hat.model.common",
                    "Common/hat.mtl",
                    "judge_card_red"
                ),
            ]
        );
    }

    #[test]
    fn a_pre_fox_model_weighted_to_hand_bones_reads_hand_weighted() {
        let body = fired(ModelKind::PreFoxModel, &fixture("hand_split/body.model")).unwrap();
        assert!(body.hand_weighted);
        let card = fired(
            ModelKind::PreFoxModel,
            &pre_fox_fixture("cardhead_face_high.model"),
        )
        .unwrap();
        assert!(!card.hand_weighted);
    }

    #[test]
    fn a_pre_fox_model_s_far_vertex_is_vertex_too_far_from_origin() {
        let temp = scratch("deep_pre_fox_far");
        // Konami's referee card, clean, with its first vertex 6000 units from the origin.
        let file = PreFoxModel::read(&pre_fox_fixture("konami_card.model")).unwrap();
        let mut model = pes_model::model::Model::from_file(&file).unwrap();
        model.meshes[0].vertices.positions[0] = [6000.0, 0.0, 0.0];
        let card = model.to_file().unwrap().write().unwrap();
        // With the material set defining its material beside it, so the model's one finding is
        // its far vertex (the set names no state, its own finding).
        let findings = findings_of(
            temp.path(),
            &[
                ("Players/03 - A/card.model", card),
                (
                    "Players/03 - A/card.mtl",
                    pre_fox_fixture("konami_card_red.mtl"),
                ),
            ],
            &[],
        );
        assert_eq!(
            findings,
            [
                counted(
                    "vertex_too_far_from_origin",
                    &folder("Players/03 - A"),
                    "card.model",
                    1,
                    Disposition::DropFolder,
                    false
                ),
                counted(
                    "mtl_state_missing",
                    &folder("Players/03 - A"),
                    "card.mtl",
                    7,
                    Disposition::Keep,
                    false
                ),
            ]
        );
    }

    #[test]
    fn a_pre_fox_error_keeps_its_code_drops_the_folder_and_may_pass_through() {
        let temp = scratch("deep_pre_fox_error");
        // The card head's material set, clean, with its one material listed twice, beside the
        // texture it names and no model: read for PES 17, where every `.mtl` is packed (on PES
        // 21 nothing reads a `.mtl` no `.model` is converted with).
        let mut set = MaterialSet::read(&pre_fox_fixture("cardhead_materials.mtl")).unwrap();
        set.materials.push(set.materials[0].clone());
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                ("Players/03 - A/materials.mtl", set.write()),
                ("Players/03 - A/texture.dds", bc1_dds(4, 4)),
            ],
            &[],
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
        // For PES 17, where the `.mtl` alone in its folder is read (on PES 21 nothing reads a
        // `.mtl` no `.model` is converted with).
        let findings = findings_for(
            PesVersion::Pes17,
            temp.path(),
            &[
                ("Players/03 - A/boots.model", b"not a model".to_vec()),
                ("Players/05 - B/boots.mtl", b"not a material set".to_vec()),
            ],
            &[],
            &[],
        );
        let model_error = PreFoxModel::read(b"not a model").unwrap_err().to_string();
        let mtl_error = MaterialSet::read(b"not a material set")
            .unwrap_err()
            .to_string();
        // The model with no `.mtl` beside it has every material undefined too.
        assert_eq!(
            findings,
            [
                broken("model_broken", "Players/03 - A", "boots.model", model_error),
                ContentFinding {
                    code: "model_material_undefined",
                    scope: folder("Players/03 - A"),
                    context: vec![("file", "boots.model".to_owned())],
                    disposition: Disposition::DropFolder,
                    pass_through_eligible: false,
                },
                broken("mtl_broken", "Players/05 - B", "boots.mtl", mtl_error),
            ]
        );
    }
}
