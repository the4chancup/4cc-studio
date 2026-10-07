//! Parity against the compiled output for the same export: compile
//! `fixtures/tracer/studio/egg Midcup Tracer` through the `compile` command and
//! compare the CPK with the reference tree in `fixtures/tracer/red/`, one
//! explicit row per reference entry under the tiers of
//! `docs/plans/team_compiler/testing.md`.

mod common;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use common::Sandbox;
use fpk::FpkFile;
use studio_core::{PipelineEvent, Severity};

/// How one entry of the reference tree is compared.
enum Row {
    /// Tier 1 leaf: same bytes (`Row` carries no tier of its own; leaf entries
    /// compare per `compare_leaf`).
    Exact,
    /// Byte-identical at a different path (the path-mapping of tier 1/2).
    Relocated(&'static str),
    /// An FPK whose entry-name set must match (tier 3); contents are checked
    /// per member in `compare_fpk`.
    Container,
    /// `Container` at a different path: the planned ID replacing the ID the
    /// old export's folder name carried.
    RelocatedContainer(&'static str),
    /// Not produced yet (tier 4): the reason.
    NotProduced(&'static str),
}

/// The per-player common texture every one of the player's models points at.
const COMMON_SHIRT: &str = "Asset/model/character/common/792/05 - The Chad Stormworks Player/sourceimages/#windx11/shirt.ftex";

/// One row per reference entry. The produced rows are also the allowlist our
/// CPK's entries are checked against. Red compiled the boots and gloves under
/// the ID their folders were named with, `2180`; the Studio format has no
/// ID-named folders and slot 05 of team 792 owns the planned ID 3745.
const TABLE: &[(&str, Row)] = &[
    (
        "Asset/model/character/boots/k2180/#Win/boots.fpk",
        Row::RelocatedContainer("Asset/model/character/boots/k3745/#Win/boots.fpk"),
    ),
    (
        "Asset/model/character/boots/k2180/#Win/boots.fpkd",
        Row::Relocated("Asset/model/character/boots/k3745/#Win/boots.fpkd"),
    ),
    (
        "Asset/model/character/boots/k2180/#windx11/boots.fpk.xml",
        Row::NotProduced("the Studio format drops the source-side fpk.xml"),
    ),
    // The same texture the face uses: one entry serves both.
    (
        "Asset/model/character/boots/k2180/#windx11/shirt.ftex",
        Row::Relocated(COMMON_SHIRT),
    ),
    (
        "Asset/model/character/face/real/79205/#Win/face.fpk",
        Row::Container,
    ),
    (
        "Asset/model/character/face/real/79205/#Win/face.fpkd",
        Row::Exact,
    ),
    (
        "Asset/model/character/face/real/79205/sourceimages/#windx11/face.fpk.xml",
        Row::NotProduced("the Studio format drops the source-side fpk.xml"),
    ),
    (
        "Asset/model/character/face/real/79205/sourceimages/#windx11/shirt.ftex",
        Row::Relocated(COMMON_SHIRT),
    ),
    (
        "Asset/model/character/glove/g2180/#Win/glove.fpk",
        Row::RelocatedContainer("Asset/model/character/glove/g3745/#Win/glove.fpk"),
    ),
    (
        "Asset/model/character/glove/g2180/#Win/glove.fpkd",
        Row::Relocated("Asset/model/character/glove/g3745/#Win/glove.fpkd"),
    ),
    (
        "Asset/model/character/glove/g2180/#windx11/glove.fpk.xml",
        Row::NotProduced("the Studio format drops the source-side fpk.xml"),
    ),
    (
        "Asset/model/character/uniform/texture/#windx11/u0792g1.ftex",
        Row::Exact,
    ),
    (
        "common/character0/model/character/uniform/team/792/792_DEF_GK1st_realUni.bin",
        Row::Exact,
    ),
    (
        "common/character0/model/character/uniform/team/UniColor.bin",
        Row::Exact,
    ),
    (
        "common/character0/model/character/uniform/team/UniformParameter.bin",
        Row::Exact,
    ),
    ("common/etc/TeamColor.bin", Row::Exact),
    ("common/render/symbol/player/79205.dds", Row::Exact),
];

fn tree(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                pending.push(entry.path());
            } else {
                let path = entry
                    .path()
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(path, fs::read(entry.path()).unwrap());
            }
        }
    }
    files
}

fn archive(path: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut cpk = cpk::CpkArchive::open(fs::File::open(path).unwrap()).unwrap();
    let entries = cpk.entries().to_vec();
    let mut files = BTreeMap::new();
    for entry in &entries {
        files.insert(entry.path.clone(), cpk.read(entry).unwrap());
    }
    files
}

/// One leaf entry: an `.ftex` compares by what `ftex_to_dds` returns for each
/// side (tier 2 — the chunk compression is a deflate encoder's choice, and
/// `ftex` declares byte identity with zlib's output a non-goal); anything else
/// compares by bytes (tier 1).
fn compare_leaf(failures: &mut Vec<String>, name: &str, ours: Option<&[u8]>, reference: &[u8]) {
    match ours {
        None => failures.push(format!("{name}: missing from our CPK")),
        Some(our_bytes) if name.ends_with(".ftex") => {
            let (our_info, reference_info) = (
                ftex::info(our_bytes).unwrap(),
                ftex::info(reference).unwrap(),
            );
            if our_info != reference_info {
                failures.push(format!(
                    "tier2 {name}: headers differ (ours {our_info:?}, reference {reference_info:?})"
                ));
            }
            if ftex::ftex_to_dds(our_bytes).unwrap() != ftex::ftex_to_dds(reference).unwrap() {
                failures.push(format!("tier2 {name}: decoded DDS content differs"));
            }
        }
        Some(our_bytes) => {
            if our_bytes != reference {
                failures.push(format!("tier1 {name}: bytes differ"));
            }
        }
    }
}

/// The texture-path table's relocated directory is an intentional layout
/// difference (per-player common): the relocated stem's directory is
/// normalized away on both sides before the decoded comparison, per
/// `testing.md`'s tier 2. The dummy textures' directories are kept literal so
/// the team-ID substitution in them is still checked.
fn decoded_model(bytes: &[u8]) -> fmdl::Model {
    let file = fmdl::FmdlFile::read(bytes).unwrap();
    let mut model = fmdl::Model::from_file(&file).unwrap();
    for material in &mut model.materials {
        for (_, texture) in &mut material.textures {
            if texture.file_name == "shirt.dds" {
                texture.directory = "<relocated>".to_owned();
            }
        }
    }
    model
}

/// Every `shirt.dds` texture reference of the FMDL `bytes` resolves to a file
/// of `compiled` (`testing.md` "Texture locations": rewritten references
/// resolve to the relocated files). The reference's directory has the FMDL
/// form `/Assets/pes16/<folder>/`; the emitted texture sits at the CPK path
/// `Asset/<folder>/#windx11/<stem>.ftex`.
fn check_resolved(
    failures: &mut Vec<String>,
    name: &str,
    bytes: &[u8],
    compiled: &BTreeMap<String, Vec<u8>>,
) {
    let file = fmdl::FmdlFile::read(bytes).unwrap();
    let model = fmdl::Model::from_file(&file).unwrap();
    for material in &model.materials {
        for (_, texture) in &material.textures {
            if texture.file_name != "shirt.dds" {
                continue;
            }
            let stem = Path::new(&texture.file_name)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or(texture.file_name.as_str());
            let directory = texture
                .directory
                .strip_prefix("/Assets/pes16")
                .unwrap_or(texture.directory.as_str());
            let resolved = format!("Asset{directory}#windx11/{stem}.ftex");
            if !compiled.contains_key(&resolved) {
                failures.push(format!(
                    "{name}: {}{} resolves to {resolved}, not in our CPK",
                    texture.directory, texture.file_name
                ));
            }
        }
    }
}

fn compare_fpk(
    failures: &mut Vec<String>,
    name: &str,
    ours: &[u8],
    reference: &[u8],
    compiled: &BTreeMap<String, Vec<u8>>,
) {
    let ours = FpkFile::read(ours).unwrap();
    let reference = FpkFile::read(reference).unwrap();
    let our_names: BTreeSet<&str> = ours.entries().map(|(name, _)| name).collect();
    let reference_names: BTreeSet<&str> = reference.entries().map(|(name, _)| name).collect();
    if our_names != reference_names {
        failures.push(format!(
            "tier3 {name}: entry-name set differs (ours {our_names:?}, reference {reference_names:?})"
        ));
    }
    for (entry, reference_bytes) in reference.entries() {
        let member = format!("{name}/{entry}");
        match ours.get(entry) {
            None => failures.push(format!("tier3 {member}: missing")),
            Some(our_bytes) if entry.ends_with(".fmdl") => {
                check_resolved(failures, &member, our_bytes, compiled);
                if decoded_model(our_bytes) != decoded_model(reference_bytes) {
                    failures.push(format!("tier2 {member}: decoded models differ"));
                }
            }
            Some(our_bytes) => compare_leaf(failures, &member, Some(our_bytes), reference_bytes),
        }
    }
}

#[test]
fn the_tracer_bullet_matches_the_reference_tree() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tracer");
    let sandbox = Sandbox::new("parity");
    let output = sandbox.root.join("parity output");
    let settings = format!(
        "[common]\npes_version = 21\n[team-compiler]\noutput_folder_path = '{}'\n",
        output.display()
    );
    let exports_root = fixture.join("studio");
    // Nothing to install into: the reference is the CPK itself.
    let run = sandbox.run(
        &settings,
        &["compile", "--no-deploy", exports_root.to_str().unwrap()],
    );
    assert_eq!(run.exit_code(), 0);
    // Notes only, but for the two player tables: with no PES install there is no table to
    // build on, so the tracer's boots and gloves rows are left out (`pipeline.md` "Bins
    // accumulation"), as Red, which wrote no table, never had them.
    let mut warnings = Vec::new();
    for envelope in &run.events {
        if let PipelineEvent::Message(message) = &envelope.event
            && message.severity >= Severity::Warning
        {
            warnings.push((message.code.code.to_string(), message.context.clone()));
        }
    }
    let table_missing = |table: &str| {
        (
            "player_table_missing".to_owned(),
            vec![
                ("table".to_owned(), table.to_owned()),
                ("rows".to_owned(), "1".to_owned()),
            ],
        )
    };
    assert_eq!(
        warnings,
        [
            table_missing("BootsList.bin"),
            table_missing("GloveList.bin")
        ],
        "the tracer compiles with notes only, but for the two player tables"
    );

    let ours = archive(&output.join("4cc_99_test.cpk"));
    let reference = tree(&fixture.join("red"));

    let mut failures = Vec::new();
    for (path, row) in TABLE {
        let reference_bytes = &reference[*path];
        match row {
            Row::Exact => compare_leaf(
                &mut failures,
                path,
                ours.get(*path).map(Vec::as_slice),
                reference_bytes,
            ),
            Row::Relocated(our_path) => compare_leaf(
                &mut failures,
                path,
                ours.get(*our_path).map(Vec::as_slice),
                reference_bytes,
            ),
            Row::Container => match ours.get(*path) {
                None => failures.push(format!("tier3 {path}: missing from our CPK")),
                Some(our_bytes) => {
                    compare_fpk(&mut failures, path, our_bytes, reference_bytes, &ours)
                }
            },
            Row::RelocatedContainer(our_path) => match ours.get(*our_path) {
                None => failures.push(format!("tier3 {path}: missing from our CPK")),
                Some(our_bytes) => {
                    compare_fpk(&mut failures, path, our_bytes, reference_bytes, &ours)
                }
            },
            Row::NotProduced(reason) => {
                if ours.contains_key(*path) {
                    failures.push(format!("tier4 {path}: produced anyway ({reason})"));
                }
            }
        }
    }

    for path in ours.keys() {
        let produced = TABLE.iter().any(|(reference_path, row)| match row {
            Row::Relocated(our_path) | Row::RelocatedContainer(our_path) => path == our_path,
            Row::NotProduced(_) => false,
            Row::Exact | Row::Container => path == reference_path,
        });
        if !produced {
            failures.push(format!(
                "our CPK holds {path}, not in the table's produced rows"
            ));
        }
    }
    for path in reference.keys() {
        if !TABLE.iter().any(|(row_path, _)| path == row_path) {
            failures.push(format!(
                "reference holds {path}, which the table does not list"
            ));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
