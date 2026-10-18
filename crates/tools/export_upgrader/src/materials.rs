//! Step 7's `.mtl` texture references and FMDL path-table entries into the
//! team's `Common/` (`plans/export_upgrader.md` step 7, `DECISIONS.md`
//! 2026-10-18 "old `.mtl` references keep their extension, and a Common
//! reference no link can spell is kept" and "an FMDL's texture entries,
//! class by class"): a reference naming a `Common/` file becomes the bare
//! name plus the `.common` link spelling it, and one no link can spell
//! stays as written, its `???` respelled the resolved team's ID, reported
//! `common_reference_kept`. A shared folder's (`Boots/`, `Gloves/`) or a
//! `Common/` `.mtl` keeps its Common references as written — `???`
//! respelled, no link — with `common_reference_kept` only when `Common/`
//! lacks what it names (`DECISIONS.md` 2026-10-18 "a shared folder's
//! `.mtl` keeps its Common references, and a link needs one texture"); a
//! shared folder's `.fmdl` links as a player folder's (the Fox compile
//! reads a shared folder's links), and a `Common/` `.fmdl` gets the shared
//! `.mtl` rule.

use std::borrow::Cow;
use std::collections::BTreeMap;

use aesthetics_export::{
    FileKind, KitToken, ModelFormat, classify, common_link_name, kit_path_token, kit_set_member,
};
use fmdl::FmdlFile;
use pes_model::format::mtl::MaterialSet;
use pes_model::ops::paths::rewrite_texture_paths;
use studio_core::{Disposition, Message, Scope};
use teams_list::TeamId;

use crate::carried::under;
use crate::kit_magic::{Spelling, respell};
use crate::messages::{Code, INPUT, tool_message};
use crate::players::path_scope;
use crate::source::SourceFailure;

/// What step 7 makes of the migrated model folders' `.mtl` and `.fmdl`
/// files: the ones rewritten, the `.common` links their references need,
/// and the findings on the input.
#[derive(Debug, Default)]
pub(crate) struct MaterialRewrites {
    /// Rewritten `.mtl`/`.fmdl` files: output path and contents; `run`
    /// writes them instead of copying their input.
    pub(crate) written: Vec<(String, Vec<u8>)>,
    /// Empty `.common` link files to write, output paths.
    pub(crate) links: Vec<String>,
    /// Findings on the input, in input-path order.
    pub(crate) findings: Vec<Message>,
}

/// The game directory prefix the team's `Common/` stands in, matched in
/// any letter case.
const COMMON_PREFIX: &str = "model/character/uniform/common/";

/// The team's Common directory an FMDL texture names, `???` the `XXX`
/// placeholder or a baked-in team ID (`common_home` for the Fox engine:
/// `…/common/<team>/<sub>sourceimages/`).
const FOX_COMMON_PREFIX: &str = "/Assets/pes16/model/character/common/";

/// The game's kit texture folder an FMDL texture keeps
/// (`fmdl_texture_kit_path`): a directory below it is a kit texture's too
/// (the compiler's prefix check).
const FOX_KIT_TEXTURE: &str = "/Assets/pes16/model/character/uniform/texture/";

/// The `???` (three characters) and `sub` of a team-Common directory
/// `/Assets/pes16/model/character/common/<???>/<sub>sourceimages/` (the
/// prefix compared ASCII case-insensitively): `sub` the path between
/// `???/` and `sourceimages/`, `""` or ending `/`, the compiler's
/// `sourceimages/` level not the export's.
fn reference_into_fox_common(directory: &str) -> Option<(&str, &str)> {
    let rest = directory
        .get(..FOX_COMMON_PREFIX.len())?
        .eq_ignore_ascii_case(FOX_COMMON_PREFIX)
        .then(|| &directory[FOX_COMMON_PREFIX.len()..])?;
    let (marker, rest) = rest.split_once('/')?;
    if marker.chars().count() != 3 || !rest.ends_with("sourceimages/") {
        return None;
    }
    Some((marker, &rest[..rest.len() - "sourceimages/".len()]))
}

/// The folded stem of a file name (`hair.dds` → `hair`, `Hair.PNG` the
/// same).
fn folded_stem(name: &str) -> String {
    let stem = name.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(name);
    vtree::fold_name(stem)
}

/// Whether the file `name` claims the folded texture stem `stem` in its
/// folder: a texture of that stem, or a `.common` link to one (a model's
/// link, `hair.model.common`, claims no texture stem) — or, for a `kitN`
/// `set_reference`, a texture or texture link that is a member of its set.
fn claims_stem(name: &str, stem: &str, set_reference: Option<&str>) -> bool {
    let claims = |name: &str| {
        classify(name) == FileKind::Texture
            && (folded_stem(name) == stem
                || set_reference.is_some_and(|reference| kit_set_member(reference, name).is_some()))
    };
    if claims(name) {
        return true;
    }
    common_link_name(name).is_some_and(|target| claims(&target))
}

/// A reference's `???` marker and `sub` (the path below the Common
/// directory, ending in `/` or empty) when its directory is
/// `model/character/uniform/common/` + exactly three characters + `/`, in
/// any letter case; `None` otherwise.
fn reference_into_common(directory: &str) -> Option<(&str, &str)> {
    let prefix = directory.get(..COMMON_PREFIX.len())?;
    if !prefix.eq_ignore_ascii_case(COMMON_PREFIX) {
        return None;
    }
    let (marker, sub) = directory[COMMON_PREFIX.len()..].split_once('/')?;
    (marker.chars().count() == 3).then_some((marker, sub))
}

/// Which kind of folder a `.mtl` lands in: a player folder, a shared
/// `Boots/`/`Gloves/` one, or `Common/` (a shared folder's rules for its
/// references).
#[derive(Clone, Copy, PartialEq, Eq)]
enum FolderKind {
    /// `Players/<folder>/`.
    Player,
    /// `Boots/<name>/`, `Gloves/<name>/`.
    Shared,
    /// `Common/`.
    Common,
}

/// The model folder an output path sits in (`Players/07 - A`,
/// `Boots/Crocs`, `Gloves/Mitts`, `Common`) and the file's path below it,
/// or `None`.
fn model_folder_of(output: &str) -> Option<(FolderKind, &str, &str)> {
    let (top, rest) = output.split_once('/')?;
    let kind = match top {
        "Players" => FolderKind::Player,
        "Boots" | "Gloves" => FolderKind::Shared,
        "Common" => return Some((FolderKind::Common, "Common", rest)),
        _ => return None,
    };
    let (folder, rest) = rest.split_once('/')?;
    Some((kind, &output[..top.len() + 1 + folder.len()], rest))
}

/// `directory`'s and `file_name`'s kit magic respelled (`Spelling::Path`,
/// every `u0???p`/`u0???g` token in each).
fn respell_reference(directory: &mut String, file_name: &mut String, team: TeamId) {
    if let Cow::Owned(new) = respell(directory, Spelling::Path, team) {
        *directory = new;
    }
    if let Cow::Owned(new) = respell(file_name, Spelling::Path, team) {
        *file_name = new;
    }
}

/// The `???` of `directory`'s Common path respelled the team's ID in
/// place: `prefix_len` the `model/character/…/common/` prefix's length
/// (either format's) and `rest` what follows it (`sub` for an `.mtl`,
/// `sub` + `sourceimages/` for an FMDL).
fn respell_marker(directory: &mut String, prefix_len: usize, rest: &str, team: TeamId) {
    *directory = format!("{}{:03}/{rest}", &directory[..prefix_len], team.get());
}

/// The `common_reference_kept` on a `.mtl`/FMDL file (`written` the
/// reference as written): the file the export ships without.
fn kept(written: &str, input: &str) -> Message {
    tool_message(
        Code::CommonReferenceKept,
        Scope::File {
            export_id: INPUT,
            path: path_scope(input),
        },
        Disposition::Keep,
        vec![("reference", written.to_owned())],
    )
}

/// The values one file's references share: the model folder it sits in
/// (`Players/07 - A`, `Boots/Crocs`, `Common`), its folder kind, its path
/// below it (`rest`), the `Common/` output paths, the plans' copies, its
/// input path and the team.
#[derive(Clone, Copy)]
struct FileContext<'a> {
    /// `Players/`/`Boots/`/`Gloves/`/`Common/` (a shared `.fmdl` is
    /// mapped to `Player` by `fmdl_entry`).
    kind: FolderKind,
    /// The model folder's output path.
    model_folder: &'a str,
    /// The file's path below `model_folder` (`x.mtl`, `realkit/x.fmdl`).
    rest: &'a str,
    /// The `Common/` output paths (respelled).
    common: &'a [String],
    /// The plans' (input, output) copies.
    copies: &'a [(String, String)],
    /// The file's input path, for the finding's scope.
    input: &'a str,
    /// The resolved team.
    team: TeamId,
}

/// What the caller's reference is (`common_decision`'s outcomes differ):
/// a team-Common reference (`common/<???>/…`), kept and reported when it
/// cannot link; or a game directory's stem, linked when it can and
/// otherwise silent.
enum Reference {
    /// `common/<???>/…`.
    TeamCommon,
    /// `face/real/…`, `boots/…`.
    GameDir,
}

/// One FMDL texture entry's outcome, class by class (`DECISIONS.md`
/// 2026-10-18 "an FMDL's texture entries, class by class"): a `dummy_`
/// stem is kept as written (the game substitutes it); a `./` or empty
/// directory has its kit magic respelled only; the game's kit texture
/// folder is kept; the team's Common (`common/<3>/[sub/]sourceimages/`)
/// is linked as an `.mtl`'s Common reference (a `Common/` FMDL's kept,
/// `???` respelled, reported when `Common/` lacks the texture); any other
/// directory below `common/` is kept; every other directory becomes `./`,
/// the `Common/` root's texture linked when the folder holds none.
fn fmdl_entry(
    path: &mut fmdl::ops::paths::TexturePath,
    ctx: &FileContext,
    planned: &mut BTreeMap<String, String>,
    findings: &mut Vec<Message>,
) {
    if folded_stem(&path.file_name).starts_with("dummy_") {
        return;
    }
    if path.directory.is_empty() || path.directory.starts_with("./") {
        respell_reference(&mut path.directory, &mut path.file_name, ctx.team);
        return;
    }
    // The game's kit texture folder is kept (`fmdl_texture_kit_path`), a
    // directory below it a kit texture's too.
    if vtree::fold_name(&path.directory).starts_with(&vtree::fold_name(FOX_KIT_TEXTURE)) {
        return;
    }
    let written = format!("{}{}", path.directory, path.file_name);
    if path
        .directory
        .get(..FOX_COMMON_PREFIX.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(FOX_COMMON_PREFIX))
    {
        let Some((_marker, sub)) = reference_into_fox_common(&path.directory) else {
            // `common/sourceimages/`, `common/vg/sourceimages/`: not the
            // team's — kept as written.
            return;
        };
        let sub = respell(sub, Spelling::Path, ctx.team).into_owned();
        if let Cow::Owned(name) = respell(&path.file_name, Spelling::Path, ctx.team) {
            path.file_name = name;
        }
        // A shared folder's FMDL is linked as a player folder's (the Fox
        // compile reads a shared folder's links); `Common/`'s own keeps.
        let decision_ctx = FileContext {
            kind: match ctx.kind {
                FolderKind::Shared => FolderKind::Player,
                other => other,
            },
            ..*ctx
        };
        match common_decision(
            &decision_ctx,
            Reference::TeamCommon,
            &sub,
            &path.file_name,
            &*planned,
            &written,
            findings,
        ) {
            Some(common_rest) => {
                planned.insert(
                    format!("{}/{common_rest}.common", ctx.model_folder),
                    common_rest,
                );
                path.directory = format!("./{sub}");
            }
            None => respell_marker(
                &mut path.directory,
                FOX_COMMON_PREFIX.len(),
                &format!("{sub}sourceimages/"),
                ctx.team,
            ),
        }
        return;
    }
    // `Common/` keeps its other game directories; a player or shared
    // folder's every other directory becomes `./`, the `Common/` root's
    // texture linked when the folder holds none of the stem.
    if ctx.kind == FolderKind::Common {
        return;
    }
    if let Cow::Owned(name) = respell(&path.file_name, Spelling::Path, ctx.team) {
        path.file_name = name;
    }
    path.directory = "./".to_owned();
    if let Some(common_rest) = common_decision(
        ctx,
        Reference::GameDir,
        "",
        &path.file_name,
        &*planned,
        &written,
        findings,
    ) {
        planned.insert(
            format!("{}/{common_rest}.common", ctx.model_folder),
            common_rest,
        );
    }
}

/// The `Common/` path the link names (`sub` + the respelled stem + the
/// member's extension for a set, the matched file's own path for any
/// other stem) when `sub`/`stem`/`set_reference` resolves in `common` and
/// no texture or texture link of the stem (or of its set) stands on the
/// nearest-first way from the file's folder up to `model_folder` — `None`
/// when kept (`common_reference_kept` pushed, `written` the reference as
/// written, for a class-4 caller's kept reference or a shared folder's/
/// `Common/`'s own missing one) or, for `game_dir`, when `Common/` lacks
/// the stem or the folder claims it and nothing is reported.
fn common_decision(
    ctx: &FileContext,
    reference: Reference,
    sub: &str,
    file_name: &str,
    planned: &BTreeMap<String, String>,
    written: &str,
    findings: &mut Vec<Message>,
) -> Option<String> {
    let game_dir = matches!(reference, Reference::GameDir);
    let stem = folded_stem(file_name);
    let respelled_stem = file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(file_name);
    let reference = format!("{sub}{file_name}");
    let set_reference = matches!(kit_path_token(&reference), Some((KitToken::Reference, _)))
        .then_some(reference.as_str());
    let sub_folded = vtree::fold_name(sub);
    // The `Common/` candidates: the `sub` directory's members of a `kitN`
    // reference's set, or the one texture of the stem.
    let matching: Vec<&str> = ctx
        .common
        .iter()
        .filter_map(|file| {
            let rest = under(file, "common")?;
            let name = rest.rsplit('/').next().unwrap_or(rest);
            if classify(name) != FileKind::Texture {
                return None;
            }
            if let Some(reference) = set_reference {
                kit_set_member(reference, rest).map(|_| rest)
            } else {
                let (folder, name) = match rest.rsplit_once('/') {
                    Some((folder, name)) => (format!("{folder}/"), name),
                    None => (String::new(), rest),
                };
                (vtree::fold_name(&folder) == sub_folded && folded_stem(name) == stem)
                    .then_some(rest)
            }
        })
        .collect();
    if !game_dir && ctx.kind != FolderKind::Player {
        // A shared folder's or `Common/` file's Common reference is
        // respelled, no link — reported when `Common/` lacks it.
        if matching.is_empty() {
            findings.push(kept(written, ctx.input));
        }
        return None;
    }
    let common_rest: Option<String> = if set_reference.is_some() {
        // A link to a set reaches its members whatever the link's own
        // extension (`common_link_files` matches them by stem), so the
        // first member's names it.
        matching.first().map(|rest| {
            let extension = rest.rsplit_once('.').map_or("", |(_, ext)| ext);
            format!("{sub}{respelled_stem}.{extension}")
        })
    } else {
        match matching.as_slice() {
            &[rest] => Some(rest.to_owned()),
            _ => None,
        }
    };
    let Some(common_rest) = common_rest else {
        if !game_dir {
            findings.push(kept(written, ctx.input));
        }
        return None;
    };
    // A `.mtl`/`.fmdl` in a subfolder naming a Common subfolder: the path
    // below it and the link mirroring `Common/` cannot agree.
    if !game_dir && !sub.is_empty() && ctx.rest.contains('/') {
        findings.push(kept(written, ctx.input));
        return None;
    }
    // The clash check: the folders a bare name looks in, nearest first
    // (the file's folder up to the model folder), or for a `sub` the one
    // `model folder/sub` folder.
    let file_folder = match ctx.rest.rsplit_once('/') {
        Some((dir, _)) => format!("{}/{dir}", ctx.model_folder),
        None => ctx.model_folder.to_owned(),
    };
    let mut folders = Vec::new();
    if sub.is_empty() {
        let mut folder = file_folder.as_str();
        loop {
            folders.push(folder.to_owned());
            if folder == ctx.model_folder {
                break;
            }
            folder = folder
                .rsplit_once('/')
                .map(|(parent, _)| parent)
                .expect("a folder below the model folder has a parent");
        }
    } else {
        folders.push(format!("{}/{sub}", ctx.model_folder));
    }
    let clashing = |folder: &str| {
        let folder = vtree::fold_name(folder.trim_end_matches('/'));
        ctx.copies.iter().any(|(_, output)| {
            let Some((parent, name)) = output.rsplit_once('/') else {
                return false;
            };
            if vtree::fold_name(parent) != folder {
                return false;
            }
            claims_stem(name, &stem, set_reference)
        }) || planned.iter().any(|(link, rest)| {
            *rest != common_rest
                && link.rsplit_once('/').is_some_and(|(parent, name)| {
                    vtree::fold_name(parent) == folder && claims_stem(name, &stem, set_reference)
                })
        })
    };
    if folders.iter().any(|folder| clashing(folder)) {
        if !game_dir {
            findings.push(kept(written, ctx.input));
        }
        return None;
    }
    Some(common_rest)
}

/// What step 7 makes of the migrated model folders' `.mtl` and `.fmdl`
/// files for a team export — one pass, one `planned` link map, so a link is
/// planned once and each format's clash walk sees the other's links:
/// `copies` the plans' (input, output) list (the player plan's and the
/// carried `Common/` files'), `common` the `Common/` output paths (the
/// respelled names the links are spelled after).
pub(crate) fn material_references(
    copies: &[(String, String)],
    common: &[String],
    team: TeamId,
    mut read: impl FnMut(&str) -> Result<Vec<u8>, SourceFailure>,
) -> MaterialRewrites {
    let mut rewrites = MaterialRewrites::default();
    let mut failures: BTreeMap<String, Vec<Message>> = BTreeMap::new();
    let mut planned: BTreeMap<String, String> = BTreeMap::new();

    for (input, output) in copies {
        let Some((kind, model_folder, rest)) = model_folder_of(output) else {
            continue;
        };
        let Some(name) = rest.rsplit('/').next() else {
            continue;
        };
        let fmdl = match classify(name) {
            FileKind::Mtl => false,
            FileKind::Model(ModelFormat::Fmdl) => true,
            _ => continue,
        };
        let ctx = FileContext {
            kind,
            model_folder,
            rest,
            common,
            copies,
            input,
            team,
        };
        // The copy loop's own read reports a failure; a file that does not
        // read is the compile's to report, copied as it is.
        let Ok(bytes) = read(input) else {
            continue;
        };
        if fmdl {
            let Ok(mut file) = FmdlFile::read(&bytes) else {
                continue;
            };
            let mut file_findings = Vec::new();
            let changed = match fmdl::ops::paths::rewrite_texture_paths(&mut file, |path| {
                fmdl_entry(path, &ctx, &mut planned, &mut file_findings);
            }) {
                Ok(changed) => changed,
                // The string table could not hold the respelled paths:
                // copied as it is.
                Err(_) => continue,
            };
            if changed > 0 {
                rewrites.written.push((output.clone(), file.write()));
            }
            if !file_findings.is_empty() {
                failures
                    .entry(input.clone())
                    .or_default()
                    .extend(file_findings);
            }
            continue;
        }
        let wrapped = wezlib::is_wrapped(&bytes);
        let Ok(mut set) = MaterialSet::read(&bytes) else {
            continue;
        };
        let mut file_findings = Vec::new();
        let changed = rewrite_texture_paths(&mut set, |path| {
            // `./` or no directory: a local reference — respell it and stop.
            if path.directory.is_empty() || path.directory.starts_with("./") {
                respell_reference(&mut path.directory, &mut path.file_name, ctx.team);
                return;
            }
            // Every other reference (a game directory's) stays as written,
            // kit magic included.
            let Some((_marker, sub)) = reference_into_common(&path.directory) else {
                return;
            };
            // The reference as written, for the finding's context.
            let written = format!("{}{}", path.directory, path.file_name);
            // The `sub` and the file name respelled (`Spelling::Path`).
            let sub = respell(sub, Spelling::Path, ctx.team).into_owned();
            if let Cow::Owned(name) = respell(&path.file_name, Spelling::Path, ctx.team) {
                path.file_name = name;
            }
            // A `dummy_` stem is the game's; its `???` alone is respelled,
            // no lookup (MA1-2: the game supplies it).
            if folded_stem(&path.file_name).starts_with("dummy_") {
                respell_marker(&mut path.directory, COMMON_PREFIX.len(), &sub, ctx.team);
                return;
            }
            match common_decision(
                &ctx,
                Reference::TeamCommon,
                &sub,
                &path.file_name,
                &planned,
                &written,
                &mut file_findings,
            ) {
                // `./` + `sub` + the name as respelled, and the link for
                // the `Common/` file's own name, planned once.
                Some(common_rest) => {
                    planned.insert(
                        format!("{}/{common_rest}.common", ctx.model_folder),
                        common_rest,
                    );
                    path.directory = format!("./{sub}");
                }
                None => respell_marker(&mut path.directory, COMMON_PREFIX.len(), &sub, ctx.team),
            }
        });
        if changed > 0 {
            let out = set.write();
            rewrites.written.push((
                output.clone(),
                if wrapped { wezlib::compress(&out) } else { out },
            ));
        }
        if !file_findings.is_empty() {
            failures
                .entry(input.clone())
                .or_default()
                .extend(file_findings);
        }
    }
    rewrites.links = planned.keys().cloned().collect();

    for (input, _) in copies {
        if let Some(messages) = failures.remove(input.as_str()) {
            rewrites.findings.extend(messages);
        }
    }
    rewrites.findings.extend(failures.into_values().flatten());
    rewrites
}

#[cfg(test)]
mod tests {
    use studio_core::Scope;

    use super::*;

    /// The resolved team, 714.
    fn team() -> TeamId {
        TeamId::new(714).unwrap()
    }

    /// A `.mtl` of one material holding one sampler per `paths` entry.
    fn mtl(paths: &[&str]) -> Vec<u8> {
        let samplers: String = paths
            .iter()
            .map(|path| {
                format!("        <sampler name=\"DiffuseMap\" path=\"{path}\" srgb=\"1\" minfilter=\"linear\" magfilter=\"linear\" uaddr=\"clamp\" vaddr=\"clamp\"/>\n")
            })
            .collect();
        format!(
            "<materialset>\n    <material name=\"face_phong\" shader=\"Wrinkle_Face\">\n{samplers}    </material>\n</materialset>\n"
        )
        .into_bytes()
    }

    /// An FMDL of the `konami_highneck` fixture with its four texture
    /// entries set from `entries` (`(directory, file_name)` pairs, in
    /// table order).
    fn fmdl(entries: &[(&str, &str)]) -> Vec<u8> {
        let mut file = FmdlFile::read(include_bytes!(
            "../../../libs/fmdl/tests/fixtures/konami_highneck.fmdl"
        ))
        .unwrap();
        let mut entries = entries.iter();
        fmdl::ops::paths::rewrite_texture_paths(&mut file, |path| {
            if let Some((directory, name)) = entries.next() {
                path.directory = (*directory).to_owned();
                path.file_name = (*name).to_owned();
            }
        })
        .unwrap();
        file.write()
    }

    /// The `(directory, file_name)` entries of `output`'s written FMDL.
    fn fmdl_paths(rewrites: &MaterialRewrites, output: &str) -> Vec<(String, String)> {
        let (_, bytes) = rewrites
            .written
            .iter()
            .find(|(path, _)| path == output)
            .expect("the FMDL was rewritten");
        let file = FmdlFile::read(bytes).unwrap();
        fmdl::ops::paths::texture_paths(&file)
            .unwrap()
            .iter()
            .map(|path| (path.directory.clone(), path.file_name.clone()))
            .collect()
    }

    #[test]
    fn an_fmdl_common_reference_becomes_the_stem_and_the_link() {
        let face = fmdl(&[(
            "/Assets/pes16/model/character/common/000/sourceimages/",
            "hair.dds",
        )]);
        let rewrites = plan(
            &[(
                "Faces/XXX07 - A/face_high.fmdl",
                "Players/07 - A/face_high.fmdl",
            )],
            &["Common/hair.dds"],
            &[("Faces/XXX07 - A/face_high.fmdl", &face)],
        );
        assert_eq!(
            fmdl_paths(&rewrites, "Players/07 - A/face_high.fmdl")[0],
            ("./".to_owned(), "hair.dds".to_owned())
        );
        assert_eq!(
            rewrites.links,
            ["Players/07 - A/hair.dds.common".to_owned()]
        );
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn a_shared_fmdl_common_subfolder_links_below_the_folder() {
        let boots = fmdl(&[(
            "/Assets/pes16/model/character/common/000/Boots/sourceimages/",
            "x.dds",
        )]);
        let rewrites = plan(
            &[("boots", "Boots/Crocs/boots.fmdl")],
            &["Common/Boots/x.dds"],
            &[("boots", &boots)],
        );
        assert_eq!(
            fmdl_paths(&rewrites, "Boots/Crocs/boots.fmdl")[0],
            ("./Boots/".to_owned(), "x.dds".to_owned())
        );
        assert_eq!(
            rewrites.links,
            ["Boots/Crocs/Boots/x.dds.common".to_owned()]
        );
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn an_fmdl_common_kit_reference_links_the_set() {
        let face = fmdl(&[(
            "/Assets/pes16/model/character/common/000/u0XXXp0/sourceimages/",
            "armor_bsm",
        )]);
        let rewrites = plan(
            &[(
                "Faces/XXX07 - A/face_high.fmdl",
                "Players/07 - A/face_high.fmdl",
            )],
            &["Common/kit1/armor_bsm.dds"],
            &[("Faces/XXX07 - A/face_high.fmdl", &face)],
        );
        assert_eq!(
            fmdl_paths(&rewrites, "Players/07 - A/face_high.fmdl")[0],
            ("./kitN/".to_owned(), "armor_bsm".to_owned())
        );
        assert_eq!(
            rewrites.links,
            ["Players/07 - A/kitN/armor_bsm.dds.common".to_owned()]
        );
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn an_fmdl_common_keeper_reference_respells_the_team() {
        let face = fmdl(&[(
            "/Assets/pes16/model/character/common/000/sourceimages/",
            "u0XXXg1.dds",
        )]);
        let rewrites = plan(
            &[(
                "Faces/XXX07 - A/face_high.fmdl",
                "Players/07 - A/face_high.fmdl",
            )],
            &["Common/u0714g1.dds"],
            &[("Faces/XXX07 - A/face_high.fmdl", &face)],
        );
        assert_eq!(
            fmdl_paths(&rewrites, "Players/07 - A/face_high.fmdl")[0],
            ("./".to_owned(), "u0714g1.dds".to_owned())
        );
        assert_eq!(
            rewrites.links,
            ["Players/07 - A/u0714g1.dds.common".to_owned()]
        );
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn an_fmdl_common_reference_whose_texture_is_missing_is_kept() {
        let face = fmdl(&[(
            "/Assets/pes16/model/character/common/000/sourceimages/",
            "hair.dds",
        )]);
        let rewrites = plan(
            &[(
                "Faces/XXX07 - A/face_high.fmdl",
                "Players/07 - A/face_high.fmdl",
            )],
            &[],
            &[("Faces/XXX07 - A/face_high.fmdl", &face)],
        );
        assert_eq!(
            fmdl_paths(&rewrites, "Players/07 - A/face_high.fmdl")[0],
            (
                "/Assets/pes16/model/character/common/714/sourceimages/".to_owned(),
                "hair.dds".to_owned()
            )
        );
        assert!(rewrites.links.is_empty());
        assert_eq!(rewrites.findings.len(), 1);
        assert_eq!(
            rewrites.findings[0].code.code.as_ref(),
            "common_reference_kept"
        );
        assert_eq!(
            rewrites.findings[0].context,
            vec![(
                "reference".to_owned(),
                "/Assets/pes16/model/character/common/000/sourceimages/hair.dds".to_owned()
            )]
        );
    }

    #[test]
    fn fmdl_entries_the_classes_keep_are_not_written() {
        let face = fmdl(&[
            (
                "/Assets/pes16/model/character/common/000/sourceimages/",
                "dummy_kit.dds",
            ),
            (
                "/Assets/pes16/model/character/uniform/texture/",
                "u0XXXp1.dds",
            ),
            (
                "/Assets/pes16/model/character/common/sourceimages/",
                "zero.dds",
            ),
        ]);
        let rewrites = plan(
            &[(
                "Faces/XXX07 - A/face_high.fmdl",
                "Players/07 - A/face_high.fmdl",
            )],
            &[],
            &[("Faces/XXX07 - A/face_high.fmdl", &face)],
        );
        // Nothing respelled, nothing linked, nothing reported: no rewrite.
        assert!(rewrites.written.is_empty());
        assert!(rewrites.links.is_empty());
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn an_fmdl_other_directory_becomes_the_model_s_own_folder() {
        let face = fmdl(&[(
            "/Assets/pes16/model/character/face/real/00000/sourceimages/",
            "skin.dds",
        )]);
        // `skin.dds` in the folder: `./`, no link.
        let rewrites = plan(
            &[
                (
                    "Faces/XXX07 - A/face_high.fmdl",
                    "Players/07 - A/face_high.fmdl",
                ),
                ("Faces/XXX07 - A/skin.dds", "Players/07 - A/skin.dds"),
            ],
            &[],
            &[
                ("Faces/XXX07 - A/face_high.fmdl", &face),
                ("Faces/XXX07 - A/skin.dds", b"s"),
            ],
        );
        assert_eq!(
            fmdl_paths(&rewrites, "Players/07 - A/face_high.fmdl")[0],
            ("./".to_owned(), "skin.dds".to_owned())
        );
        assert!(rewrites.links.is_empty());
        // `skin.dds` only in `Common/`: `./` and the link.
        let rewrites = plan(
            &[(
                "Faces/XXX07 - A/face_high.fmdl",
                "Players/07 - A/face_high.fmdl",
            )],
            &["Common/skin.dds"],
            &[("Faces/XXX07 - A/face_high.fmdl", &face)],
        );
        assert_eq!(
            fmdl_paths(&rewrites, "Players/07 - A/face_high.fmdl")[0],
            ("./".to_owned(), "skin.dds".to_owned())
        );
        assert_eq!(
            rewrites.links,
            ["Players/07 - A/skin.dds.common".to_owned()]
        );
        // Nowhere: `./`, no link, no finding.
        let rewrites = plan(
            &[(
                "Faces/XXX07 - A/face_high.fmdl",
                "Players/07 - A/face_high.fmdl",
            )],
            &[],
            &[("Faces/XXX07 - A/face_high.fmdl", &face)],
        );
        assert_eq!(
            fmdl_paths(&rewrites, "Players/07 - A/face_high.fmdl")[0],
            ("./".to_owned(), "skin.dds".to_owned())
        );
        assert!(rewrites.links.is_empty());
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn an_fmdl_kit_name_links_when_a_member_sits_in_the_folder() {
        let face = fmdl(&[(
            "/Assets/pes16/model/character/face/real/00000/sourceimages/",
            "u0805p0.dds",
        )]);
        let rewrites = plan(
            &[
                (
                    "Faces/XXX07 - A/face_high.fmdl",
                    "Players/07 - A/face_high.fmdl",
                ),
                ("Faces/XXX07 - A/kit1.dds", "Players/07 - A/kit1.dds"),
            ],
            &[],
            &[
                ("Faces/XXX07 - A/face_high.fmdl", &face),
                ("Faces/XXX07 - A/kit1.dds", b"k"),
            ],
        );
        // `kit1.dds` in the folder is a member of `kitN`'s set: `./` `kitN`,
        // no link.
        assert_eq!(
            fmdl_paths(&rewrites, "Players/07 - A/face_high.fmdl")[0],
            ("./".to_owned(), "kitN.dds".to_owned())
        );
        assert!(rewrites.links.is_empty());
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn a_common_fmdl_keeps_its_common_references() {
        let boots = fmdl(&[(
            "/Assets/pes16/model/character/common/000/sourceimages/",
            "x.dds",
        )]);
        let rewrites = plan(
            &[("common", "Common/boots.fmdl")],
            &["Common/x.dds"],
            &[("common", &boots)],
        );
        assert_eq!(
            fmdl_paths(&rewrites, "Common/boots.fmdl")[0],
            (
                "/Assets/pes16/model/character/common/714/sourceimages/".to_owned(),
                "x.dds".to_owned()
            )
        );
        assert!(rewrites.links.is_empty());
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn an_fmdl_that_does_not_read_is_copied_as_it_is() {
        let rewrites = plan(
            &[(
                "Faces/XXX07 - A/face_high.fmdl",
                "Players/07 - A/face_high.fmdl",
            )],
            &[],
            &[("Faces/XXX07 - A/face_high.fmdl", b"fmdl")],
        );
        assert!(rewrites.written.is_empty());
        assert!(rewrites.links.is_empty());
        assert!(rewrites.findings.is_empty());
    }

    /// `material_references` over `copies` and `common`, each `.mtl`'s bytes
    /// behind `read`; a listed copy without bytes is a test bug.
    fn plan(copies: &[(&str, &str)], common: &[&str], bytes: &[(&str, &[u8])]) -> MaterialRewrites {
        let copies: Vec<(String, String)> = copies
            .iter()
            .map(|(from, to)| ((*from).to_owned(), (*to).to_owned()))
            .collect();
        let common: Vec<String> = common.iter().map(|file| (*file).to_owned()).collect();
        let bytes: BTreeMap<&str, &[u8]> = bytes.iter().copied().collect();
        material_references(&copies, &common, team(), |path| {
            bytes
                .get(path)
                .map(|bytes| Ok(bytes.to_vec()))
                .unwrap_or_else(|| panic!("a listed copy has no bytes: {path}"))
        })
    }

    #[test]
    fn a_local_reference_s_kit_magic_is_respelled() {
        let rewrites = plan(
            &[("face", "Players/07 - A/face.mtl")],
            &[],
            &[("face", &mtl(&["./skin_u0758p0.dds"]))],
        );
        assert_eq!(rewrites.links.len(), 0);
        let text = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(text.contains("path=\"./skin_kitN.dds\""), "{text}");
    }

    #[test]
    fn a_game_directory_reference_is_left_as_written() {
        let rewrites = plan(
            &[("face", "Players/07 - A/face.mtl")],
            &[],
            &[(
                "face",
                &mtl(&["model/character/uniform/texture/u0814p1.dds"]),
            )],
        );
        // Nothing changed: no rewrite at all.
        assert!(rewrites.written.is_empty());
        assert!(rewrites.links.is_empty());
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn a_common_keeper_reference_names_the_respelled_file() {
        let rewrites = plan(
            &[("face", "Players/07 - A/face.mtl")],
            &["Common/u0714g1.dds"],
            &[(
                "face",
                &mtl(&["model/character/uniform/common/XXX/u0XXXg1.dds"]),
            )],
        );
        let text = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(text.contains("path=\"./u0714g1.dds\""), "{text}");
        assert_eq!(
            rewrites.links,
            vec!["Players/07 - A/u0714g1.dds.common".to_owned()]
        );
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn a_common_dummy_stem_gets_its_marker_respelled_alone() {
        let rewrites = plan(
            &[("face", "Players/07 - A/face.mtl")],
            &[],
            &[(
                "face",
                &mtl(&["model/character/uniform/common/XXX/dummy_kit.dds"]),
            )],
        );
        let text = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(
            text.contains("path=\"model/character/uniform/common/714/dummy_kit.dds\""),
            "{text}"
        );
        assert!(rewrites.links.is_empty());
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn a_common_kit_reference_links_the_set_s_members() {
        let rewrites = plan(
            &[("face", "Players/07 - A/face.mtl")],
            &["Common/kit1.dds", "Common/kit2.dds"],
            &[(
                "face",
                &mtl(&["model/character/uniform/common/XXX/u0XXXp0.dds"]),
            )],
        );
        let text = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(text.contains("path=\"./kitN.dds\""), "{text}");
        assert_eq!(
            rewrites.links,
            vec!["Players/07 - A/kitN.dds.common".to_owned()]
        );
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn a_kit_set_reference_with_a_member_in_the_folder_is_kept() {
        let rewrites = plan(
            &[
                ("face", "Players/07 - A/face.mtl"),
                ("kit1", "Players/07 - A/kit1.dds"),
            ],
            &["Common/kit1.dds", "Common/kit2.dds"],
            &[
                (
                    "face",
                    &mtl(&["model/character/uniform/common/XXX/u0XXXp0.dds"]),
                ),
                ("kit1", b"dds"),
            ],
        );
        assert_eq!(rewrites.links.len(), 0);
        let text = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(
            text.contains("path=\"model/character/uniform/common/714/kitN.dds\""),
            "{text}"
        );
        assert_eq!(rewrites.findings.len(), 1);
        assert_eq!(
            rewrites.findings[0].code.code.as_ref(),
            "common_reference_kept"
        );
    }

    #[test]
    fn a_shared_folder_mtl_respells_a_local_reference() {
        let rewrites = plan(
            &[("boots", "Boots/Crocs/boots.mtl")],
            &[],
            &[("boots", &mtl(&["./u0XXXp1.dds"]))],
        );
        let text = written_text(&rewrites, "Boots/Crocs/boots.mtl");
        assert!(text.contains("path=\"./kit1.dds\""), "{text}");
    }

    #[test]
    fn a_shared_folder_mtl_reference_to_a_missing_common_file_is_kept() {
        let rewrites = plan(
            &[("boots", "Boots/Crocs/boots.mtl")],
            &[],
            &[(
                "boots",
                &mtl(&["model/character/uniform/common/XXX/hair.dds"]),
            )],
        );
        assert!(rewrites.links.is_empty());
        let text = written_text(&rewrites, "Boots/Crocs/boots.mtl");
        assert!(
            text.contains("path=\"model/character/uniform/common/714/hair.dds\""),
            "{text}"
        );
        assert_eq!(rewrites.findings.len(), 1);
        assert_eq!(
            rewrites.findings[0].code.code.as_ref(),
            "common_reference_kept"
        );
        assert_eq!(
            rewrites.findings[0].context,
            vec![(
                "reference".to_owned(),
                "model/character/uniform/common/XXX/hair.dds".to_owned()
            )]
        );
    }

    #[test]
    fn two_references_in_one_mtl_link_their_own_common_files() {
        let rewrites = plan(
            &[("face", "Players/07 - A/face.mtl")],
            &["Common/hair.dds", "Common/jessie/hair.dds"],
            &[(
                "face",
                &mtl(&[
                    "model/character/uniform/common/XXX/hair.dds",
                    "model/character/uniform/common/XXX/jessie/hair.dds",
                ]),
            )],
        );
        let text = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(text.contains("path=\"./hair.dds\""), "{text}");
        assert!(text.contains("path=\"./jessie/hair.dds\""), "{text}");
        assert_eq!(
            rewrites.links,
            [
                "Players/07 - A/hair.dds.common".to_owned(),
                "Players/07 - A/jessie/hair.dds.common".to_owned(),
            ]
        );
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn a_common_mtl_keeps_its_common_references() {
        let rewrites = plan(
            &[("mtl", "Common/hair.mtl")],
            &["Common/hair.dds"],
            &[(
                "mtl",
                &mtl(&["model/character/uniform/common/XXX/hair.dds"]),
            )],
        );
        let text = written_text(&rewrites, "Common/hair.mtl");
        assert!(
            text.contains("path=\"model/character/uniform/common/714/hair.dds\""),
            "{text}"
        );
        assert!(rewrites.links.is_empty());
        assert!(rewrites.findings.is_empty());
    }

    /// The one output path's `.mtl` text, parsed back.
    fn written_text(rewrites: &MaterialRewrites, output: &str) -> String {
        let (_, bytes) = rewrites
            .written
            .iter()
            .find(|(path, _)| path == output)
            .expect("the .mtl was rewritten");
        String::from_utf8(bytes.clone()).unwrap()
    }

    #[test]
    fn a_reference_into_common_becomes_the_bare_name_and_the_link() {
        let face = mtl(&["model/character/uniform/common/XXX/hair.dds"]);
        let rewrites = plan(
            &[
                ("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl"),
                ("Faces/XXX07 - A/face.fmdl", "Players/07 - A/face.fmdl"),
            ],
            &["Common/hair.dds"],
            &[
                ("Faces/XXX07 - A/face.mtl", &face),
                ("Faces/XXX07 - A/face.fmdl", b"x"),
            ],
        );
        let out = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(out.contains("./hair.dds"), "{out}");
        assert_eq!(
            rewrites.links,
            ["Players/07 - A/hair.dds.common".to_owned()]
        );
        // The marker's own spelling and case likewise.
        let face = mtl(&["MODEL/Character/Uniform/Common/714/Hair.DDS"]);
        let rewrites = plan(
            &[("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl")],
            &["Common/hair.dds"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        let out = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(out.contains("./Hair.DDS"), "{out}");
        // `Common/hair.png` is found by stem and the link names its file.
        let rewrites = plan(
            &[("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl")],
            &["Common/hair.png"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        assert_eq!(
            rewrites.links,
            ["Players/07 - A/hair.png.common".to_owned()]
        );
    }

    #[test]
    fn a_common_subfolder_reference_links_at_the_path_below_the_folder() {
        let face = mtl(&["model/character/uniform/common/XXX/jessie/hair.dds"]);
        let rewrites = plan(
            &[
                ("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl"),
                (
                    "Faces/XXX07 - A/realkit/x.mtl",
                    "Players/07 - A/realkit/x.mtl",
                ),
            ],
            &["Common/jessie/hair.dds"],
            &[
                ("Faces/XXX07 - A/face.mtl", &face),
                ("Faces/XXX07 - A/realkit/x.mtl", &face),
            ],
        );
        // The root `.mtl` gets the path below the model folder and the
        // mirrored link; the subfolder `.mtl`'s is kept, `???` respelled.
        let out = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(out.contains("./jessie/hair.dds"), "{out}");
        assert_eq!(
            rewrites.links,
            ["Players/07 - A/jessie/hair.dds.common".to_owned()]
        );
        let out = written_text(&rewrites, "Players/07 - A/realkit/x.mtl");
        assert!(
            out.contains("model/character/uniform/common/714/jessie/hair.dds"),
            "{out}"
        );
        let kept = rewrites
            .findings
            .iter()
            .find(|m| m.code.code.as_ref() == "common_reference_kept")
            .expect("common_reference_kept expected");
        assert_eq!(
            kept.context,
            vec![(
                "reference".to_owned(),
                "model/character/uniform/common/XXX/jessie/hair.dds".to_owned()
            )]
        );
        assert_eq!(
            kept.scope,
            Scope::File {
                export_id: INPUT,
                path: path_scope("Faces/XXX07 - A/realkit/x.mtl")
            }
        );
    }

    #[test]
    fn a_folder_file_of_the_stem_keeps_the_reference() {
        // `Players/07 - A/` already holds `hair.png`: the link would clash
        // or lose to the nearer file — kept.
        let face = mtl(&["model/character/uniform/common/XXX/hair.dds"]);
        let rewrites = plan(
            &[
                ("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl"),
                ("Faces/XXX07 - A/hair.png", "Players/07 - A/hair.png"),
            ],
            &["Common/hair.dds"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        let out = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(
            out.contains("model/character/uniform/common/714/hair.dds"),
            "{out}"
        );
        assert!(rewrites.links.is_empty());
        // From a subfolder `.mtl` the bare name looks up to the model folder,
        // so the root's `hair.png` keeps it too.
        let rewrites = plan(
            &[
                (
                    "Faces/XXX07 - A/realkit/apc.mtl",
                    "Players/07 - A/realkit/apc.mtl",
                ),
                ("Faces/XXX07 - A/hair.png", "Players/07 - A/hair.png"),
            ],
            &["Common/hair.dds"],
            &[("Faces/XXX07 - A/realkit/apc.mtl", &face)],
        );
        let out = written_text(&rewrites, "Players/07 - A/realkit/apc.mtl");
        assert!(
            out.contains("model/character/uniform/common/714/hair.dds"),
            "{out}"
        );
        assert!(rewrites.links.is_empty());
        // A model's link of that stem claims no texture: the link is made.
        let rewrites = plan(
            &[
                ("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl"),
                (
                    "Faces/XXX07 - A/hair.model.common",
                    "Players/07 - A/hair.model.common",
                ),
            ],
            &["Common/hair.dds"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        assert_eq!(
            rewrites.links,
            ["Players/07 - A/hair.dds.common".to_owned()]
        );
        // Two `.mtl` files naming one `Common/` file share its one link.
        let two = plan(
            &[
                ("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl"),
                ("Faces/XXX07 - A/other.mtl", "Players/07 - A/other.mtl"),
            ],
            &["Common/hair.dds"],
            &[
                ("Faces/XXX07 - A/face.mtl", &face),
                ("Faces/XXX07 - A/other.mtl", &face),
            ],
        );
        assert_eq!(two.links, ["Players/07 - A/hair.dds.common".to_owned()]);
        for output in ["Players/07 - A/face.mtl", "Players/07 - A/other.mtl"] {
            assert!(
                written_text(&two, output).contains("./hair.dds"),
                "{output}"
            );
        }
    }

    #[test]
    fn an_mtl_naming_nothing_or_unparsable_stays_as_it_is() {
        // `./face.dds` and `model/character/face/common/` are not the team's
        // Common — nothing changes, so nothing is written.
        let unchanged = mtl(&["./face.dds", "model/character/face/common/face_eyelash.dds"]);
        let bad = b"mtl".to_vec();
        let plan = plan(
            &[
                ("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl"),
                ("Faces/XXX07 - A/bad.mtl", "Players/07 - A/bad.mtl"),
            ],
            &["Common/hair.dds"],
            &[
                ("Faces/XXX07 - A/face.mtl", &unchanged),
                ("Faces/XXX07 - A/bad.mtl", &bad),
            ],
        );
        assert!(plan.written.is_empty());
        assert!(plan.links.is_empty());
        assert!(plan.findings.is_empty());
    }

    #[test]
    fn the_common_file_must_match_both_folder_and_stem() {
        // `Common/` holds `beard.dds` (the folder, not the stem) and
        // `jessie/hair.dds` (the stem, not the folder): `hair` is kept.
        let face = mtl(&["model/character/uniform/common/XXX/hair.dds"]);
        let rewrites = plan(
            &[("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl")],
            &["Common/beard.dds", "Common/jessie/hair.dds"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        let out = written_text(&rewrites, "Players/07 - A/face.mtl");
        assert!(
            out.contains("model/character/uniform/common/714/hair.dds"),
            "{out}"
        );
        assert!(rewrites.links.is_empty());
    }

    #[test]
    fn two_common_files_named_from_one_folder_both_link() {
        let face = mtl(&[
            "model/character/uniform/common/XXX/hair.dds",
            "model/character/uniform/common/XXX/beard.dds",
        ]);
        let rewrites = plan(
            &[("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl")],
            &["Common/beard.dds", "Common/hair.dds"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        assert_eq!(
            rewrites.links,
            [
                "Players/07 - A/beard.dds.common".to_owned(),
                "Players/07 - A/hair.dds.common".to_owned()
            ]
        );
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn a_shared_folder_mtl_keeps_the_reference_with_the_team_id() {
        // A `.mtl` of a shared `Boots/`/`Gloves/` folder keeps its Common
        // references as written but `???`, respelled the team's ID — no
        // finding and no link (the pre-Fox compile reads no shared folder's
        // `.common` links).
        let boots = mtl(&["model/character/uniform/common/XXX/hair.dds"]);
        let rewrites = plan(
            &[("Boots/k0430 - Crocs/boots.mtl", "Boots/Crocs/boots.mtl")],
            &["Common/hair.dds"],
            &[("Boots/k0430 - Crocs/boots.mtl", &boots)],
        );
        let out = written_text(&rewrites, "Boots/Crocs/boots.mtl");
        assert!(
            out.contains("model/character/uniform/common/714/hair.dds"),
            "{out}"
        );
        assert!(rewrites.links.is_empty());
        assert!(rewrites.findings.is_empty());
    }

    #[test]
    fn a_non_texture_of_the_stem_in_common_is_no_link_target() {
        // `Common/hair.mtl` shares the stem but is no texture: kept and
        // warned, no link.
        let face = mtl(&["model/character/uniform/common/XXX/hair.dds"]);
        let rewrites = plan(
            &[("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl")],
            &["Common/hair.mtl"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        assert!(rewrites.links.is_empty());
        assert_eq!(
            rewrites
                .findings
                .iter()
                .map(|m| m.code.code.as_ref())
                .collect::<Vec<_>>(),
            ["common_reference_kept"]
        );
    }

    #[test]
    fn two_textures_of_the_stem_in_one_common_directory_keep_the_reference() {
        // `Common/hair.dds` beside `Common/hair.png`: both textures of the
        // stem in the directory, so no single file the link could spell —
        // kept and warned.
        let face = mtl(&["model/character/uniform/common/XXX/hair.dds"]);
        let rewrites = plan(
            &[("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl")],
            &["Common/hair.dds", "Common/hair.png"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        assert!(rewrites.links.is_empty());
        assert_eq!(
            rewrites
                .findings
                .iter()
                .map(|m| m.code.code.as_ref())
                .collect::<Vec<_>>(),
            ["common_reference_kept"]
        );
    }

    #[test]
    fn a_rewritten_mtl_keeps_the_wesys_wrap() {
        let face = wezlib::compress(&mtl(&["model/character/uniform/common/XXX/hair.dds"]));
        let rewrites = plan(
            &[("Faces/XXX07 - A/face.mtl", "Players/07 - A/face.mtl")],
            &["Common/hair.dds"],
            &[("Faces/XXX07 - A/face.mtl", &face)],
        );
        let (_, bytes) = rewrites
            .written
            .iter()
            .find(|(path, _)| path == "Players/07 - A/face.mtl")
            .expect("the .mtl was rewritten");
        assert!(wezlib::is_wrapped(bytes));
        let out = String::from_utf8(wezlib::decompress(bytes).unwrap()).unwrap();
        assert!(out.contains("./hair.dds"), "{out}");
    }
}
