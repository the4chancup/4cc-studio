//! What `compile` builds so far, and the gate that skips any export holding anything else
//! (`team_compiler/README.md` "Phase 3 scope", narrowed step by step through Phase 4): one
//! classification, which planning reads to skip an export and processing reads for each
//! file's role, so the two never disagree. The gate is withdrawn when `compile` builds
//! everything an export holds; the classification stays.

use aesthetics_export::{
    ExportIdentity, FileDescriptor, FileKind, FpcDirective, KitLayout, ModelFormat, ModelSuffix,
    PlayerFolder, PlayerIndex, PlayerSlot, ResolvedAestheticsExport, ValidatedAestheticsExport,
    ValidatedRoster, model_suffix,
};
use pes_version::{Engine, PesVersion};
use vtree::ScopePath;

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

/// The three Fox packages a player folder's models go to, each a `.fpk` and `.fpkd` pair in
/// its own game folder: the face by player id, the boots and the gloves by the player's
/// planned model id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ModelPackage {
    /// `face/real/{player id}/#Win/face.fpk`.
    Face,
    /// `boots/k{id}/#Win/boots.fpk`.
    Boots,
    /// `glove/g{id}/#Win/glove.fpk`.
    Gloves,
}

impl ModelPackage {
    /// The three packages in a folder's canonical order: the face, the boots, the gloves.
    pub(crate) const ALL: [ModelPackage; 3] = [
        ModelPackage::Face,
        ModelPackage::Boots,
        ModelPackage::Gloves,
    ];

    /// The package's file stem: `face.fpk`, `boots.fpk`, `glove.fpk`.
    pub(crate) fn file_stem(self) -> &'static str {
        match self {
            ModelPackage::Face => "face",
            ModelPackage::Boots => "boots",
            ModelPackage::Gloves => "glove",
        }
    }
}

/// What one file of a player folder becomes in the Fox output.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PlayerFile {
    /// A model, packed into `package` as `<name>.fmdl` with its texture paths rewritten:
    /// `name` is its allowed name, the free prefix stripped (`kit_boots.fmdl` packs as
    /// `boots.fmdl`).
    Model {
        package: ModelPackage,
        name: &'static str,
    },
    /// A file packed into `package` as it is, under `name`: the face's `face_diff.bin` and
    /// `fcl_hair_sim.fclo`, and a model's skeleton under its slot's name.
    Packed {
        package: ModelPackage,
        name: &'static str,
    },
    /// A texture with this stem and format, converted once into the player's common folder.
    Texture(String, TextureFormat),
}

impl PlayerFile {
    /// The package the file goes into; `None` for a texture, which goes to the player's
    /// common folder for every package to point at.
    pub(crate) fn package(&self) -> Option<ModelPackage> {
        match self {
            PlayerFile::Model { package, .. } | PlayerFile::Packed { package, .. } => {
                Some(*package)
            }
            PlayerFile::Texture(..) => None,
        }
    }
}

/// What a player folder's models say about its other files: which have a package to go in,
/// and which model a `.skl` pairs with. Computed once per folder and passed to `player_file`.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct FolderModels {
    /// The folder holds a face model, so its face files have a package to go in.
    face: bool,
    /// The stem of its `fcl_hair` model, whose skeleton packs as `fcl_hair_sim.skl`.
    hair_stem: Option<String>,
    /// The stem of its boots model, whose skeleton packs as `boots.skl`. `face_high`,
    /// `hair_high`, `oral` and the gloves have no skeleton slot.
    boots_stem: Option<String>,
}

impl FolderModels {
    /// The models of `folder`, as its direct `.fmdl` files with a recognized suffix.
    pub(crate) fn of(folder: &PlayerFolder) -> FolderModels {
        let mut models = FolderModels::default();
        for file in &folder.files {
            if file.path.parent().as_ref() != Some(&folder.path)
                || file.kind != FileKind::Model(ModelFormat::Fmdl)
            {
                continue;
            }
            let stem = file_stem(file.path.name());
            match model_suffix(stem) {
                Some(ModelSuffix::FclHair) => {
                    models.face = true;
                    models.hair_stem.get_or_insert_with(|| stem.to_owned());
                }
                Some(ModelSuffix::FaceHigh | ModelSuffix::HairHigh | ModelSuffix::Oral) => {
                    models.face = true;
                }
                Some(ModelSuffix::Boots) => {
                    models.boots_stem.get_or_insert_with(|| stem.to_owned());
                }
                Some(
                    ModelSuffix::GloveL
                    | ModelSuffix::GloveR
                    | ModelSuffix::HandL
                    | ModelSuffix::HandR,
                )
                | None => {}
            }
        }
        models
    }
}

/// What `file` of the player folder at `folder`, whose models are `models`, becomes in the
/// Fox output; `None` when it has no role `compile` builds yet.
pub(crate) fn player_file(
    folder: &ScopePath,
    file: &FileDescriptor,
    models: &FolderModels,
) -> Option<PlayerFile> {
    if file.path.parent().as_ref() != Some(folder) {
        return None;
    }
    let name = file.path.name();
    let stem = file_stem(name);
    let face_packed = |name| {
        models.face.then_some(PlayerFile::Packed {
            package: ModelPackage::Face,
            name,
        })
    };
    match file.kind {
        // A model with no recognized suffix is face content the `fcl_hair` merge takes (step
        // 4.5), so it has no role yet.
        FileKind::Model(ModelFormat::Fmdl) => {
            let (package, name) = match model_suffix(stem)? {
                ModelSuffix::FaceHigh => (ModelPackage::Face, "face_high"),
                ModelSuffix::HairHigh => (ModelPackage::Face, "hair_high"),
                ModelSuffix::Oral => (ModelPackage::Face, "oral"),
                ModelSuffix::FclHair => (ModelPackage::Face, "fcl_hair"),
                ModelSuffix::Boots => (ModelPackage::Boots, "boots"),
                // Hand-skeleton models have nowhere else to go on Fox.
                ModelSuffix::GloveL | ModelSuffix::HandL => (ModelPackage::Gloves, "glove_l"),
                ModelSuffix::GloveR | ModelSuffix::HandR => (ModelPackage::Gloves, "glove_r"),
            };
            Some(PlayerFile::Model { package, name })
        }
        FileKind::Texture => {
            texture_format(name).map(|format| PlayerFile::Texture(stem.to_owned(), format))
        }
        // The game loads a skeleton under its slot's name; the export names it after the
        // model it pairs with, and without that model the skeleton has nothing to drive.
        FileKind::Skl if models.hair_stem.as_deref() == Some(stem) => Some(PlayerFile::Packed {
            package: ModelPackage::Face,
            name: "fcl_hair_sim.skl",
        }),
        FileKind::Skl if models.boots_stem.as_deref() == Some(stem) => Some(PlayerFile::Packed {
            package: ModelPackage::Boots,
            name: "boots.skl",
        }),
        FileKind::Bin if name == "face_diff.bin" => face_packed("face_diff.bin"),
        FileKind::Fclo if name == "fcl_hair_sim.fclo" => face_packed("fcl_hair_sim.fclo"),
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
/// file a Fox face needs that its folder lacks). `None` when `compile` builds all of it.
/// Planning drops a Fox target's `kit_mask` before asking: the gate would count it.
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
            && let Some(item) = player_not_compiled(folder)
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
    for (slot, file) in &export.portraits {
        // A slot with a portrait from both sources is refused by naming this file: whether
        // the two agree is the deep pass's `portrait_conflict`, which compares their bytes.
        if !is_dds(file) || folder_holds_portrait(export, *slot) {
            return Some(what_entry(file));
        }
    }
    // A small logo comes only with a main one, which is named first, so the small one is not
    // walked.
    let mut rest = export
        .logo
        .as_ref()
        .map(|logo| &logo.main.file)
        .into_iter()
        .chain(&export.collars)
        .chain(&export.common);
    rest.next().map(what_entry)
}

/// Whether `file` is a `.dds`, the one portrait format emitted as it is; every other image
/// format, `.ftex` included, waits for its conversion.
fn is_dds(file: &FileDescriptor) -> bool {
    texture_format(file.path.name()) == Some(TextureFormat::Dds)
}

/// Whether the player folder `slot` maps in `export`, if one does, holds a `portrait.*`.
fn folder_holds_portrait(export: &ValidatedAestheticsExport, slot: PlayerSlot) -> bool {
    // A referee roster never reaches here: the gate names `refs` before any file.
    let ValidatedRoster::Team(slots) = &export.roster else {
        return false;
    };
    slots
        .get(&slot)
        .and_then(|index| export.players.get(index.0))
        .is_some_and(|folder| folder.portrait.is_some())
}

/// The first thing in the mapped player `folder` that `compile` cannot build into its Fox
/// packages yet.
fn player_not_compiled(folder: &PlayerFolder) -> Option<(&'static str, String)> {
    let models = FolderModels::of(folder);
    let mut roles = Vec::new();
    // Every file the folder holds is checked before any file it lacks: a file with no role
    // may be the missing one, misnamed (`Face_Diff.bin`).
    for file in &folder.files {
        let Some(role) = player_file(&folder.path, file, &models) else {
            return Some(what_entry(file));
        };
        // A second file packing under one name (`boots.fmdl` beside `kit_boots.fmdl`) is a
        // merge, which step 4.5 builds.
        if roles.contains(&role) {
            return Some(what_entry(file));
        }
        roles.push(role);
    }
    let path = folder.path.as_str();
    if let Some(portrait) = &folder.portrait
        && !is_dds(portrait)
    {
        return Some(what_entry(portrait));
    }
    if folder.ingame_face {
        return Some(("what", format!("{path}/ingame_face")));
    }
    // `fpc.on` changes every kit config; `fpc.off` only reaches the savefile.
    if folder.fpc == Some(FpcDirective::On) {
        return Some(("what", format!("{path}/fpc.on")));
    }
    if !roles
        .iter()
        .any(|role| matches!(role, PlayerFile::Model { .. }))
    {
        return Some(("what", path.to_owned()));
    }
    // The files a Fox face needs beside its models, which Phase 4 injects when they are missing.
    let face_packed = |name| PlayerFile::Packed {
        package: ModelPackage::Face,
        name,
    };
    if models.face && !roles.contains(&face_packed("face_diff.bin")) {
        return Some(("missing", format!("{path}/face_diff.bin")));
    }
    if let Some(hair_stem) = &models.hair_stem {
        if !roles.contains(&face_packed("fcl_hair_sim.fclo")) {
            return Some(("missing", format!("{path}/fcl_hair_sim.fclo")));
        }
        // The skeleton named after the hair model is packed as `fcl_hair_sim.skl`.
        if !roles.contains(&face_packed("fcl_hair_sim.skl")) {
            return Some(("missing", format!("{path}/{hair_stem}.skl")));
        }
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
    fn fox_player_folders_dds_portraits_and_kits_are_compiled() {
        assert_eq!(gate(&[]), None);
        assert_eq!(
            gate(&[
                "Players/03 - A/hair_high.fmdl",
                "Players/03 - A/oral.fmdl",
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/fcl_hair.skl",
                "Players/03 - A/fcl_hair_sim.fclo",
                "Players/03 - A/kit_boots.fmdl",
                "Players/03 - A/kit_boots.skl",
                "Players/03 - A/glove_l.fmdl",
                "Players/03 - A/keeper_gloveR.fmdl",
                "Players/03 - A/skin.dds",
                "Players/03 - A/hair.FTEX",
                "Players/03 - A/portrait.DDS",
                "Portraits/player_07.dds",
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
        // A folder needs a model, not a face: boots alone compile, with or without their
        // skeleton, and so do gloves.
        assert_eq!(
            first_hit(&["Players/03 - A/boots.fmdl", "Players/03 - A/skin.dds"]),
            None
        );
        assert_eq!(first_hit(&["Players/03 - A/handL.fmdl"]), None);
    }

    #[test]
    fn two_files_packing_under_one_name_name_the_second() {
        // The second in the folder's order, which lists its files sorted.
        for (first, second) in [
            ("boots.fmdl", "kit_boots.fmdl"),
            ("glove_l.fmdl", "gloveL.fmdl"),
            ("glove_r.fmdl", "handR.fmdl"),
            ("face_high.fmdl", "x_face_high.fmdl"),
        ] {
            assert_eq!(
                first_hit(&[
                    &format!("Players/03 - A/{first}"),
                    &format!("Players/03 - A/{second}"),
                ]),
                what(&format!("Players/03 - A/{second}")),
                "{first} + {second}"
            );
        }
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
            "Players/03 - A/torso.fmdl",
            "Players/03 - A/hair.png",
            "Players/03 - A/face/hair_high.fmdl",
        ] {
            assert_eq!(gate(&[file]), what(file), "{file}");
        }
    }

    #[test]
    fn a_skeleton_without_the_model_of_its_name_is_named() {
        assert_eq!(
            gate(&["Players/03 - A/fcl_hair.skl"]),
            what("Players/03 - A/fcl_hair.skl")
        );
        // `boots.skl` beside `kit_boots.fmdl` pairs with nothing: the skeleton is named
        // after the model's source name.
        assert_eq!(
            gate(&["Players/03 - A/kit_boots.fmdl", "Players/03 - A/boots.skl"]),
            what("Players/03 - A/boots.skl")
        );
        // The gloves have no skeleton slot.
        assert_eq!(
            gate(&["Players/03 - A/glove_l.fmdl", "Players/03 - A/glove_l.skl"]),
            what("Players/03 - A/glove_l.skl")
        );
    }

    #[test]
    fn a_portrait_in_another_format_than_dds_is_named() {
        for file in [
            "Players/03 - A/portrait.png",
            "Players/03 - A/portrait.ftex",
            "Portraits/player_03.png",
            "Portraits/player_07.ftex",
        ] {
            assert_eq!(gate(&[file]), what(file), "{file}");
        }
    }

    #[test]
    fn a_slot_with_a_portrait_from_both_sources_is_named_by_its_portraits_file() {
        assert_eq!(
            gate(&["Players/03 - A/portrait.dds", "Portraits/player_03.dds"]),
            what("Portraits/player_03.dds")
        );
        // Another slot's file, mapped to a folder without a portrait or to no folder, is no
        // conflict.
        assert_eq!(
            gate(&[
                "Players/03 - A/portrait.dds",
                "Players/07 - B/face_high.fmdl",
                "Players/07 - B/face_diff.bin",
                "Portraits/player_07.dds",
                "Portraits/player_09.dds",
            ]),
            None
        );
        // The folder's portrait stands for every slot mapping the folder.
        let export = resolved(
            "co - Gate",
            &[
                ("Players/A/face_high.fmdl", 1),
                ("Players/A/face_diff.bin", 1),
                ("Players/A/portrait.dds", 1),
                ("Portraits/player_07.dds", 1),
            ],
            &[],
            Some(b"03 A\n07 A\n"),
        );
        assert_eq!(
            first_not_compiled(&export, PesVersion::Pes21),
            what("Portraits/player_07.dds")
        );
    }

    #[test]
    fn ingame_face_or_fpc_on_is_named() {
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
    fn a_folder_without_a_model_or_a_file_a_fox_face_needs_is_named() {
        // A face file without a face model has no package to go in, so it is named first;
        // a folder of textures alone is named as a whole.
        assert_eq!(
            first_hit(&["Players/03 - A/face_diff.bin", "Players/03 - A/skin.dds"]),
            what("Players/03 - A/face_diff.bin")
        );
        assert_eq!(
            first_hit(&["Players/03 - A/skin.dds"]),
            what("Players/03 - A")
        );
        assert_eq!(
            first_hit(&["Players/03 - A/face_high.fmdl"]),
            missing("Players/03 - A/face_diff.bin")
        );
        // The boots need no face files.
        assert_eq!(
            first_hit(&["Players/03 - A/boots.fmdl", "Players/03 - A/face_diff.bin"]),
            what("Players/03 - A/face_diff.bin")
        );
        // The hair skeleton missing is named by the model's source name.
        assert_eq!(
            first_hit(&[
                "Players/03 - A/x_fcl_hair.fmdl",
                "Players/03 - A/face_diff.bin",
                "Players/03 - A/fcl_hair_sim.fclo"
            ]),
            missing("Players/03 - A/x_fcl_hair.skl")
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
    fn logo_collars_and_common_are_named() {
        for file in ["logo.dds", "Collars/collar.dds", "Common/skin.dds"] {
            assert_eq!(gate(&[file]), what(file), "{file}");
        }
    }

    #[test]
    fn the_first_hit_follows_players_files_missing_files_shared_folders_kits_then_the_rest() {
        let face_high = "Players/03 - A/face_high.fmdl";
        let torso = "Players/03 - A/torso.fmdl";
        let link = "Players/03 - A/Crocs.boots";
        let shared = "Boots/Crocs/boots.fmdl";
        let kit = "Kits/g1/kit.png";
        let portrait = "Portraits/player_03.png";
        // A folder's own files come before what it lacks: `face_diff.bin` is missing too.
        assert_eq!(
            first_hit(&[face_high, torso, link, shared, kit, portrait]),
            what(torso)
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
                ("Players/Unlisted/torso.fmdl", 1),
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

    /// The folder `Players/03 - A` holding the files `names`.
    fn folder(names: &[&str]) -> PlayerFolder {
        let files = names
            .iter()
            .map(|name| {
                let path = ScopePath::new(&format!("Players/03 - A/{name}")).unwrap();
                FileDescriptor {
                    size: 0,
                    kind: aesthetics_export::classify(path.name()),
                    source: path.clone(),
                    path,
                }
            })
            .collect();
        PlayerFolder {
            path: ScopePath::new("Players/03 - A").unwrap(),
            player_name: "A".to_owned(),
            files,
            links: Vec::new(),
            ingame_face: false,
            fpc: None,
            portrait: None,
            settings: None,
        }
    }

    /// The role of each file of `names` in `Players/03 - A`.
    fn roles(names: &[&str]) -> Vec<Option<PlayerFile>> {
        let folder = folder(names);
        let models = FolderModels::of(&folder);
        folder
            .files
            .iter()
            .map(|file| player_file(&folder.path, file, &models))
            .collect()
    }

    /// The role of the file `name` in a folder holding it beside `fcl_hair.fmdl` and
    /// `kit_boots.fmdl`.
    fn role(name: &str) -> Option<PlayerFile> {
        roles(&["fcl_hair.fmdl", "kit_boots.fmdl", name])
            .pop()
            .unwrap()
    }

    fn model(package: ModelPackage, name: &'static str) -> Option<PlayerFile> {
        Some(PlayerFile::Model { package, name })
    }

    fn packed(package: ModelPackage, name: &'static str) -> Option<PlayerFile> {
        Some(PlayerFile::Packed { package, name })
    }

    #[test]
    fn each_model_goes_to_its_package_under_its_allowed_name() {
        for (name, package, allowed) in [
            ("face_high.fmdl", ModelPackage::Face, "face_high"),
            ("hair_high.fmdl", ModelPackage::Face, "hair_high"),
            ("oral.fmdl", ModelPackage::Face, "oral"),
            ("fcl_hair.fmdl", ModelPackage::Face, "fcl_hair"),
            ("x_fcl_hair.fmdl", ModelPackage::Face, "fcl_hair"),
            ("boots.fmdl", ModelPackage::Boots, "boots"),
            ("kit_boots.fmdl", ModelPackage::Boots, "boots"),
            ("Kit_BOOTS.fmdl", ModelPackage::Boots, "boots"),
            ("glove_l.fmdl", ModelPackage::Gloves, "glove_l"),
            ("gloveL.fmdl", ModelPackage::Gloves, "glove_l"),
            ("keeper_gloveR.fmdl", ModelPackage::Gloves, "glove_r"),
            ("glove_r.fmdl", ModelPackage::Gloves, "glove_r"),
            ("handL.fmdl", ModelPackage::Gloves, "glove_l"),
            ("handR.fmdl", ModelPackage::Gloves, "glove_r"),
        ] {
            assert_eq!(roles(&[name]), [model(package, allowed)], "{name}");
        }
    }

    #[test]
    fn each_other_player_file_has_its_role_or_none() {
        assert_eq!(
            role("skin.dds"),
            Some(PlayerFile::Texture("skin".to_owned(), TextureFormat::Dds))
        );
        assert_eq!(
            role("skin.ftex"),
            Some(PlayerFile::Texture("skin".to_owned(), TextureFormat::Ftex))
        );
        assert_eq!(
            role("fcl_hair.skl"),
            packed(ModelPackage::Face, "fcl_hair_sim.skl")
        );
        assert_eq!(
            role("kit_boots.skl"),
            packed(ModelPackage::Boots, "boots.skl")
        );
        assert_eq!(
            role("face_diff.bin"),
            packed(ModelPackage::Face, "face_diff.bin")
        );
        assert_eq!(
            role("fcl_hair_sim.fclo"),
            packed(ModelPackage::Face, "fcl_hair_sim.fclo")
        );
        for refused in [
            "torso.fmdl",
            "myboots.fmdl",
            "face_high.model",
            "boots.skl",
            "glove_l.skl",
            "face.xml",
            "skin.png",
            "hair.png.common",
            "face_diff2.bin",
            "Face_Diff.bin",
            "fcl_hair.fclo",
            "face/face_high.fmdl",
        ] {
            assert_eq!(role(refused), None, "{refused}");
        }
    }

    #[test]
    fn a_skeleton_pairs_with_the_model_of_its_name_and_face_files_need_a_face_model() {
        // The skeleton is named after the model, whatever the model's free prefix.
        assert_eq!(
            roles(&["x_fcl_hair.fmdl", "x_fcl_hair.skl", "fcl_hair.skl"]),
            [
                model(ModelPackage::Face, "fcl_hair"),
                packed(ModelPackage::Face, "fcl_hair_sim.skl"),
                None
            ]
        );
        assert_eq!(
            roles(&["boots.fmdl", "boots.skl", "kit_boots.skl"]),
            [
                model(ModelPackage::Boots, "boots"),
                packed(ModelPackage::Boots, "boots.skl"),
                None
            ]
        );
        // Without a face model the face's files have no package to go in.
        assert_eq!(
            roles(&["boots.fmdl", "face_diff.bin", "fcl_hair_sim.fclo"]),
            [model(ModelPackage::Boots, "boots"), None, None]
        );
        assert_eq!(
            roles(&["face_high.fmdl", "face_diff.bin", "fcl_hair_sim.fclo"]),
            [
                model(ModelPackage::Face, "face_high"),
                packed(ModelPackage::Face, "face_diff.bin"),
                packed(ModelPackage::Face, "fcl_hair_sim.fclo")
            ]
        );
    }

    #[test]
    fn a_package_is_the_models_and_packed_files_but_not_the_textures() {
        let texture = PlayerFile::Texture("skin".to_owned(), TextureFormat::Dds);
        assert_eq!(texture.package(), None);
        assert_eq!(
            packed(ModelPackage::Boots, "boots.skl").unwrap().package(),
            Some(ModelPackage::Boots)
        );
        assert_eq!(
            model(ModelPackage::Gloves, "glove_l").unwrap().package(),
            Some(ModelPackage::Gloves)
        );
        assert_eq!(
            ModelPackage::ALL.map(ModelPackage::file_stem),
            ["face", "boots", "glove"]
        );
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
