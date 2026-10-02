//! What `compile` builds in Phase 3, and the gate that skips any export holding anything else
//! (`team_compiler/README.md` "Phase 3 scope"): one classification, which planning reads to
//! skip an export and processing reads for each file's role, so the two never disagree.
//! Phase 3 only: withdrawn in Phase 4, when `compile` builds everything an export holds.

use aesthetics_export::{
    ExportIdentity, FileDescriptor, FileKind, FpcDirective, KitLayout, ModelFormat, PlayerFolder,
    PlayerIndex, ResolvedAestheticsExport, ValidatedRoster,
};
use pes_version::{Engine, PesVersion};
use vtree::ScopePath;

/// The face model names the Fox engine loads.
const FACE_STEMS: [&str; 4] = ["face_high", "hair_high", "oral", "fcl_hair"];

/// The kit texture stems, in the order of the kit config's texture-name fields.
pub(crate) const KIT_TEXTURE_STEMS: [&str; 5] =
    ["kit", "kit_back", "kit_chest", "kit_leg", "kit_name"];

/// The texture formats Phase 3 compiles for the Fox engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextureFormat {
    /// `.dds`, converted to FTEX.
    Dds,
    /// `.ftex`, packed as it is.
    Ftex,
}

/// The format of the texture file `name`, by its extension in any case; `None` for every other
/// image format, which Phase 3 does not compile.
pub(crate) fn texture_format(name: &str) -> Option<TextureFormat> {
    let (_, extension) = name.rsplit_once('.')?;
    if extension.eq_ignore_ascii_case("dds") {
        Some(TextureFormat::Dds)
    } else if extension.eq_ignore_ascii_case("ftex") {
        Some(TextureFormat::Ftex)
    } else {
        None
    }
}

/// What one file of a player folder becomes in the face output.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum FaceFile {
    /// A face model with this stem, packed as `<stem>.fmdl` with its texture paths rewritten.
    Model(String),
    /// A texture with this stem and format, converted into the player's common folder.
    Texture(String, TextureFormat),
    /// A file packed into the face package as it is, under this name.
    Packed(&'static str),
}

/// Whether `folder` holds `fcl_hair.fmdl`, the model a `fcl_hair.skl` pairs with: the
/// `holds_hair_model` that `face_file` takes.
pub(crate) fn holds_hair_model(folder: &PlayerFolder) -> bool {
    folder
        .files
        .iter()
        .any(|file| file.path.name() == "fcl_hair.fmdl")
}

/// What `file` of the player folder at `folder` becomes in the face output; `None` when it has
/// no Phase 3 role. `holds_hair_model`: the folder holds `fcl_hair.fmdl`.
pub(crate) fn face_file(
    folder: &ScopePath,
    file: &FileDescriptor,
    holds_hair_model: bool,
) -> Option<FaceFile> {
    if file.path.parent().as_ref() != Some(folder) {
        return None;
    }
    let name = file.path.name();
    let stem = file_stem(name);
    match file.kind {
        FileKind::Model(ModelFormat::Fmdl) if FACE_STEMS.contains(&stem) => {
            Some(FaceFile::Model(stem.to_owned()))
        }
        FileKind::Texture => {
            texture_format(name).map(|format| FaceFile::Texture(stem.to_owned(), format))
        }
        // The game loads the hair simulation's skeleton as `fcl_hair_sim.skl`; the export
        // names a skeleton after the model it pairs with, and without that model the skeleton
        // has nothing to drive.
        FileKind::Skl if stem == "fcl_hair" && holds_hair_model => {
            Some(FaceFile::Packed("fcl_hair_sim.skl"))
        }
        FileKind::Bin if name == "face_diff.bin" => Some(FaceFile::Packed("face_diff.bin")),
        FileKind::Fclo if name == "fcl_hair_sim.fclo" => {
            Some(FaceFile::Packed("fcl_hair_sim.fclo"))
        }
        FileKind::Model(_)
        | FileKind::Skl
        | FileKind::Bin
        | FileKind::Fclo
        | FileKind::Xml
        | FileKind::Mtl
        | FileKind::MaterialsToml
        | FileKind::SharedLink(_)
        | FileKind::CommonLink
        | FileKind::Marker(_)
        | FileKind::Metadata(_)
        | FileKind::Other => None,
    }
}

/// A file name's stem: the name up to its last `.`.
pub(crate) fn file_stem(name: &str) -> &str {
    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}

/// The first thing in `resolved` that `compile` cannot build yet for `version`, as the context
/// entry of `content_not_yet_compiled`: `what` (a path, the target or `refs`), or `missing` (a
/// file a Fox face needs that its folder lacks). `None` when Phase 3 builds all of it. Planning
/// drops a Fox target's `kit_mask` before asking: the gate would count it.
pub(crate) fn first_not_compiled(
    resolved: &ResolvedAestheticsExport,
    version: PesVersion,
) -> Option<(&'static str, String)> {
    // The walk's order is the plan's ("Phase 3 scope"), so one export always names the same
    // item; reordering the checks changes which item a member is told about.
    match version.engine() {
        Engine::Fox => {}
        Engine::PreFox => return Some(("what", version.to_string())),
    }
    if resolved.identity == ExportIdentity::Referees {
        return Some(("what", "refs".to_owned()));
    }
    let export = &resolved.export;
    let mapped: Vec<PlayerIndex> = match &export.roster {
        ValidatedRoster::Team(slots) => slots.values().copied().collect(),
        ValidatedRoster::Referees(slots) => slots.values().copied().collect(),
    };
    // A folder no roster slot maps is not compiled, so whatever it holds does not count.
    for (index, folder) in export.players.iter().enumerate() {
        if mapped.contains(&PlayerIndex(index))
            && let Some(item) = face_not_compiled(folder)
        {
            return Some(item);
        }
    }
    let mut shared = export
        .faces
        .iter()
        .chain(&export.boots)
        .chain(&export.gloves)
        .flat_map(|folder| &folder.files);
    if let Some(file) = shared.next() {
        return Some(what_entry(file));
    }
    for kit in export.kits.kits.values() {
        // The target is Fox here, so a kit drawn for the other layout needs a conversion.
        if kit.layout == Some(KitLayout::PreFox) {
            return Some(("what", kit.path.as_str().to_owned()));
        }
        let refused = kit.textures.iter().find(|texture| {
            !KIT_TEXTURE_STEMS.contains(&texture.stem.as_str())
                || texture_format(texture.file.path.name()).is_none()
        });
        if let Some(texture) = refused {
            return Some(what_entry(&texture.file));
        }
    }
    // A small logo comes only with a main one, which is named first, so the small one is not
    // walked.
    let mut rest = export
        .portraits
        .values()
        .chain(export.logo.as_ref().map(|logo| &logo.main.file))
        .chain(&export.collars)
        .chain(&export.common);
    rest.next().map(what_entry)
}

/// The first thing in the mapped player `folder` that Phase 3 cannot build into a Fox face.
fn face_not_compiled(folder: &PlayerFolder) -> Option<(&'static str, String)> {
    let holds_hair_model = holds_hair_model(folder);
    let mut roles = Vec::new();
    // Every file the folder holds is checked before any file it lacks: a file with no role
    // may be the missing one, misnamed (`Face_Diff.bin`).
    for file in &folder.files {
        match face_file(&folder.path, file, holds_hair_model) {
            Some(role) => roles.push(role),
            None => return Some(what_entry(file)),
        }
    }
    let path = folder.path.as_str();
    if let Some(portrait) = &folder.portrait {
        return Some(what_entry(portrait));
    }
    if folder.ingame_face {
        return Some(("what", format!("{path}/ingame_face")));
    }
    // `fpc.on` changes every kit config; `fpc.off` only reaches the savefile.
    if folder.fpc == Some(FpcDirective::On) {
        return Some(("what", format!("{path}/fpc.on")));
    }
    if !roles.iter().any(|role| matches!(role, FaceFile::Model(_))) {
        return Some(("what", path.to_owned()));
    }
    // The files a Fox face needs beside its models, which Phase 4 injects when they are missing.
    if !roles.contains(&FaceFile::Packed("face_diff.bin")) {
        return Some(("missing", format!("{path}/face_diff.bin")));
    }
    if holds_hair_model && !roles.contains(&FaceFile::Packed("fcl_hair_sim.fclo")) {
        return Some(("missing", format!("{path}/fcl_hair_sim.fclo")));
    }
    // The export's `fcl_hair.skl` is packed as `fcl_hair_sim.skl`.
    if holds_hair_model && !roles.contains(&FaceFile::Packed("fcl_hair_sim.skl")) {
        return Some(("missing", format!("{path}/fcl_hair.skl")));
    }
    None
}

/// The `what` context entry naming `file` by its export path.
fn what_entry(file: &FileDescriptor) -> (&'static str, String) {
    ("what", file.path.as_str().to_owned())
}

#[cfg(test)]
mod tests {
    use studio_core::ExportId;

    use super::*;
    use crate::plan::{TaskKind, plan_run};
    use crate::testing::{resolved, resolved_with_issues};

    /// A Fox face folder Phase 3 compiles: a face model and `face_diff.bin`.
    const FACE: [&str; 2] = [
        "Players/03 - A/face_high.fmdl",
        "Players/03 - A/face_diff.bin",
    ];

    /// The gate's first hit in the export `co - Gate` holding `files`, for PES 21.
    fn first_hit(files: &[&str]) -> Option<(&'static str, String)> {
        let files: Vec<(&str, u64)> = files.iter().map(|path| (*path, 1)).collect();
        first_not_compiled(&resolved("co - Gate", &files, &[], None), PesVersion::Pes21)
    }

    /// The gate's first hit in the export `co - Gate` holding `FACE` and `files`, for PES 21.
    fn gate(files: &[&str]) -> Option<(&'static str, String)> {
        first_hit(&[FACE.as_slice(), files].concat())
    }

    fn what(path: &str) -> Option<(&'static str, String)> {
        Some(("what", path.to_owned()))
    }

    fn missing(path: &str) -> Option<(&'static str, String)> {
        Some(("missing", path.to_owned()))
    }

    #[test]
    fn fox_face_folders_and_kits_are_compiled() {
        assert_eq!(gate(&[]), None);
        assert_eq!(
            gate(&[
                "Players/03 - A/hair_high.fmdl",
                "Players/03 - A/oral.fmdl",
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/fcl_hair.skl",
                "Players/03 - A/fcl_hair_sim.fclo",
                "Players/03 - A/skin.dds",
                "Players/03 - A/hair.FTEX",
                "Kits/g1/kit.dds",
                "Kits/g1/config.toml",
                "Kits/g1/fox",
                "Kits/p1/kit_back.ftex",
                "Kits/p1/kit_chest.dds",
                "Kits/p1/kit_leg.dds",
                "Kits/p1/kit_name.dds",
            ]),
            None
        );
    }

    #[test]
    fn a_pre_fox_target_is_named() {
        let export = resolved("co - Gate", &[(FACE[0], 1), (FACE[1], 1)], &[], None);
        assert_eq!(
            first_not_compiled(&export, PesVersion::Pes17),
            what("PES 2017")
        );
        for version in [PesVersion::Pes15, PesVersion::Pes16] {
            assert_eq!(
                first_not_compiled(&export, version),
                what(&version.to_string())
            );
        }
    }

    #[test]
    fn a_referee_export_is_named_refs() {
        let export = resolved(
            "refs Cup",
            &[
                ("Players/Keeper/face_high.fmdl", 1),
                ("Players/Keeper/face_diff.bin", 1),
            ],
            &[],
            Some(b"01 Keeper\n"),
        );
        assert_eq!(first_not_compiled(&export, PesVersion::Pes21), what("refs"));
    }

    #[test]
    fn a_player_file_with_no_role_is_named() {
        for file in [
            "Players/03 - A/boots.fmdl",
            "Players/03 - A/hair.png",
            "Players/03 - A/face/hair_high.fmdl",
        ] {
            assert_eq!(gate(&[file]), what(file), "{file}");
        }
    }

    #[test]
    fn a_hair_skeleton_without_its_model_is_named() {
        assert_eq!(
            gate(&["Players/03 - A/fcl_hair.skl"]),
            what("Players/03 - A/fcl_hair.skl")
        );
    }

    #[test]
    fn a_portrait_ingame_face_or_fpc_on_is_named() {
        assert_eq!(
            gate(&["Players/03 - A/portrait.dds"]),
            what("Players/03 - A/portrait.dds")
        );
        assert_eq!(
            gate(&["Players/03 - A/fpc.on"]),
            what("Players/03 - A/fpc.on")
        );
        // `ingame_face` excludes an explicit face model, so this folder holds only the hair
        // model; the marker is named before the hair files it lacks.
        assert_eq!(
            first_hit(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/face_diff.bin",
                "Players/03 - A/ingame_face",
            ]),
            what("Players/03 - A/ingame_face")
        );
    }

    #[test]
    fn a_folder_without_a_face_model_or_a_file_a_fox_face_needs_is_named() {
        assert_eq!(
            first_hit(&["Players/03 - A/face_diff.bin", "Players/03 - A/skin.dds"]),
            what("Players/03 - A")
        );
        assert_eq!(
            first_hit(&["Players/03 - A/face_high.fmdl"]),
            missing("Players/03 - A/face_diff.bin")
        );
        assert_eq!(
            gate(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/fcl_hair.skl"
            ]),
            missing("Players/03 - A/fcl_hair_sim.fclo")
        );
        assert_eq!(
            gate(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/fcl_hair_sim.fclo"
            ]),
            missing("Players/03 - A/fcl_hair.skl")
        );
        // Without `fcl_hair.fmdl` neither hair file is needed, and the simulation still packs.
        assert_eq!(gate(&["Players/03 - A/fcl_hair_sim.fclo"]), None);
    }

    #[test]
    fn shared_folders_are_named_by_their_files() {
        for (link, file) in [
            ("Players/03 - A/Round.face", "Faces/Round/face_high.fmdl"),
            ("Players/03 - A/Crocs.boots", "Boots/Crocs/boots.fmdl"),
            ("Players/03 - A/Grip.gloves", "Gloves/Grip/glove_l.fmdl"),
        ] {
            assert_eq!(gate(&[link, file]), what(file), "{file}");
        }
    }

    #[test]
    fn a_kit_drawn_for_the_other_engine_is_named() {
        assert_eq!(
            gate(&["Kits/g1/kit.dds", "Kits/g1/pre-fox"]),
            what("Kits/g1")
        );
    }

    #[test]
    fn a_kit_texture_the_config_has_no_field_for_or_in_another_format_is_named() {
        for file in ["Kits/g1/kit_srm.dds", "Kits/g1/kit_back.png"] {
            assert_eq!(gate(&["Kits/g1/kit.dds", file]), what(file), "{file}");
        }
    }

    #[test]
    fn portraits_logo_collars_and_common_are_named() {
        for file in [
            "Portraits/player_03.dds",
            "logo.dds",
            "Collars/collar.dds",
            "Common/skin.dds",
        ] {
            assert_eq!(gate(&[file]), what(file), "{file}");
        }
    }

    #[test]
    fn the_first_hit_follows_players_files_missing_files_shared_folders_kits_then_the_rest() {
        let face_high = "Players/03 - A/face_high.fmdl";
        let boots = "Players/03 - A/boots.fmdl";
        let link = "Players/03 - A/Crocs.boots";
        let shared = "Boots/Crocs/boots.fmdl";
        let kit = "Kits/g1/kit.png";
        let portrait = "Portraits/player_03.dds";
        // A folder's own files come before what it lacks: `face_diff.bin` is missing too.
        assert_eq!(
            first_hit(&[face_high, boots, link, shared, kit, portrait]),
            what(boots)
        );
        assert_eq!(
            first_hit(&[face_high, link, shared, kit, portrait]),
            missing("Players/03 - A/face_diff.bin")
        );
        assert_eq!(gate(&[link, shared, kit, portrait]), what(shared));
        assert_eq!(gate(&[kit, portrait]), what(kit));
        assert_eq!(gate(&[portrait]), what(portrait));
    }

    #[test]
    fn what_is_never_emitted_is_not_counted() {
        // `fpc.off` and `settings.toml` go to the savefile; a kit's colors and icon and the
        // root's metadata emit nothing yet.
        assert_eq!(
            gate(&[
                "Players/03 - A/fpc.off",
                "Players/03 - A/settings.toml",
                "Kits/g1/kit.dds",
                "Kits/g1/colors.txt",
                "colors.txt",
            ]),
            None
        );

        // A folder no roster slot maps, and an `all/` no kit inherits from, emit nothing.
        let (export, issues) = resolved_with_issues(
            "co - Gate",
            &[
                ("Players/A/face_high.fmdl", 1),
                ("Players/A/face_diff.bin", 1),
                ("Players/Unlisted/boots.fmdl", 1),
                ("Kits/all/kit_back.png", 1),
            ],
            &[],
            Some(b"03 A\n"),
        );
        assert_eq!(issues, ["player_unlisted", "kit_all_unused"]);
        assert_eq!(first_not_compiled(&export, PesVersion::Pes21), None);
    }

    #[test]
    fn a_kit_mask_is_dropped_on_a_fox_target_before_the_gate_and_the_task() {
        let files: Vec<(&str, u64)> = FACE
            .iter()
            .chain(&[
                "Kits/g1/config.toml",
                "Kits/g1/kit.dds",
                "Kits/g1/kit_mask.dds",
            ])
            .map(|path| (*path, 1))
            .collect();
        let export = resolved("co - Gate", &files, &[], None);

        let report = plan_run(vec![(ExportId(0), export)], PesVersion::Pes21);

        assert!(report.messages.is_empty(), "{:?}", report.messages);
        let TaskKind::Kit { kit, .. } = &report.manifest.tasks[1].kind else {
            panic!("the second task is the kit's");
        };
        let stems: Vec<&str> = kit
            .textures
            .iter()
            .map(|texture| texture.stem.as_str())
            .collect();
        assert_eq!(stems, ["kit"]);
        assert_eq!(
            report.manifest.tasks[1].charge, 2,
            "the config and the kit are read, the mask is not"
        );
    }

    /// The role of the file at `relative` in `Players/03 - A`, a folder that holds
    /// `fcl_hair.fmdl` when `holds_hair_model`.
    fn role_in(relative: &str, holds_hair_model: bool) -> Option<FaceFile> {
        let path = ScopePath::new(relative).unwrap();
        let file = FileDescriptor {
            size: 0,
            kind: aesthetics_export::classify(path.name()),
            source: path.clone(),
            path,
        };
        face_file(
            &ScopePath::new("Players/03 - A").unwrap(),
            &file,
            holds_hair_model,
        )
    }

    /// The role of the file at `relative` in a folder that holds `fcl_hair.fmdl`.
    fn role(relative: &str) -> Option<FaceFile> {
        role_in(relative, true)
    }

    #[test]
    fn each_face_file_has_its_role_or_none() {
        for stem in FACE_STEMS {
            assert_eq!(
                role(&format!("Players/03 - A/{stem}.fmdl")),
                Some(FaceFile::Model(stem.to_owned()))
            );
        }
        assert_eq!(
            role("Players/03 - A/skin.dds"),
            Some(FaceFile::Texture("skin".to_owned(), TextureFormat::Dds))
        );
        assert_eq!(
            role("Players/03 - A/skin.ftex"),
            Some(FaceFile::Texture("skin".to_owned(), TextureFormat::Ftex))
        );
        assert_eq!(
            role("Players/03 - A/fcl_hair.skl"),
            Some(FaceFile::Packed("fcl_hair_sim.skl"))
        );
        assert_eq!(role_in("Players/03 - A/fcl_hair.skl", false), None);
        assert_eq!(
            role("Players/03 - A/face_diff.bin"),
            Some(FaceFile::Packed("face_diff.bin"))
        );
        assert_eq!(
            role("Players/03 - A/fcl_hair_sim.fclo"),
            Some(FaceFile::Packed("fcl_hair_sim.fclo"))
        );
        for refused in [
            "Players/03 - A/torso.fmdl",
            "Players/03 - A/boots.fmdl",
            "Players/03 - A/face_high.model",
            "Players/03 - A/boots.skl",
            "Players/03 - A/face.xml",
            "Players/03 - A/skin.png",
            "Players/03 - A/hair.png.common",
            "Players/03 - A/face_diff2.bin",
            "Players/03 - A/Face_Diff.bin",
            "Players/03 - A/fcl_hair.fclo",
            "Players/03 - A/face/face_high.fmdl",
        ] {
            assert_eq!(role(refused), None, "{refused}");
        }
    }

    #[test]
    fn a_texture_format_is_its_extension_in_any_case() {
        assert_eq!(texture_format("kit.dds"), Some(TextureFormat::Dds));
        assert_eq!(texture_format("kit.DDS"), Some(TextureFormat::Dds));
        assert_eq!(texture_format("kit.FTEX"), Some(TextureFormat::Ftex));
        for refused in ["kit.png", "kit.tga", "kit", "kit.dds.png"] {
            assert_eq!(texture_format(refused), None, "{refused}");
        }
    }

    #[test]
    fn a_stem_is_the_name_up_to_its_last_dot() {
        assert_eq!(file_stem("hair.png.common"), "hair.png");
        assert_eq!(file_stem("shirt.dds"), "shirt");
        assert_eq!(file_stem("NO_USE"), "NO_USE");
    }
}
