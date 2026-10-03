//! Parity against the compiled output for the same export: compile
//! `fixtures/tracer/studio/egg Tracer` through the `compile` command and
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
    /// Not produced in Phase 3 (tier 4): the reason.
    NotProduced(&'static str),
}

/// One row per reference entry. The produced rows are also the allowlist our
/// CPK's entries are checked against.
const TABLE: &[(&str, Row)] = &[
    (
        "Asset/model/character/boots/k2180/#Win/boots.fpk",
        Row::NotProduced("boots are Phase 4 content"),
    ),
    (
        "Asset/model/character/boots/k2180/#Win/boots.fpkd",
        Row::NotProduced("boots are Phase 4 content"),
    ),
    (
        "Asset/model/character/boots/k2180/#windx11/boots.fpk.xml",
        Row::NotProduced("the Studio format drops the source-side fpk.xml"),
    ),
    (
        "Asset/model/character/boots/k2180/#windx11/shirt.ftex",
        Row::NotProduced("the boots texture rides in the boots output, Phase 4"),
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
        Row::Relocated(
            "Asset/model/character/common/792/05 - The Chad Stormworks Player/sourceimages/#windx11/shirt.ftex",
        ),
    ),
    (
        "Asset/model/character/glove/g2180/#Win/glove.fpk",
        Row::NotProduced("gloves are Phase 4 content"),
    ),
    (
        "Asset/model/character/glove/g2180/#Win/glove.fpkd",
        Row::NotProduced("gloves are Phase 4 content"),
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
        Row::NotProduced("bins beyond the kit configs are Phase 4"),
    ),
    (
        "common/character0/model/character/uniform/team/UniformParameter.bin",
        Row::Exact,
    ),
    (
        "common/etc/TeamColor.bin",
        Row::NotProduced("bins are Phase 4"),
    ),
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

fn compare_fpk(failures: &mut Vec<String>, name: &str, ours: &[u8], reference: &[u8]) {
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
    let run = sandbox.run(&settings, &["compile", exports_root.to_str().unwrap()]);
    assert_eq!(run.exit_code(), 0);
    for envelope in &run.events {
        if let PipelineEvent::Message(message) = &envelope.event {
            assert!(
                message.severity < Severity::Warning,
                "the tracer compiles with notes only: {message:?}"
            );
        }
    }

    let ours = archive(&output.join("4cc_90_test.cpk"));
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
                Some(our_bytes) => compare_fpk(&mut failures, path, our_bytes, reference_bytes),
            },
            Row::NotProduced(reason) => {
                if ours.contains_key(*path) {
                    failures.push(format!(
                        "tier4 {path}: produced in Phase 3 anyway ({reason})"
                    ));
                }
            }
        }
    }

    for path in ours.keys() {
        let produced = TABLE.iter().any(|(reference_path, row)| match row {
            Row::Relocated(our_path) => path == our_path,
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
