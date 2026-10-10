//! `FolderDraft` → `PlayerFolder` / `SharedModelFolder`, the pass-through
//! file lists, and each folder kind's own checks (the file-type allowlist,
//! links, markers and naming rules of "Validation semantics").

use std::collections::{BTreeMap, BTreeSet};

use vtree::ScopePath;

use crate::FileKind;
use crate::conventions::{
    Marker, MetadataFile, SharedKind, classify, common_link_name, is_boots, is_explicit_face,
    is_gloves, is_player_singleton, model_suffix, shared_link_name, split_folder_name,
};
use crate::listing::ValidationContext;
use crate::parse::{AestheticsExportDraft, ExportKind, FileDescriptor, FolderDraft};
use crate::validate::{Disposition, IssueScope, ValidationIssue};
use crate::validate::{issue_in, strict_disposition};

use super::links;

/// A player folder: the sanitized main reference point for one player.
/// Numbering is a roster property, not folder content; the roster maps the
/// slots. Typed fields hold the savefile-stage inputs and folder-level
/// references; `files` holds the model-pipeline content (models, textures,
/// material files, skeletons), however it reaches the output — files inside a
/// subfolder, of any name and at any depth, are ordinary `files` entries, the
/// path keeping the subfolder (`player_folders.md` "Subfolders").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerFolder {
    /// The folder's path (`Players/03 - A`): the scope its issues name.
    pub path: vtree::ScopePath,
    /// The folder name without its `NN - ` number; with a roster file the
    /// whole folder name.
    pub player_name: String,
    /// Names, sizes, kinds — contents load later, per task.
    pub files: Vec<FileDescriptor>,
    /// Shared-folder link files (`Crocs.boots` → shared `Boots/Crocs/`), at
    /// any depth of the folder, each linked folder once however many links
    /// name it (`jessie/Crocs.boots` beside a root `Crocs.boots`); `.common`
    /// links are `files` entries of link kind, resolved against `Common/` by
    /// the pipeline.
    pub links: Vec<SharedLink>,
    /// A recognized `ingame_face` / `ingame_face.txt` marker.
    pub ingame_face: bool,
    /// The normalized `fpc_on`/`fpc_off` directive; `None` when absent or when
    /// both markers are present.
    pub fpc: Option<FpcDirective>,
    /// A `portrait.*` texture directly in the folder; a surviving folder has
    /// at most one (two `portrait.*` stems is a `texture_stem_conflict`).
    pub portrait: Option<FileDescriptor>,
    /// `settings.toml`.
    pub settings: Option<FileDescriptor>,
}

/// A shared-folder link's target: `Crocs.boots` names `Boots/Crocs/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedLink {
    /// Which shared folder kind the link targets.
    pub kind: SharedKind,
    /// The shared folder's name (`Crocs`).
    pub name: String,
}

/// The normalized `fpc_on`/`fpc_off` marker directive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FpcDirective {
    /// `fpc_on`, or a bare `fpc`: apply the FPC preset.
    On,
    /// `fpc_off`: un-apply the FPC preset.
    Off,
}

/// A shared model folder (`Faces/`/`Boots/`/`Gloves/` child), referenced by
/// name from player folders; IDs are assigned by run planning, not here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedModelFolder {
    /// The folder's path (`Boots/Crocs`): the scope its issues name.
    pub path: vtree::ScopePath,
    /// The folder's name (its path's last segment) — the link lookup key.
    pub folder_name: String,
    /// Every file below it.
    pub files: Vec<FileDescriptor>,
}

/// `FolderDraft` → `PlayerFolder`, no checks (that is the next slice's). The
/// folder's name is the player name, whole under a roster file, else the
/// `<head>[ - <label>]` split's label when present.
pub(crate) fn player_folder(draft: &FolderDraft, roster_file: bool) -> PlayerFolder {
    let name = draft.path.name();
    let player_name = if roster_file {
        name.to_owned()
    } else {
        split_folder_name(name).1.unwrap_or(name).to_owned()
    };
    let depth = draft.path.segments().count() + 1;
    let mut files = Vec::new();
    let mut links = Vec::new();
    let mut ingame_face = false;
    let mut fpc_on = false;
    let mut fpc_off = false;
    let mut portrait = None;
    let mut settings = None;
    for file in &draft.files {
        // Only a file directly in the folder can be a marker, settings or
        // portrait; everything else below a subfolder is pipeline content (the
        // allowlist names such a file out of place). A shared link counts at
        // any depth, as one in the root (`player_folders.md` "Subfolders").
        if file.path.segments().count() != depth && !matches!(file.kind, FileKind::SharedLink(_)) {
            files.push(file.clone());
            continue;
        }
        let name = file.path.name();
        match file.kind {
            FileKind::SharedLink(_) => {
                // `shared_link_name` and `classify` share the link table, so a
                // SharedLink kind implies a non-empty stem.
                let (kind, name) = shared_link_name(name)
                    .expect("a SharedLink kind implies a non-empty link stem");
                links.push(SharedLink { kind, name });
            }
            FileKind::Marker(Marker::IngameFace) => ingame_face = true,
            FileKind::Marker(Marker::FpcOn) => fpc_on = true,
            FileKind::Marker(Marker::FpcOff) => fpc_off = true,
            FileKind::Metadata(MetadataFile::SettingsToml) => settings = Some(file.clone()),
            FileKind::Texture
                if name
                    .rsplit_once('.')
                    .is_some_and(|(stem, _)| stem.eq_ignore_ascii_case("portrait")) =>
            {
                portrait = Some(file.clone());
            }
            FileKind::Model(_)
            | FileKind::Texture
            | FileKind::Skl
            | FileKind::Fclo
            | FileKind::Xml
            | FileKind::Mtl
            | FileKind::MaterialsToml
            | FileKind::Bin
            | FileKind::CommonLink
            | FileKind::Marker(Marker::PreFox | Marker::Fox | Marker::Icon)
            | FileKind::Metadata(
                MetadataFile::PlayersTxt
                | MetadataFile::RefsTxt
                | MetadataFile::RefLists
                | MetadataFile::NotesTxt
                | MetadataFile::ColorsTxt
                | MetadataFile::ConfigToml
                | MetadataFile::Readme,
            )
            | FileKind::Other => files.push(file.clone()),
        }
    }
    // Both fpc markers present is a conflict, not a directive; the check that
    // drops such a folder is the next slice's.
    let fpc = match (fpc_on, fpc_off) {
        (true, false) => Some(FpcDirective::On),
        (false, true) => Some(FpcDirective::Off),
        _ => None,
    };
    PlayerFolder {
        path: draft.path.clone(),
        player_name,
        files,
        links,
        ingame_face,
        fpc,
        portrait,
        settings,
    }
}

/// The `Kits/` folder, sanitized: one `KitFolder` per surviving kit, plus
/// `all/`'s surviving textures as found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KitsFolder {
    /// One per kit folder; `all/` is not a kit. A `Full` team export with no surviving player
    /// kit gets an empty `Kits/p1`, one with no surviving `g1` an empty `Kits/g1` (every team
    /// needs one of each: `team_compiler/pipeline.md` "Bins accumulation").
    pub kits: BTreeMap<kit_config::KitSlot, KitFolder>,
    /// `all/` textures as found (each also appears, as `Shared`, in every kit
    /// lacking that stem).
    pub shared: Vec<FileDescriptor>,
}

/// One kit folder's sanitized contents: the slot's own look plus what it
/// inherits from `all/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KitFolder {
    /// `Kits/p1` or `Kits/p1 - Lakers`, as the export spells it: the scope a consumer's kit
    /// findings name.
    pub path: ScopePath,
    /// The free part after ` - `; GUI/editor display only.
    pub label: Option<String>,
    /// `config.toml` (absent → generated at compile time).
    pub config: Option<FileDescriptor>,
    /// `colors.txt`, the kit's two menu colors (read by `read_colors_txt`).
    pub colors: Option<FileDescriptor>,
    /// The `icon_<N>` marker's number, 0–23; `None` when absent or
    /// `kit_icon_invalid` (the default 3 applies).
    pub icon: Option<u8>,
    /// `fox` / `pre-fox` marker file; `None` = drawn for the target engine.
    pub layout: Option<KitLayout>,
    /// The *effective* set: own files, plus `all/` files for stems the kit
    /// lacks.
    pub textures: Vec<KitTexture>,
}

/// Which engine's kit UV layout `kit` and its mask/srm are drawn for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KitLayout {
    /// `pre-fox`.
    PreFox,
    /// `fox`.
    Fox,
}

/// One texture in a kit's effective set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KitTexture {
    /// `kit`, `kit_back`, … (lowercased).
    pub stem: String,
    /// The texture's descriptor.
    pub file: FileDescriptor,
    /// `Own` | `Shared` — provenance for the editor and
    /// `kit_textures_inherited`.
    pub source: KitTextureSource,
}

/// Where a `KitTexture` comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KitTextureSource {
    /// The kit folder itself.
    Own,
    /// Inherited from `all/` (the stem was missing).
    Shared,
}

/// `FolderDraft` → `SharedModelFolder`, unchecked this slice.
pub(crate) fn shared_model_folder(draft: &FolderDraft) -> SharedModelFolder {
    SharedModelFolder {
        path: draft.path.clone(),
        folder_name: draft.path.name().to_owned(),
        files: draft.files.clone(),
    }
}

/// A file's position relative to the player folder holding it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Position {
    /// Directly in the folder, where the markers and settings are read.
    Direct,
    /// Anywhere below one of its subfolders, at any depth: a player folder of
    /// its own, holding model content and links (`player_folders.md` "Subfolders").
    Below,
}

/// Whether `path` is a file directly in `folder`.
pub(crate) fn directly_in(path: &ScopePath, folder: &ScopePath) -> bool {
    path.parent()
        .is_some_and(|parent| parent.fold_key() == folder.fold_key())
}

/// `path`'s position under `folder`, one of whose own files it is.
pub(crate) fn position(path: &ScopePath, folder: &ScopePath) -> Position {
    if directly_in(path, folder) {
        Position::Direct
    } else {
        Position::Below
    }
}

/// One `texture_stem_conflict` per file of a folded stem two or more files
/// share: `File`-scoped `DropFile` (`Portraits/` and `Common/` drop files,
/// not folders).
pub(crate) fn file_stem_conflicts<'a>(
    context: &ValidationContext,
    files: impl Iterator<Item = &'a FileDescriptor>,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut stems: BTreeMap<String, Vec<&FileDescriptor>> = BTreeMap::new();
    for file in files {
        stems
            .entry(fold(stem(file.path.name())))
            .or_default()
            .push(file);
    }
    for (stem, files) in stems {
        if files.len() > 1 {
            let names = files
                .iter()
                .map(|file| file.path.name().to_owned())
                .collect::<Vec<_>>()
                .join(",");
            for file in files {
                issues.push(issue_in(
                    context,
                    "texture_stem_conflict",
                    IssueScope::File(file.path.clone()),
                    vec![("stem", stem.clone()), ("files", names.clone())],
                    Disposition::DropFile,
                ));
            }
        }
    }
}

/// One `texture_stem_conflict` per folded stem two or more files share:
/// `Folder`-scoped `DropFolder`, context `("stem", fold key)` and `("files",
/// display names joined by `","`).
pub(crate) fn stem_conflicts(
    context: &ValidationContext,
    scope: IssueScope,
    stems: impl Iterator<Item = (String, String)>,
    issues: &mut Vec<ValidationIssue>,
) {
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (key, display) in stems {
        groups.entry(key).or_default().push(display);
    }
    for (stem, files) in groups {
        if files.len() > 1 {
            issues.push(issue_in(
                context,
                "texture_stem_conflict",
                scope.clone(),
                vec![("stem", stem), ("files", files.join(","))],
                Disposition::DropFolder,
            ));
        }
    }
}

/// `path` below `folder` as a relative path string (`extra/x.dds`).
pub(crate) fn relative(path: &ScopePath, folder: &ScopePath) -> String {
    path.segments()
        .skip(folder.segments().count())
        .collect::<Vec<_>>()
        .join("/")
}

/// "Model content" (the allowlist table): models, textures, `.skl`, `.fclo`,
/// `.xml`, `.mtl`, material tomls and `.bin`.
fn is_model_content(kind: FileKind) -> bool {
    matches!(
        kind,
        FileKind::Model(_)
            | FileKind::Texture
            | FileKind::Skl
            | FileKind::Fclo
            | FileKind::Xml
            | FileKind::Mtl
            | FileKind::MaterialsToml
            | FileKind::Bin
    )
}

/// What a file below a subfolder of the player folder may be: model content, but a
/// per-player singleton (`is_player_singleton`), which counts directly in the player folder
/// alone, and a link where one counts (`counts_as_link`).
fn player_below_allowed(file: &FileDescriptor) -> bool {
    (is_model_content(file.kind) && !is_player_singleton(file.path.name())) || counts_as_link(file)
}

/// Whether `file`, a file of a player folder, is a link where it sits: a shared
/// link at any depth, read as one in the root, and a `.common` link at any depth,
/// standing for the `Common/` file at its own path below the folder
/// (`player_folders.md` "Subfolders", `model_format.md` "Link files").
pub(crate) fn counts_as_link(file: &FileDescriptor) -> bool {
    match file.kind {
        FileKind::SharedLink(_) | FileKind::CommonLink => true,
        FileKind::Model(_)
        | FileKind::Texture
        | FileKind::Skl
        | FileKind::Fclo
        | FileKind::Xml
        | FileKind::Mtl
        | FileKind::MaterialsToml
        | FileKind::Bin
        | FileKind::Marker(_)
        | FileKind::Metadata(_)
        | FileKind::Other => false,
    }
}

/// What a file directly in a player folder may be (allowlist row 1).
fn player_direct_allowed(kind: FileKind) -> bool {
    is_model_content(kind)
        || matches!(
            kind,
            FileKind::SharedLink(_)
                | FileKind::CommonLink
                | FileKind::Marker(Marker::IngameFace | Marker::FpcOn | Marker::FpcOff)
                | FileKind::Metadata(MetadataFile::SettingsToml)
        )
}

/// A stem's tail: the name before its last `.` (`hair.dds` → `hair`).
pub(crate) fn stem(name: &str) -> &str {
    name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name)
}

/// The category subfolders of the prototype referee layout, one per category
/// inside each referee's folder, matched in any case.
const PROTO_SUBFOLDERS: [&str; 4] = ["face", "boots", "gloves", "common"];

/// In a referee export, the first direct subfolder of `folder`, in its files'
/// order, named as one of `PROTO_SUBFOLDERS`, spelled as the export spells it:
/// the prototype referee layout, which types a subfolder's files by the subfolder's
/// name where a Studio subfolder's files are typed by their own names
/// (`player_folders.md` "Subfolders"). `None` in a team export, where the names
/// are plain subfolders.
fn proto_layout_subfolder<'a>(
    draft: &AestheticsExportDraft,
    folder: &'a FolderDraft,
) -> Option<&'a str> {
    match draft.kind() {
        ExportKind::Referees => {}
        ExportKind::Team => return None,
    }
    let depth = folder.path.segments().count();
    folder
        .files
        .iter()
        .filter_map(|file| {
            let mut below = file.path.segments().skip(depth);
            let first = below.next()?;
            // A file directly in the folder has no subfolder.
            below.next()?;
            Some(first)
        })
        .find(|name| {
            PROTO_SUBFOLDERS
                .iter()
                .any(|proto| name.eq_ignore_ascii_case(proto))
        })
}

/// A player folder's own findings, in the order the semantics list them: the
/// prototype referee layout (alone when found), allowlist, duplicate links, missing
/// targets, markers, stems, then the edit-hair files.
pub(crate) fn check_player(
    draft: &AestheticsExportDraft,
    folder: &FolderDraft,
    context: &ValidationContext,
    issues: &mut Vec<ValidationIssue>,
) {
    let scope = IssueScope::Folder(folder.path.clone());

    // 0. A referee folder in the prototype layout is that layout's one finding,
    // as `export_layout_old` is the old export layout's: every other finding
    // would describe the old layout again.
    if let Some(subfolder) = proto_layout_subfolder(draft, folder) {
        issues.push(issue_in(
            context,
            "player_layout_proto",
            scope,
            vec![("folder", subfolder.to_owned())],
            Disposition::DropFolder,
        ));
        return;
    }

    // 1. The file-type allowlist.
    for file in &folder.files {
        let allowed = match position(&file.path, &folder.path) {
            Position::Direct => player_direct_allowed(file.kind),
            Position::Below => player_below_allowed(file),
        };
        if !allowed {
            issues.push(issue_in(
                context,
                "file_type_disallowed",
                scope.clone(),
                vec![("file", relative(&file.path, &folder.path))],
                strict_disposition(context, Disposition::DropFolder),
            ));
        }
    }

    // 2. One shared folder per kind, over the whole tree: two links naming one folder (a
    // subfolder's beside the root's, or `Crocs.boots` beside `crocs.boots.txt`) name it once.
    for kind in [SharedKind::Face, SharedKind::Boots, SharedKind::Gloves] {
        let named: BTreeSet<String> = folder
            .files
            .iter()
            .filter(|file| file.kind == FileKind::SharedLink(kind))
            .filter_map(|file| shared_link_name(file.path.name()))
            .map(|(_, name)| fold(&name))
            .collect();
        if named.len() > 1 {
            issues.push(issue_in(
                context,
                "shared_link_duplicate",
                scope.clone(),
                vec![("kind", kind.name().to_owned())],
                Disposition::DropFolder,
            ));
        }
    }

    // 3.-4. A link's target must exist (the resolver is `links`'s).
    let resolved = links::player_links(folder, draft);
    for link in &resolved {
        if let links::ResolvedLinkKind::Shared(_) = &link.kind
            && link.target.is_none()
        {
            issues.push(issue_in(
                context,
                "link_target_missing",
                scope.clone(),
                vec![("link", link.link_name.clone())],
                Disposition::DropFolder,
            ));
        }
    }
    for link in &resolved {
        if let links::ResolvedLinkKind::Common(name) = &link.kind
            && link.common_target_missing(context)
        {
            issues.push(issue_in(
                context,
                "common_link_missing",
                scope.clone(),
                vec![
                    ("link", link.link_name.clone()),
                    ("path", format!("Common/{name}")),
                ],
                Disposition::DropFolder,
            ));
        }
    }

    // 5. Both fpc markers contradict.
    let fpc_on = folder.files.iter().any(|file| {
        file.kind == FileKind::Marker(Marker::FpcOn)
            && position(&file.path, &folder.path) == Position::Direct
    });
    let fpc_off = folder.files.iter().any(|file| {
        file.kind == FileKind::Marker(Marker::FpcOff)
            && position(&file.path, &folder.path) == Position::Direct
    });
    if fpc_on && fpc_off {
        issues.push(issue_in(
            context,
            "fpc_conflict",
            scope.clone(),
            vec![],
            Disposition::DropFolder,
        ));
    }

    // 6. `ingame_face` plus explicit face content contradict.
    let ingame_face = folder.files.iter().any(|file| {
        file.kind == FileKind::Marker(Marker::IngameFace)
            && position(&file.path, &folder.path) == Position::Direct
    });
    if ingame_face {
        // A model is face content by its name wherever it sits, and so is a link: a face link
        // and a model link count as links at any depth (`counts_as_link`).
        let trigger = folder.files.iter().find(|file| match file.kind {
            FileKind::SharedLink(SharedKind::Face) => true,
            FileKind::SharedLink(SharedKind::Boots | SharedKind::Gloves) => false,
            FileKind::Model(_) => {
                model_suffix(stem(file.path.name())).is_some_and(is_explicit_face)
            }
            FileKind::CommonLink => {
                common_link_name(file.path.name()).is_some_and(|name| is_explicit_model_file(&name))
            }
            FileKind::Texture
            | FileKind::Skl
            | FileKind::Fclo
            | FileKind::Xml
            | FileKind::Mtl
            | FileKind::MaterialsToml
            | FileKind::Bin
            | FileKind::Marker(_)
            | FileKind::Metadata(_)
            | FileKind::Other => false,
        });
        if let Some(file) = trigger {
            issues.push(issue_in(
                context,
                "ingame_face_explicit_face_model",
                scope.clone(),
                vec![("file", file.path.name().to_owned())],
                Disposition::DropFolder,
            ));
        }
    }

    // 7. Texture stems collide within one folder of the player folder's tree, the
    // root or one subfolder: a texture name resolves nearest first, in its model's
    // folder before each parent (`player_folders.md` "Subfolders"), so the root's
    // `hair.dds` and a subfolder's are no conflict. A texture `.common` link claims
    // its linked name in the folder holding it (a link counts as the linked file being
    // local, `model_format.md` "Rules").
    let mut namespaces: BTreeMap<Option<String>, Vec<(String, String)>> = BTreeMap::new();
    for file in &folder.files {
        let Some(name) = texture_claim(file) else {
            continue;
        };
        let namespace = file.path.parent().map(|parent| parent.fold_key());
        namespaces
            .entry(namespace)
            .or_default()
            .push((fold(stem(&name)), relative(&file.path, &folder.path)));
    }
    for stems in namespaces.into_values() {
        stem_conflicts(context, scope.clone(), stems.into_iter(), issues);
    }

    // 8. The edit-hair files.
    edithair_files(folder, context, issues);
}

/// The name `file`, a file of a player folder, claims in the texture namespace of the folder
/// holding it: a texture's own, or the linked name of a texture `.common` link at any depth;
/// `None` for any other file.
fn texture_claim(file: &FileDescriptor) -> Option<String> {
    if file.kind == FileKind::Texture {
        return Some(file.path.name().to_owned());
    }
    if file.kind != FileKind::CommonLink {
        return None;
    }
    common_link_name(file.path.name()).filter(|name| classify(name) == FileKind::Texture)
}

/// The draft's shared folders of `kind`.
pub(crate) fn shared_folders(draft: &AestheticsExportDraft, kind: SharedKind) -> &Vec<FolderDraft> {
    match kind {
        SharedKind::Face => &draft.faces,
        SharedKind::Boots => &draft.boots,
        SharedKind::Gloves => &draft.gloves,
    }
}

/// Whether `file_name` names a model file whose suffix is an explicit face
/// model (the `.common` link's target side of the ingame-face rule).
fn is_explicit_model_file(name: &str) -> bool {
    matches!(classify(name), FileKind::Model(_))
        && model_suffix(stem(name)).is_some_and(is_explicit_face)
}

/// A name's fold key for lookups (the same casing rule the tree uses, without
/// the validation — a stem or link name need not be a valid path segment).
pub(crate) fn fold(name: &str) -> String {
    vtree::fold_name(name)
}

/// A shared folder's own findings: the allowlist, stem collisions, the
/// boots/gloves naming rule (`fmdl_name_invalid` on Fox, `model_name_invalid`
/// on pre-Fox), and the edit-hair files.
pub(crate) fn check_shared(
    folder: &FolderDraft,
    kind: SharedKind,
    context: &ValidationContext,
    issues: &mut Vec<ValidationIssue>,
) {
    let scope = IssueScope::Folder(folder.path.clone());

    // Model content only, and only directly in the folder.
    for file in &folder.files {
        if !(directly_in(&file.path, &folder.path) && is_model_content(file.kind)) {
            issues.push(issue_in(
                context,
                "file_type_disallowed",
                scope.clone(),
                vec![("file", relative(&file.path, &folder.path))],
                strict_disposition(context, Disposition::DropFolder),
            ));
        }
    }

    // Texture stems collide over the direct files.
    stem_conflicts(
        context,
        scope.clone(),
        folder
            .files
            .iter()
            .filter(|file| file.kind == FileKind::Texture && directly_in(&file.path, &folder.path))
            .map(|file| (fold(stem(file.path.name())), file.path.name().to_owned())),
        issues,
    );

    // Every boots/gloves model must say so by suffix (`Faces/` takes any
    // name — nothing there can be a face anywhere else): one rule, under each
    // engine's model catalog's code.
    let name_invalid = match context.version.engine() {
        pes_version::Engine::Fox => "fmdl_name_invalid",
        pes_version::Engine::PreFox => "model_name_invalid",
    };
    for file in &folder.files {
        let allowed = match kind {
            SharedKind::Face => true,
            SharedKind::Boots => model_suffix(stem(file.path.name())).is_some_and(is_boots),
            SharedKind::Gloves => model_suffix(stem(file.path.name())).is_some_and(is_gloves),
        };
        if matches!(file.kind, FileKind::Model(_))
            && directly_in(&file.path, &folder.path)
            && !allowed
        {
            issues.push(issue_in(
                context,
                name_invalid,
                scope.clone(),
                vec![("file", file.path.name().to_owned())],
                Disposition::DropFolder,
            ));
        }
    }

    edithair_files(folder, context, issues);
}

/// The edit-hair files' names: a model folder holding one of them, at any depth,
/// is refused on a pre-Fox target (`edithair_unsupported`).
const EDITHAIR_NAMES: [&str; 2] = ["face_edithair.xml", "hair.xml"];

/// `edithair_unsupported` for each file of `folder`, at any depth below it and of
/// any kind, named (case-folded) as an edit-hair file, when the target is
/// pre-Fox: the folder is dropped, the finding naming the file below the folder.
fn edithair_files(
    folder: &FolderDraft,
    context: &ValidationContext,
    issues: &mut Vec<ValidationIssue>,
) {
    match context.version.engine() {
        pes_version::Engine::PreFox => {}
        // The file means nothing to a Fox game (an XML there is
        // `xml_ignored_fox`'s, `team_compiler/messages.md`).
        pes_version::Engine::Fox => return,
    }
    for file in &folder.files {
        let name = fold(file.path.name());
        if EDITHAIR_NAMES.iter().any(|edithair| fold(edithair) == name) {
            issues.push(issue_in(
                context,
                "edithair_unsupported",
                IssueScope::Folder(folder.path.clone()),
                vec![("file", relative(&file.path, &folder.path))],
                Disposition::DropFolder,
            ));
        }
    }
}

/// The `Common/` allowlist and its own stem namespaces: model content only, at
/// any depth (on PES 15-17 a subfolder is packed at its own path); conflicting
/// textures drop each other. The stem check is per directory of `Common/`, as
/// `check_player`'s is per folder of a player's tree: every directory packs at
/// its own path, so two textures of one stem collide only within one directory.
pub(crate) fn check_common(
    draft: &AestheticsExportDraft,
    context: &ValidationContext,
    issues: &mut Vec<ValidationIssue>,
) {
    for file in &draft.common {
        if !is_model_content(file.kind) {
            issues.push(issue_in(
                context,
                "common_file_disallowed",
                IssueScope::File(file.path.clone()),
                vec![],
                strict_disposition(context, Disposition::DropFile),
            ));
        }
    }
    let mut directories: BTreeMap<Option<String>, Vec<&FileDescriptor>> = BTreeMap::new();
    for file in &draft.common {
        if file.kind == FileKind::Texture {
            directories
                .entry(file.path.parent().map(|parent| parent.fold_key()))
                .or_default()
                .push(file);
        }
    }
    for files in directories.into_values() {
        file_stem_conflicts(context, files.into_iter(), issues);
    }
}

/// The `Collars/` allowlist: model files directly in the folder, in any model format. The game
/// draws a collar with the team's kit texture, so a texture or a `.mtl` there has nothing to
/// go with. Each other file drops alone, a collar being its own unit. The `collar_<ID>` name
/// is the consumer's to check: which IDs exist depends on the target version.
pub(crate) fn check_collars(
    draft: &AestheticsExportDraft,
    context: &ValidationContext,
    issues: &mut Vec<ValidationIssue>,
) {
    let folder = ScopePath::new("Collars").expect("`Collars` is a valid path segment");
    for file in &draft.collars {
        if !(directly_in(&file.path, &folder) && matches!(file.kind, FileKind::Model(_))) {
            issues.push(issue_in(
                context,
                "file_type_disallowed",
                IssueScope::File(file.path.clone()),
                vec![("file", relative(&file.path, &folder))],
                strict_disposition(context, Disposition::DropFile),
            ));
        }
    }
}
