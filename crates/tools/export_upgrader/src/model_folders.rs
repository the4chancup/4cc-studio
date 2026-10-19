//! Model-folder mode (`plans/export_upgrader.md` "Model-folder mode"): a
//! folder of old-format model folders (the item folders that used to live
//! under `Faces/`, `Boots/` and `Gloves/`) upgraded one per output folder,
//! to drop into a Studio export. The copies are staged in the export layout
//! step 7's functions read (`Players/<name>/…`, `Boots/<name>/…`,
//! `Common/…`), the write alone taking the `Players`/`Boots`/`Gloves` level
//! off.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use anyhow::anyhow;
use studio_core::{CliError, Disposition, Message, Scope, ToolContext};
use vtree::ScopePath;

use crate::carried::under;
use crate::cli::{ERRORS, UpgradeModelsArgs};
use crate::face_xml::{face_xml_rewrites, kept_common_files};
use crate::kit_magic::{drop_collisions, respell_outputs};
use crate::materials::material_references;
use crate::messages::{Code, INPUT, OUTPUT, extract_failed, read_failed, tool_message};
use crate::model_names::rename_models;
use crate::players::{ItemKind, clean_folder_name, parse_item_name, path_scope, slot_of};
use crate::source::{self, OldExport};
use crate::upgrade::{
    Identity, RunEvents, aborted, invalid, resolve_input, verdict, write_output_file,
};

/// Where an output folder of model-folder mode goes once dropped into a
/// Studio export.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Destination {
    /// A player folder, into `Players/`.
    Players,
    /// A shared boots folder, into `Boots/`.
    Boots,
    /// A shared gloves folder, into `Gloves/`.
    Gloves,
    /// The upgraded `Common/`, at the export's root.
    Root,
}

impl Destination {
    /// The report's `into` value: `Players`, `Boots`, `Gloves` or `root`.
    pub(crate) fn into_value(self) -> &'static str {
        match self {
            Destination::Players => "Players",
            Destination::Boots => "Boots",
            Destination::Gloves => "Gloves",
            Destination::Root => "root",
        }
    }

    /// The staged output path's first segment.
    fn staged_top(self) -> &'static str {
        match self {
            Destination::Players => "Players",
            Destination::Boots => "Boots",
            Destination::Gloves => "Gloves",
            Destination::Root => "Common",
        }
    }
}

/// One output folder: its name at the output's top level, and where it goes.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PlacedFolder {
    pub(crate) name: String,
    pub(crate) destination: Destination,
}

/// What model-folder mode makes of the input's top level.
#[derive(Debug, Default)]
pub(crate) struct ModelFolderPlan {
    /// Input path → staged output path, in the export layout step 7's
    /// functions read (`Players/15 - Snuffy/face.fmdl`, `Boots/Crocs/x.model`,
    /// `Common/hair.dds`).
    pub(crate) copies: Vec<(String, String)>,
    /// The bare `ingame_face` markers to write, staged paths.
    pub(crate) markers: Vec<String>,
    /// The output folders, in input-name order.
    pub(crate) folders: Vec<PlacedFolder>,
    /// The `unmigrated_content` findings on the input.
    pub(crate) findings: Vec<Message>,
}

/// The group's destination and the name its folder is published under,
/// `None` when no cleaned name exists (a real folder name always has one,
/// but `clean_folder_name`'s type allows it). `Common` in any letter case;
/// else `k`/`g` + four digits by the item grammar (or `g` and three); else
/// a face folder, `NN - Name` when its slot reads or the folder's own name.
fn placed_folder(name: &str) -> Option<(Destination, String)> {
    if name.eq_ignore_ascii_case("common") {
        return Some((Destination::Root, "Common".to_owned()));
    }
    for (kind, destination) in [
        (ItemKind::Boots, Destination::Boots),
        (ItemKind::Gloves, Destination::Gloves),
    ] {
        if let Some((id, name_part)) = parse_item_name(kind, name) {
            let name = clean_folder_name(&name_part)
                .unwrap_or_else(|| format!("{}{id:04}", kind.letter()));
            return Some((destination, name));
        }
    }
    if let Some(nn) = slot_of(name) {
        // The name part is what follows the five characters, its leading
        // spaces and hyphens trimmed, as `face_folders` reads it.
        let name_part = name.get(5..).unwrap_or("").trim_start_matches([' ', '-']);
        let name = match clean_folder_name(name_part) {
            Some(part) => format!("{nn:02} - {part}"),
            None => format!("{nn:02}"),
        };
        return Some((Destination::Players, name));
    }
    clean_folder_name(name).map(|name| (Destination::Players, name))
}

/// The input's top level resolved into the output folders (plan
/// "Model-folder mode"): files grouped by their first segment, `BTreeMap`
/// giving the input-name order; a file directly in the input, a folder
/// holding no model file directly in it and a folder whose published name an
/// earlier kept folder already took are unmigrated content.
pub(crate) fn model_folder_plan(files: &[String]) -> ModelFolderPlan {
    let mut plan = ModelFolderPlan::default();
    let mut groups: BTreeMap<String, Vec<&String>> = BTreeMap::new();
    for file in files {
        match file.split_once('/') {
            Some((top, _)) => groups.entry(top.to_owned()).or_default().push(file),
            None => plan.findings.push(tool_message(
                Code::UnmigratedContent,
                Scope::File {
                    export_id: INPUT,
                    path: path_scope(file),
                },
                Disposition::DropFile,
                vec![],
            )),
        }
    }
    // The published names taken so far, folded: they all sit at the output's
    // top level whatever their destination, so a face folder `Crocs` clashes
    // with `k0430 - Crocs` and `common` with `Common`.
    let mut taken: BTreeSet<String> = BTreeSet::new();
    for (name, group) in groups {
        let refused = |plan: &mut ModelFolderPlan| {
            plan.findings.push(tool_message(
                Code::UnmigratedContent,
                Scope::Folder {
                    export_id: INPUT,
                    path: path_scope(&name),
                },
                Disposition::DropFolder,
                vec![],
            ));
        };
        let Some((destination, published_name)) = placed_folder(&name) else {
            refused(&mut plan);
            continue;
        };
        // A folder holding no `.fmdl`/`.model` directly in it is no model
        // folder: one level, one folder per model (`Common` aside).
        let has_model = destination == Destination::Root
            || group.iter().any(|file| {
                let rest = &file[name.len() + 1..];
                let lowered = rest.to_ascii_lowercase();
                !rest.contains('/') && (lowered.ends_with(".fmdl") || lowered.ends_with(".model"))
            });
        if !has_model {
            refused(&mut plan);
            continue;
        }
        if !taken.insert(vtree::fold_name(&published_name)) {
            refused(&mut plan);
            continue;
        }
        let mut marker = false;
        for file in &group {
            let rest = &file[name.len() + 1..];
            let lowered = rest.to_ascii_lowercase();
            // In a player folder the old `ingame_face` spellings normalize
            // to the bare marker, one per folder; every other file keeps
            // its name (no portrait rename: no `Portraits/` to compare it
            // with).
            if destination == Destination::Players
                && !rest.contains('/')
                && (lowered == "ingame_face" || lowered == "ingame_face.txt")
            {
                marker = true;
                continue;
            }
            let staged = match destination {
                Destination::Root => format!("Common/{rest}"),
                Destination::Players | Destination::Boots | Destination::Gloves => {
                    format!("{}/{published_name}/{rest}", destination.staged_top())
                }
            };
            plan.copies.push(((*file).clone(), staged));
        }
        if marker {
            plan.markers
                .push(format!("Players/{published_name}/ingame_face"));
        }
        plan.folders.push(PlacedFolder {
            name: published_name,
            destination,
        });
    }
    plan
}

/// The output path a staged one is written at: `Players/`, `Boots/` or
/// `Gloves/` taken off, a `Common/` path as it is.
pub(crate) fn published(staged: &str) -> &str {
    match staged.split_once('/') {
        Some(("Players" | "Boots" | "Gloves", rest)) => rest,
        _ => staged,
    }
}

/// Runs `upgrade-models`: the input checked, the folder listed as it is, the
/// output folder made, the top level planned and migrated through step 7's
/// functions on the staged layout, the copies written where they publish,
/// each output folder's destination reported, and the verdict: `upgrade`'s
/// "CLI" order without the team, the roster or the kits.
pub(crate) fn run(args: &UpgradeModelsArgs, ctx: &ToolContext) -> Result<u8, CliError> {
    // "CLI": the input must exist and be a folder.
    let path = &args.folder;
    if !path.exists() {
        return Err(invalid(anyhow!(
            "{}: no such folder or file",
            path.display()
        )));
    }
    if !path.is_dir() {
        return Err(invalid(anyhow!("{}: not a folder", path.display())));
    }
    let path = resolve_input(path)?;
    let name = source::source_name(&path);
    let mut events = RunEvents::new(ctx);
    events.started(INPUT, &name);

    // Step 1, as it is: the folder's top level is the model folders.
    let mut export = match OldExport::open_as_is(&path) {
        Ok(export) => export,
        Err(failure) => {
            events.message(extract_failed(INPUT, failure));
            return Ok(ERRORS);
        }
    };

    // "CLI": the output folder, refused when it exists or cannot be made.
    let output = args.output.clone().unwrap_or_else(|| {
        path.parent()
            .expect("the input is absolute and not a filesystem root (resolve_input)")
            .join(format!("{name} upgraded"))
    });
    if output.exists() {
        return Err(aborted(anyhow!(
            "{}: the output folder already exists",
            output.display()
        )));
    }
    fs::create_dir_all(&output).map_err(|error| {
        aborted(anyhow!(
            "{}: cannot create the output folder: {error}",
            output.display()
        ))
    })?;
    let output_name = output
        .file_name()
        .expect("a created folder's path ends in its name")
        .to_string_lossy()
        .into_owned();
    events.started(OUTPUT, &output_name);

    // Step 7's chain on the staged copies, exactly `upgrade`'s: the kit
    // magic's respells first, the `Common/` names a `face.xml` or a link
    // keeps, then the model renames and the output-path collisions, the
    // `.mtl` rewrites and the `face.xml` respells against the one list.
    let identity = Identity::ModelFolders;
    let mut plan = model_folder_plan(&export.files);
    let staged = respell_outputs(std::mem::take(&mut plan.copies), &identity);
    let kept = kept_common_files(&staged, &identity, |path| export.read(path));
    let (copies, dropped) = drop_collisions(rename_models(staged, &kept));
    plan.copies = copies;
    plan.findings.extend(dropped);
    // `Common/` for the materials: the respelled output paths the links are
    // spelled after.
    let common: Vec<String> = plan
        .copies
        .iter()
        .filter(|(_, output)| under(output, "common").is_some())
        .map(|(_, output)| output.clone())
        .collect();
    let materials = material_references(&plan.copies, &common, &identity, |path| export.read(path));
    let face_xml = face_xml_rewrites(&plan.copies, &identity, |path| export.read(path));
    let mut any_unmigrated = false;
    for message in plan.findings.into_iter().chain(materials.findings) {
        if message.code.code.as_ref() == Code::UnmigratedContent.as_str() {
            any_unmigrated = true;
        }
        events.message(message);
    }
    if any_unmigrated {
        events.unmigrated();
    }

    // The writes publish the staged paths: the `Players/`/`Boots/`/`Gloves/`
    // level comes off, each output folder droppable as it stands.
    for (from, to) in &plan.copies {
        if materials.written.iter().any(|(output, _)| output == to)
            || face_xml.iter().any(|(output, _)| output == to)
        {
            // A rewritten `.mtl` or `face.xml` replaces its input's copy.
            continue;
        }
        match export.read(from) {
            Ok(bytes) => write_output_file(&output, published(to), &bytes)?,
            Err(failure) => {
                events.message(read_failed(INPUT, failure, Disposition::DropFile));
            }
        }
    }
    for (path, bytes) in &materials.written {
        write_output_file(&output, published(path), bytes)?;
    }
    for (path, bytes) in &face_xml {
        write_output_file(&output, published(path), bytes)?;
    }
    for link in &materials.links {
        write_output_file(&output, published(link), b"")?;
    }
    for marker in &plan.markers {
        write_output_file(&output, published(marker), b"")?;
    }

    // The report names, per output folder, where it goes once dropped in.
    for folder in &plan.folders {
        events.message(tool_message(
            Code::ModelFolderDestination,
            Scope::Folder {
                export_id: OUTPUT,
                path: ScopePath::new(&folder.name)
                    .expect("a published name is a cleaned folder name, a valid segment"),
            },
            Disposition::Keep,
            vec![("into", folder.destination.into_value().to_owned())],
        ));
    }

    // "Verification": the staged listing's validation (next brief).

    // "CLI": the verdict line and the exit code, as `upgrade`'s.
    let (code, exit_code) = verdict(events.worst(), events.dropped());
    events.message(tool_message(code, Scope::Run, Disposition::Keep, vec![]));
    Ok(exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn files(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    fn placed(name: &str, destination: Destination) -> PlacedFolder {
        PlacedFolder {
            name: name.to_owned(),
            destination,
        }
    }

    #[test]
    fn the_input_folders_become_output_folders_with_their_destinations() {
        let plan = model_folder_plan(&files(&[
            "Common/hair.dds",
            "Snuffy/hair.model",
            "Snuffy/ingame_face.txt",
            "XXX15 - Snuffy/face.fmdl",
            "XXX15 - Snuffy/u0XXXp1.dds",
            "k0430 - Crocs/gloveL_crocs.model",
            "k0431/boots.model",
        ]));
        assert_eq!(
            plan.folders,
            [
                placed("Common", Destination::Root),
                placed("Snuffy", Destination::Players),
                placed("15 - Snuffy", Destination::Players),
                placed("Crocs", Destination::Boots),
                placed("k0431", Destination::Boots),
            ]
        );
        assert_eq!(
            plan.copies,
            [
                ("Common/hair.dds".to_owned(), "Common/hair.dds".to_owned()),
                (
                    "Snuffy/hair.model".to_owned(),
                    "Players/Snuffy/hair.model".to_owned()
                ),
                (
                    "XXX15 - Snuffy/face.fmdl".to_owned(),
                    "Players/15 - Snuffy/face.fmdl".to_owned()
                ),
                (
                    "XXX15 - Snuffy/u0XXXp1.dds".to_owned(),
                    "Players/15 - Snuffy/u0XXXp1.dds".to_owned()
                ),
                (
                    "k0430 - Crocs/gloveL_crocs.model".to_owned(),
                    "Boots/Crocs/gloveL_crocs.model".to_owned()
                ),
                (
                    "k0431/boots.model".to_owned(),
                    "Boots/k0431/boots.model".to_owned()
                ),
            ]
        );
        assert_eq!(plan.markers, ["Players/Snuffy/ingame_face".to_owned()]);
        assert!(plan.findings.is_empty(), "{:?}", plan.findings);
    }

    #[test]
    fn a_gloves_folder_is_named_by_its_name_part_or_its_id() {
        let plan = model_folder_plan(&files(&[
            "g0502 - Mitts/glove_l.model",
            "g567/glove_l.model",
        ]));
        assert_eq!(
            plan.folders,
            [
                placed("Mitts", Destination::Gloves),
                placed("g0567", Destination::Gloves),
            ]
        );
        assert_eq!(
            plan.copies,
            [
                (
                    "g0502 - Mitts/glove_l.model".to_owned(),
                    "Gloves/Mitts/glove_l.model".to_owned()
                ),
                (
                    "g567/glove_l.model".to_owned(),
                    "Gloves/g0567/glove_l.model".to_owned()
                ),
            ]
        );
        assert!(plan.findings.is_empty(), "{:?}", plan.findings);
    }

    /// The refusal of a group: `unmigrated_content`, folder scope on `INPUT`,
    /// `DropFolder`, no context.
    fn assert_refused(message: &Message, folder: &str) {
        assert_eq!(message.code.code.as_ref(), "unmigrated_content");
        assert_eq!(
            message.scope,
            Scope::Folder {
                export_id: INPUT,
                path: path_scope(folder),
            }
        );
        assert_eq!(message.disposition, Disposition::DropFolder);
        assert!(message.context.is_empty());
    }

    #[test]
    fn a_folder_holding_no_model_file_directly_in_it_is_refused() {
        // A two-level layout and a folder of plain files alike hold no
        // `.fmdl`/`.model` directly in them.
        let plan = model_folder_plan(&files(&["players/Ann/face.fmdl", "Textures/u0XXXp1.dds"]));
        assert_eq!(plan.findings.len(), 2, "{:?}", plan.findings);
        assert_refused(&plan.findings[0], "Textures");
        assert_refused(&plan.findings[1], "players");
        assert!(plan.folders.is_empty());
        assert!(plan.copies.is_empty());
    }

    #[test]
    fn the_first_folder_by_input_name_wins_a_name_clash() {
        // `71415 - A` and `XXX15 - A` both publish `15 - A`.
        let plan = model_folder_plan(&files(&["71415 - A/face.fmdl", "XXX15 - A/face.fmdl"]));
        assert_eq!(plan.folders, [placed("15 - A", Destination::Players)]);
        assert_eq!(
            plan.copies,
            [(
                "71415 - A/face.fmdl".to_owned(),
                "Players/15 - A/face.fmdl".to_owned()
            )]
        );
        assert_eq!(plan.findings.len(), 1, "{:?}", plan.findings);
        assert_refused(&plan.findings[0], "XXX15 - A");

        // A face folder `Crocs` and `k0430 - Crocs` both publish `Crocs`
        // beside each other at the output's top level.
        let plan = model_folder_plan(&files(&["Crocs/face.fmdl", "k0430 - Crocs/boots.model"]));
        assert_eq!(plan.folders, [placed("Crocs", Destination::Players)]);
        assert_eq!(
            plan.copies,
            [(
                "Crocs/face.fmdl".to_owned(),
                "Players/Crocs/face.fmdl".to_owned()
            )]
        );
        assert_eq!(plan.findings.len(), 1, "{:?}", plan.findings);
        assert_refused(&plan.findings[0], "k0430 - Crocs");

        // Letter case aside: `k0430 - crocs` still publishes `Crocs`'s name.
        let plan = model_folder_plan(&files(&["Crocs/face.fmdl", "k0430 - crocs/boots.model"]));
        assert_eq!(plan.folders, [placed("Crocs", Destination::Players)]);
        assert_eq!(plan.findings.len(), 1, "{:?}", plan.findings);
        assert_refused(&plan.findings[0], "k0430 - crocs");
    }

    #[test]
    fn a_file_directly_in_the_input_is_unmigrated_content() {
        let plan = model_folder_plan(&files(&["readme.txt"]));
        assert_eq!(plan.findings.len(), 1, "{:?}", plan.findings);
        let message = &plan.findings[0];
        assert_eq!(message.code.code.as_ref(), "unmigrated_content");
        assert_eq!(
            message.scope,
            Scope::File {
                export_id: INPUT,
                path: path_scope("readme.txt"),
            }
        );
        assert_eq!(message.disposition, Disposition::DropFile);
        assert!(message.context.is_empty());
        assert!(plan.folders.is_empty());
        assert!(plan.copies.is_empty());
    }

    #[test]
    fn a_face_folder_without_a_slot_keeps_its_own_name() {
        // `XXX30`'s "slot" is past 23: no slot, the name is the folder's own.
        let plan = model_folder_plan(&files(&["XXX30 - A/face.fmdl", "XXX15/face.fmdl"]));
        assert_eq!(
            plan.folders,
            [
                placed("15", Destination::Players),
                placed("XXX30 - A", Destination::Players),
            ]
        );
        assert_eq!(
            plan.copies,
            [
                (
                    "XXX15/face.fmdl".to_owned(),
                    "Players/15/face.fmdl".to_owned()
                ),
                (
                    "XXX30 - A/face.fmdl".to_owned(),
                    "Players/XXX30 - A/face.fmdl".to_owned()
                ),
            ]
        );
        assert!(plan.findings.is_empty(), "{:?}", plan.findings);
    }

    #[test]
    fn published_takes_the_staged_top_level_off() {
        assert_eq!(
            published("Players/15 - Snuffy/face.fmdl"),
            "15 - Snuffy/face.fmdl"
        );
        assert_eq!(published("Boots/Crocs/x"), "Crocs/x");
        assert_eq!(published("Common/hair.dds"), "Common/hair.dds");
    }
}
