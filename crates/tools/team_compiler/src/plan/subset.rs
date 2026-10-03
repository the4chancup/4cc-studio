//! What `compile` builds so far, and the gate that skips any export holding anything else
//! (`team_compiler/README.md` "Phase 3 scope", narrowed step by step through Phase 4): one
//! classification, which planning reads to skip an export and processing reads for each
//! file's role, so the two never disagree. The gate is withdrawn when `compile` builds
//! everything an export holds; the classification stays.

use aesthetics_export::{
    ExportIdentity, FileDescriptor, FileKind, FpcDirective, KitLayout, ModelFormat, ModelSuffix,
    PlayerFolder, PlayerSlot, ResolvedAestheticsExport, SharedKind, SharedLink, SharedModelFolder,
    ValidatedAestheticsExport, ValidatedRoster, model_suffix,
};
use pes_version::{Engine, PesVersion};
use vtree::ScopePath;

use super::ids::shared_folders_taking_ids;
use super::mapped_players;

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

    /// The package as a finding names it (`shared_texture_conflict`'s `dropped`): `face`,
    /// `boots`, `gloves`.
    pub(crate) fn name(self) -> &'static str {
        match self {
            ModelPackage::Face => "face",
            ModelPackage::Boots => "boots",
            ModelPackage::Gloves => "gloves",
        }
    }
}

/// The package a shared folder of `kind` is loaded as: `Boots/` holds boots models, `Gloves/`
/// gloves models, `Faces/` face parts.
pub(crate) fn package_of(kind: SharedKind) -> ModelPackage {
    match kind {
        SharedKind::Face => ModelPackage::Face,
        SharedKind::Boots => ModelPackage::Boots,
        SharedKind::Gloves => ModelPackage::Gloves,
    }
}

/// The link file for the shared folder `name` of `kind`, as the export spells it without the
/// tolerated `.txt` tail (`Crocs.boots`).
pub(crate) fn link_name(kind: SharedKind, name: &str) -> String {
    let extension = match kind {
        SharedKind::Face => "face",
        SharedKind::Boots => "boots",
        SharedKind::Gloves => "gloves",
    };
    format!("{name}.{extension}")
}

/// Whether `player`'s `link` combines on Fox: the shared folder's models become parts of the
/// player's own package instead of the shared output being loaded as it is. A face link
/// always does, a shared face having no output of its own; a boots or gloves link does when
/// the player holds a model of that package (`player_folders.md` "A link plus local models
/// combines"). Pre-Fox has no exclusive packages, so there every link is plain.
pub(crate) fn link_combines(player: &PlayerFolder, link: &SharedLink) -> bool {
    match link.kind {
        SharedKind::Face => true,
        SharedKind::Boots | SharedKind::Gloves => {
            holds_model(&player.path, &player.files, package_of(link.kind))
        }
    }
}

/// The shared folder `link` names in `export`, matched as validation matched it: by
/// case-folded name. `None` only for a link validation did not resolve, and it drops the
/// folder of such a link.
pub(crate) fn linked_folder<'a>(
    export: &'a ValidatedAestheticsExport,
    link: &SharedLink,
) -> Option<&'a SharedModelFolder> {
    let folders = match link.kind {
        SharedKind::Face => &export.faces,
        SharedKind::Boots => &export.boots,
        SharedKind::Gloves => &export.gloves,
    };
    let name_key = vtree::fold_name(&link.name);
    folders
        .iter()
        .find(|folder| vtree::fold_name(&folder.folder_name) == name_key)
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
    /// A face file packed into `package` as it is, under `name`: `face_diff.bin` and
    /// `fcl_hair_sim.fclo`. Not a part to merge: the package holds one, the player folder's
    /// own over a combined face folder's (`player_folders.md` "A link plus local models
    /// combines").
    Packed {
        package: ModelPackage,
        name: &'static str,
    },
    /// A model's skeleton, the `.skl` named after a `fcl_hair` or `boots` part, packed into
    /// `package` under its slot's `name` (`fcl_hair_sim.skl`, `boots.skl`). Every part may
    /// bring one, and the merge decides whose is packed (`player_folders.md` "Merge
    /// constraint").
    Skeleton {
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
            PlayerFile::Model { package, .. }
            | PlayerFile::Packed { package, .. }
            | PlayerFile::Skeleton { package, .. } => Some(*package),
            PlayerFile::Texture(..) => None,
        }
    }
}

/// What a player folder's models say about its other files: which have a package to go in,
/// and which model a `.skl` pairs with. Computed once per folder and passed to `player_file`.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct FolderModels {
    /// The folder's face files have a package to go in: it holds a face model, or links a
    /// shared face (`with_linked_face`).
    face: bool,
    /// The stems of its `fcl_hair` models, each of whose skeletons packs as
    /// `fcl_hair_sim.skl` (several parts merge into one `fcl_hair.fmdl`, and the merge decides
    /// whose skeleton that is).
    hair_stems: Vec<String>,
    /// The stems of its boots models, each of whose skeletons packs as `boots.skl`, by the
    /// same rule. `face_high`, `hair_high`, `oral` and the gloves have no skeleton slot.
    boots_stems: Vec<String>,
}

impl FolderModels {
    /// The models among `files` of the folder at `folder`: its direct `.fmdl` files with a
    /// recognized suffix.
    pub(crate) fn of(folder: &ScopePath, files: &[FileDescriptor]) -> FolderModels {
        let mut models = FolderModels::default();
        for file in files {
            if file.path.parent().as_ref() != Some(folder)
                || file.kind != FileKind::Model(ModelFormat::Fmdl)
            {
                continue;
            }
            let stem = file_stem(file.path.name());
            match model_suffix(stem) {
                Some(ModelSuffix::FclHair) => {
                    models.face = true;
                    models.hair_stems.push(stem.to_owned());
                }
                Some(ModelSuffix::FaceHigh | ModelSuffix::HairHigh | ModelSuffix::Oral) => {
                    models.face = true;
                }
                Some(ModelSuffix::Boots) => models.boots_stems.push(stem.to_owned()),
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

    /// The models of a player folder linking a shared face: the shared face is the player's
    /// face (a merge of one, `player_folders.md` "A link plus local models combines"), so the
    /// folder's `face_diff.bin` and `fcl_hair_sim.fclo` have a package to go in with no face
    /// model of the folder's own.
    pub(crate) fn with_linked_face(mut self) -> FolderModels {
        self.face = true;
        self
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
        FileKind::Skl if models.hair_stems.iter().any(|hair| hair == stem) => {
            Some(PlayerFile::Skeleton {
                package: ModelPackage::Face,
                name: "fcl_hair_sim.skl",
            })
        }
        FileKind::Skl if models.boots_stems.iter().any(|boots| boots == stem) => {
            Some(PlayerFile::Skeleton {
                package: ModelPackage::Boots,
                name: "boots.skl",
            })
        }
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

/// Whether the folder at `folder` holding `files` has a model that packs into `package`: on
/// Fox, what gives a player its own package of that kind.
pub(crate) fn holds_model(
    folder: &ScopePath,
    files: &[FileDescriptor],
    package: ModelPackage,
) -> bool {
    let models = FolderModels::of(folder, files);
    files.iter().any(|file| {
        matches!(
            player_file(folder, file, &models),
            Some(PlayerFile::Model { package: owner, .. }) if owner == package
        )
    })
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
    // A folder no roster slot maps is not compiled, so whatever it holds does not count.
    for folder in mapped_players(export) {
        if let Some(item) = player_not_compiled(export, folder) {
            return Some(item);
        }
    }
    // A shared boots or gloves folder compiles on its own when a mapped player links it
    // plainly; one no such player links has no output and is not counted. A shared face has
    // no output of its own and is walked above, as a source of each player linking it:
    // validation drops a shared folder no mapped player links, so every `Faces/` folder has
    // a linking player.
    for kind in [SharedKind::Boots, SharedKind::Gloves] {
        for folder in shared_folders_taking_ids(export, version.engine(), kind) {
            if let Some(item) = shared_not_compiled(kind, folder) {
                return Some(item);
            }
        }
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

/// The first thing in the mapped player `folder` of `export` that `compile` cannot build into
/// its Fox packages yet.
fn player_not_compiled(
    export: &ValidatedAestheticsExport,
    folder: &PlayerFolder,
) -> Option<(&'static str, String)> {
    let mut models = FolderModels::of(&folder.path, &folder.files);
    if folder
        .links
        .iter()
        .any(|link| matches!(link.kind, SharedKind::Face))
    {
        models = models.with_linked_face();
    }
    let mut roles = Vec::new();
    // Every file the folder holds is checked before any file it lacks: a file with no role
    // may be the missing one, misnamed (`Face_Diff.bin`).
    for file in &folder.files {
        let Some(role) = player_file(&folder.path, file, &models) else {
            return Some(what_entry(file));
        };
        roles.push(role);
    }
    let path = folder.path.as_str();
    // The hair's skeleton is named after the first hair model among the face's sources, the
    // player's own folder before a combined face folder.
    let mut hair_skeleton = models
        .hair_stems
        .first()
        .map(|stem| format!("{path}/{stem}.skl"));
    // A boots or gloves link alone loads the shared output as it is; one beside a local model
    // of its package combines, as every face link does: the shared folder's files become the
    // player's own, with the roles a shared folder's files have.
    for link in folder
        .links
        .iter()
        .filter(|link| link_combines(folder, link))
    {
        let shared = linked_folder(export, link)
            .expect("validation drops a player folder whose link names no shared folder");
        match shared_roles(link.kind, shared) {
            Ok(shared_roles) => roles.extend(shared_roles),
            Err(item) => return Some(item),
        }
        match link.kind {
            SharedKind::Face => {
                let shared_models = FolderModels::of(&shared.path, &shared.files);
                hair_skeleton = hair_skeleton.or_else(|| {
                    shared_models
                        .hair_stems
                        .first()
                        .map(|stem| format!("{}/{stem}.skl", shared.path.as_str()))
                });
            }
            SharedKind::Boots | SharedKind::Gloves => {}
        }
    }
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
    // A folder with no model of its own and no link has nothing to compile; one with only a
    // link is a player wearing a shared output.
    if folder.links.is_empty()
        && !roles
            .iter()
            .any(|role| matches!(role, PlayerFile::Model { .. }))
    {
        return Some(("what", path.to_owned()));
    }
    // The files a Fox face needs beside its models, which Phase 4 injects when they are
    // missing, from any of the face's sources: the player's own folder or a combined face
    // folder.
    let face_packed = |name| PlayerFile::Packed {
        package: ModelPackage::Face,
        name,
    };
    if models.face && !roles.contains(&face_packed("face_diff.bin")) {
        return Some(("missing", format!("{path}/face_diff.bin")));
    }
    if let Some(hair_skeleton) = hair_skeleton {
        if !roles.contains(&face_packed("fcl_hair_sim.fclo")) {
            return Some(("missing", format!("{path}/fcl_hair_sim.fclo")));
        }
        // A skeleton named after any hair part is packed as `fcl_hair_sim.skl`; when no part
        // brings one the face has no skeleton (its template is step 4.5's), and a part
        // without one beside a part with one is the merge's `skl_merge_conflict`.
        if !roles.iter().any(|role| {
            matches!(
                role,
                PlayerFile::Skeleton {
                    package: ModelPackage::Face,
                    ..
                }
            )
        }) {
            return Some(("missing", hair_skeleton));
        }
    }
    None
}

/// The first thing in the shared `folder` of `kind`, which takes a shared ID, that `compile`
/// cannot build into its own Fox package yet.
fn shared_not_compiled(
    kind: SharedKind,
    folder: &SharedModelFolder,
) -> Option<(&'static str, String)> {
    shared_roles(kind, folder).err()
}

/// The role of each file of the shared `folder` of `kind`, in file order, or the first file
/// `compile` cannot build: the files go by a player folder's roles (a `Faces/` folder's face
/// files and hair skeletons included), but only the package the folder is loaded as and its
/// textures have a place to go, and a folder with no model is named as a whole.
fn shared_roles(
    kind: SharedKind,
    folder: &SharedModelFolder,
) -> Result<Vec<PlayerFile>, (&'static str, String)> {
    let path = &folder.path;
    let package = package_of(kind);
    let models = FolderModels::of(path, &folder.files);
    let mut roles = Vec::new();
    for file in &folder.files {
        let Some(role) = player_file(path, file, &models) else {
            return Err(what_entry(file));
        };
        // A model of another package has no package here.
        if role.package().is_some_and(|owner| owner != package) {
            return Err(what_entry(file));
        }
        roles.push(role);
    }
    if !roles
        .iter()
        .any(|role| matches!(role, PlayerFile::Model { .. }))
    {
        return Err(("what", path.as_str().to_owned()));
    }
    Ok(roles)
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
        // So does a folder holding only a link to a shared folder.
        assert_eq!(
            first_hit(&["Players/03 - A/Crocs.boots", "Boots/Crocs/boots.fmdl"]),
            None
        );
    }

    #[test]
    fn parts_packing_under_one_name_merge_for_every_package() {
        assert_eq!(gate(&["Players/03 - A/x_face_high.fmdl"]), None);
        for (first, second) in [
            ("boots.fmdl", "kit_boots.fmdl"),
            ("glove_l.fmdl", "gloveL.fmdl"),
            ("glove_r.fmdl", "handR.fmdl"),
        ] {
            assert_eq!(
                first_hit(&[
                    &format!("Players/03 - A/{first}"),
                    &format!("Players/03 - A/{second}"),
                ]),
                None,
                "{first} + {second}"
            );
        }
        // Each boots part's skeleton, too, and each hair part's.
        assert_eq!(
            first_hit(&[
                "Players/03 - A/boots.fmdl",
                "Players/03 - A/boots.skl",
                "Players/03 - A/kit_boots.fmdl",
                "Players/03 - A/kit_boots.skl",
            ]),
            None
        );
        assert_eq!(
            gate(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/fcl_hair.skl",
                "Players/03 - A/x_fcl_hair.fmdl",
                "Players/03 - A/x_fcl_hair.skl",
                "Players/03 - A/fcl_hair_sim.fclo",
            ]),
            None
        );
        // One hair part's skeleton is enough for the gate: a part without one beside it is
        // the merge's conflict, not a missing file.
        assert_eq!(
            gate(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/x_fcl_hair.fmdl",
                "Players/03 - A/x_fcl_hair.skl",
                "Players/03 - A/fcl_hair_sim.fclo",
            ]),
            None
        );
        assert_eq!(
            gate(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/x_fcl_hair.fmdl",
                "Players/03 - A/fcl_hair_sim.fclo",
            ]),
            missing("Players/03 - A/fcl_hair.skl")
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
    fn a_shared_boots_or_gloves_folder_a_player_links_plainly_is_compiled() {
        // A player holding only the link, and one holding a face beside it.
        assert_eq!(
            first_hit(&[
                "Players/03 - A/Crocs.boots",
                "Players/03 - A/Grip.gloves",
                "Boots/Crocs/boots.fmdl",
                "Boots/Crocs/boots.skl",
                "Boots/Crocs/shirt.dds",
                "Gloves/Grip/glove_l.fmdl",
                "Gloves/Grip/gloveR.fmdl",
            ]),
            None
        );
        assert_eq!(
            gate(&["Players/03 - A/Crocs.boots", "Boots/Crocs/kit_boots.fmdl"]),
            None
        );
    }

    #[test]
    fn a_face_link_combines_and_a_boots_or_gloves_link_beside_a_local_model_of_its_kind_does() {
        for link in ["Round.face", "Round.face.txt"] {
            assert_eq!(
                gate(&[
                    &format!("Players/03 - A/{link}"),
                    "Faces/Round/face_high.fmdl"
                ]),
                None,
                "{link}"
            );
        }
        for (link, local, shared) in [
            ("Crocs.boots", "kit_boots.fmdl", "Boots/Crocs/boots.fmdl"),
            ("Grip.gloves", "handL.fmdl", "Gloves/Grip/glove_l.fmdl"),
        ] {
            assert_eq!(
                gate(&[
                    &format!("Players/03 - A/{link}"),
                    &format!("Players/03 - A/{local}"),
                    shared,
                ]),
                None,
                "{link}"
            );
        }
        // A local model of the other kind leaves the link plain.
        assert_eq!(
            gate(&[
                "Players/03 - A/Crocs.boots",
                "Players/03 - A/glove_l.fmdl",
                "Boots/Crocs/boots.fmdl",
            ]),
            None
        );
    }

    #[test]
    fn a_combined_shared_folder_s_files_are_walked_and_a_texture_stem_held_twice_is_compiled() {
        let combining = [
            "Players/03 - A/Crocs.boots",
            "Players/03 - A/kit_boots.fmdl",
        ];
        // What a plainly linked shared folder may hold, a combined one may too.
        assert_eq!(
            gate(&[
                combining[0],
                combining[1],
                "Boots/Crocs/boots.fmdl",
                "Boots/Crocs/boots.skl",
                "Boots/Crocs/kit_boots.fmdl",
                "Boots/Crocs/sole.dds",
            ]),
            None
        );
        // A file the shared folder cannot hold is named, and a folder with no model as a whole.
        assert_eq!(
            gate(&[
                combining[0],
                combining[1],
                "Boots/Crocs/boots.fmdl",
                "Boots/Crocs/sole.png",
            ]),
            what("Boots/Crocs/sole.png")
        );
        assert_eq!(
            gate(&[combining[0], combining[1], "Boots/Crocs/sole.dds"]),
            what("Boots/Crocs")
        );
        // A texture of a stem the player holds too, in any case or format, or one an earlier
        // combined folder holds, is compiled: whether the bytes agree is the textures task's
        // comparison.
        assert_eq!(
            gate(&[
                combining[0],
                combining[1],
                "Players/03 - A/Sole.dds",
                "Players/03 - A/Grip.gloves",
                "Players/03 - A/glove_l.fmdl",
                "Boots/Crocs/boots.fmdl",
                "Boots/Crocs/sole.ftex",
                "Gloves/Grip/glove_r.fmdl",
                "Gloves/Grip/sole.dds",
            ]),
            None
        );
    }

    #[test]
    fn a_combined_face_folder_s_files_take_a_face_s_roles() {
        let link = "Players/03 - A/Round.face";
        // Face models, the hair's skeleton, the face's files and textures.
        assert_eq!(
            gate(&[
                link,
                "Faces/Round/face_high.fmdl",
                "Faces/Round/hair_high.fmdl",
                "Faces/Round/oral.fmdl",
                "Faces/Round/x_fcl_hair.fmdl",
                "Faces/Round/x_fcl_hair.skl",
                "Faces/Round/face_diff.bin",
                "Faces/Round/fcl_hair_sim.fclo",
                "Faces/Round/skin.dds",
                "Faces/Round/hair.ftex",
            ]),
            None
        );
        // A model of another package, a face file without a face model, a skeleton pairing
        // with no model, a texture in another format: named, as in a player folder.
        for named in [
            "Faces/Round/boots.fmdl",
            "Faces/Round/glove_l.fmdl",
            "Faces/Round/torso.fmdl",
            "Faces/Round/face_diff.bin",
            "Faces/Round/fcl_hair.skl",
            "Faces/Round/skin.png",
        ] {
            assert_eq!(gate(&[link, named]), what(named), "{named}");
        }
        // The other files have a package to go in once a face model is in the folder.
        assert_eq!(
            gate(&[
                link,
                "Faces/Round/face_diff.bin",
                "Faces/Round/hair_high.fmdl"
            ]),
            None
        );
        // A folder with no model, textures alone, is named as a whole.
        assert_eq!(gate(&[link, "Faces/Round/skin.dds"]), what("Faces/Round"));
    }

    #[test]
    fn the_face_s_files_may_come_from_either_of_its_sources() {
        let link = "Players/03 - A/Round.face";
        let face_high = "Players/03 - A/face_high.fmdl";
        let shared_face = "Faces/Round/face_high.fmdl";
        // `face_diff.bin` in the shared folder serves a face model in the player's, and in the
        // player's a face model in the shared folder; missing from both, it is the player's.
        assert_eq!(
            first_hit(&[face_high, link, shared_face, "Faces/Round/face_diff.bin"]),
            None
        );
        assert_eq!(
            first_hit(&[link, "Players/03 - A/face_diff.bin", shared_face]),
            None
        );
        assert_eq!(
            first_hit(&[link, shared_face]),
            missing("Players/03 - A/face_diff.bin")
        );
        // The hair's skeleton may pair with the shared folder's hair part, or the player's.
        assert_eq!(
            gate(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/fcl_hair_sim.fclo",
                link,
                "Faces/Round/x_fcl_hair.fmdl",
                "Faces/Round/x_fcl_hair.skl",
            ]),
            None
        );
        assert_eq!(
            gate(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/fcl_hair.skl",
                link,
                "Faces/Round/x_fcl_hair.fmdl",
                "Faces/Round/fcl_hair_sim.fclo",
            ]),
            None
        );
        // Missing from both, it is named after the first hair part: the player's own before
        // the shared folder's.
        assert_eq!(
            gate(&[
                "Players/03 - A/fcl_hair.fmdl",
                "Players/03 - A/fcl_hair_sim.fclo",
                link,
                "Faces/Round/x_fcl_hair.fmdl",
            ]),
            missing("Players/03 - A/fcl_hair.skl")
        );
        assert_eq!(
            first_hit(&[
                link,
                "Faces/Round/face_diff.bin",
                "Faces/Round/x_fcl_hair.fmdl",
                "Faces/Round/fcl_hair_sim.fclo",
            ]),
            missing("Faces/Round/x_fcl_hair.skl")
        );
        assert_eq!(
            first_hit(&[
                link,
                "Faces/Round/face_diff.bin",
                "Faces/Round/x_fcl_hair.fmdl",
                "Faces/Round/x_fcl_hair.skl",
            ]),
            missing("Players/03 - A/fcl_hair_sim.fclo")
        );
    }

    /// The export's content folder holding the shared folders of `kind`.
    fn content_folder(kind: SharedKind) -> &'static str {
        match kind {
            SharedKind::Face => "Faces",
            SharedKind::Boots => "Boots",
            SharedKind::Gloves => "Gloves",
        }
    }

    /// `shared_not_compiled` over the shared `Boots/Crocs` (or `Gloves/Crocs`) holding `names`.
    fn shared_hit(kind: SharedKind, names: &[&str]) -> Option<(&'static str, String)> {
        let folder_path = format!("{}/Crocs", content_folder(kind));
        let files = names
            .iter()
            .map(|name| {
                let path = ScopePath::new(&format!("{folder_path}/{name}")).unwrap();
                FileDescriptor {
                    size: 0,
                    kind: aesthetics_export::classify(path.name()),
                    source: path.clone(),
                    path,
                }
            })
            .collect();
        let folder = SharedModelFolder {
            path: ScopePath::new(&folder_path).unwrap(),
            folder_name: "Crocs".to_owned(),
            files,
        };
        shared_not_compiled(kind, &folder)
    }

    #[test]
    fn a_shared_folder_takes_only_its_own_kind_s_models_with_their_files() {
        assert_eq!(
            shared_hit(
                SharedKind::Boots,
                &["boots.fmdl", "boots.skl", "shirt.dds", "sole.ftex"]
            ),
            None
        );
        assert_eq!(
            shared_hit(
                SharedKind::Gloves,
                &["glove_l.fmdl", "glove_r.fmdl", "grip.dds"]
            ),
            None
        );
        // Several parts under one name merge, each with its own skeleton.
        assert_eq!(
            shared_hit(
                SharedKind::Boots,
                &["boots.fmdl", "boots.skl", "kit_boots.fmdl", "kit_boots.skl"]
            ),
            None
        );
        assert_eq!(
            shared_hit(
                SharedKind::Face,
                &[
                    "face_high.fmdl",
                    "fcl_hair.fmdl",
                    "fcl_hair.skl",
                    "x_fcl_hair.fmdl",
                    "face_diff.bin",
                    "fcl_hair_sim.fclo",
                    "skin.dds"
                ]
            ),
            None
        );
        for (kind, names, named) in [
            (
                SharedKind::Boots,
                &["boots.fmdl", "glove_l.fmdl"][..],
                "glove_l.fmdl",
            ),
            (
                SharedKind::Gloves,
                &["boots.fmdl", "glove_l.fmdl"],
                "boots.fmdl",
            ),
            (
                SharedKind::Face,
                &["boots.fmdl", "face_high.fmdl"],
                "boots.fmdl",
            ),
            (
                SharedKind::Boots,
                &["boots.fmdl", "face_diff.bin"],
                "face_diff.bin",
            ),
            (SharedKind::Face, &["face_diff.bin"], "face_diff.bin"),
            (
                SharedKind::Boots,
                &["boots.fmdl", "kit_boots.skl"],
                "kit_boots.skl",
            ),
            (SharedKind::Boots, &["boots.fmdl", "shirt.png"], "shirt.png"),
        ] {
            assert_eq!(
                shared_hit(kind, names),
                what(&format!("{}/Crocs/{named}", content_folder(kind))),
                "{names:?}"
            );
        }
        // A folder with no model, textures alone or nothing at all, is named as a whole.
        assert_eq!(
            shared_hit(SharedKind::Boots, &["shirt.dds"]),
            what("Boots/Crocs")
        );
        assert_eq!(shared_hit(SharedKind::Gloves, &[]), what("Gloves/Crocs"));
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
    fn the_first_hit_follows_players_files_links_missing_files_shared_folders_kits_then_the_rest() {
        let face_high = "Players/03 - A/face_high.fmdl";
        let torso = "Players/03 - A/torso.fmdl";
        let face_link = "Players/03 - A/Round.face";
        let shared_face = "Faces/Round/face_high.fmdl";
        let shared_face_texture = "Faces/Round/hair.png";
        let link = "Players/03 - A/Crocs.boots";
        let shared = "Boots/Crocs/boots.fmdl";
        let shared_texture = "Boots/Crocs/shirt.png";
        let kit = "Kits/g1/kit.png";
        let portrait = "Portraits/player_03.png";
        // A folder's own files come before its links' folders, and both before what it lacks:
        // `face_diff.bin` is missing too.
        assert_eq!(
            first_hit(&[
                face_high,
                torso,
                face_link,
                shared_face,
                shared_face_texture,
                link,
                shared,
                shared_texture,
                kit,
                portrait
            ]),
            what(torso)
        );
        assert_eq!(
            first_hit(&[
                face_high,
                face_link,
                shared_face,
                shared_face_texture,
                link,
                shared,
                shared_texture,
                kit,
                portrait
            ]),
            what(shared_face_texture)
        );
        assert_eq!(
            first_hit(&[face_high, link, shared, shared_texture, kit, portrait]),
            missing("Players/03 - A/face_diff.bin")
        );
        assert_eq!(
            gate(&[link, shared, shared_texture, kit, portrait]),
            what(shared_texture)
        );
        assert_eq!(gate(&[link, shared, kit, portrait]), what(kit));
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
        let models = FolderModels::of(&folder.path, &folder.files);
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

    fn skeleton(package: ModelPackage, name: &'static str) -> Option<PlayerFile> {
        Some(PlayerFile::Skeleton { package, name })
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
            skeleton(ModelPackage::Face, "fcl_hair_sim.skl")
        );
        assert_eq!(
            role("kit_boots.skl"),
            skeleton(ModelPackage::Boots, "boots.skl")
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
                skeleton(ModelPackage::Face, "fcl_hair_sim.skl"),
                None
            ]
        );
        assert_eq!(
            roles(&["boots.fmdl", "boots.skl", "kit_boots.skl"]),
            [
                model(ModelPackage::Boots, "boots"),
                skeleton(ModelPackage::Boots, "boots.skl"),
                None
            ]
        );
        // Every boots part pairs its own skeleton, not only the first, and so does every hair
        // part.
        assert_eq!(
            roles(&[
                "a_boots.fmdl",
                "a_boots.skl",
                "kit_boots.fmdl",
                "kit_boots.skl"
            ]),
            [
                model(ModelPackage::Boots, "boots"),
                skeleton(ModelPackage::Boots, "boots.skl"),
                model(ModelPackage::Boots, "boots"),
                skeleton(ModelPackage::Boots, "boots.skl")
            ]
        );
        assert_eq!(
            roles(&[
                "a_fcl_hair.fmdl",
                "a_fcl_hair.skl",
                "fcl_hair.fmdl",
                "fcl_hair.skl"
            ]),
            [
                model(ModelPackage::Face, "fcl_hair"),
                skeleton(ModelPackage::Face, "fcl_hair_sim.skl"),
                model(ModelPackage::Face, "fcl_hair"),
                skeleton(ModelPackage::Face, "fcl_hair_sim.skl")
            ]
        );
        // Without a face model the face's files have no package to go in, unless a shared
        // face is linked: then the shared face is the player's.
        assert_eq!(
            roles(&["boots.fmdl", "face_diff.bin", "fcl_hair_sim.fclo"]),
            [model(ModelPackage::Boots, "boots"), None, None]
        );
        let folder = folder(&["boots.fmdl", "face_diff.bin", "fcl_hair_sim.fclo"]);
        let models = FolderModels::of(&folder.path, &folder.files).with_linked_face();
        let linked: Vec<Option<PlayerFile>> = folder
            .files
            .iter()
            .map(|file| player_file(&folder.path, file, &models))
            .collect();
        assert_eq!(
            linked,
            [
                model(ModelPackage::Boots, "boots"),
                packed(ModelPackage::Face, "face_diff.bin"),
                packed(ModelPackage::Face, "fcl_hair_sim.fclo")
            ]
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
            skeleton(ModelPackage::Boots, "boots.skl")
                .unwrap()
                .package(),
            Some(ModelPackage::Boots)
        );
        assert_eq!(
            packed(ModelPackage::Face, "face_diff.bin")
                .unwrap()
                .package(),
            Some(ModelPackage::Face)
        );
        assert_eq!(
            model(ModelPackage::Gloves, "glove_l").unwrap().package(),
            Some(ModelPackage::Gloves)
        );
        assert_eq!(
            ModelPackage::ALL.map(ModelPackage::file_stem),
            ["face", "boots", "glove"]
        );
        assert_eq!(
            ModelPackage::ALL.map(ModelPackage::name),
            ["face", "boots", "gloves"]
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
