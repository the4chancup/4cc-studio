//! Parity against the compiled output for the same export, pre-Fox: compile
//! `fixtures/tracer_prefox/studio/jp Midcup Tracer` for PES 17 through the `compile` command
//! and compare the CPK with the reference tree in `fixtures/tracer_prefox/red/`, one explicit
//! row per reference entry, and per entry of the reference's nested face CPK, under the tiers
//! of `docs/plans/team_compiler/testing.md`.

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use base64::Engine as _;
use common::Sandbox;
use pes_model::format::mtl::{MaterialEntry, MaterialSet};
use studio_core::{PipelineEvent, Severity};

/// The player's face CPK, at the same path in both trees.
const FACE_CPK: &str = "common/character0/model/character/face/real/73120.cpk";
/// The directory the entries of the face CPK sit in.
const FACE_DIR: &str = "common/character0/model/character/face/real/73120/";
/// The player's textures: the reference keeps them in his face CPK, beside the `.mtl` files
/// naming them `./<name>`; the compiler relocates every texture of a player folder to his
/// common subfolder (`pipeline.md` step 6), at this `.mtl` path, packed under
/// `common/character1/`.
const RELOCATED: &str = "model/character/uniform/common/731/20 - Fumos/";

/// How one entry of the reference's outer CPK is compared.
enum Row {
    /// Tier 1: the same bytes at the same path.
    Exact,
    /// The reference's nested face CPK: each entry compared per `NESTED`.
    FaceCpk,
    /// Tier 2: a DDS with the same dimensions, codec and pixel payload (the reference copies
    /// the source file as it is, GIMP's reserved bytes and no mip flags; the compiler writes
    /// its header in one canonical form).
    Texture,
    /// Tier 2: the kit config, the same bytes outside the name fields of the four number
    /// textures (`kit_back`, `kit_chest`, `kit_leg`, `kit_name`), which the reference keeps
    /// from the member's config, respelled to the team, for textures the cut does not ship,
    /// and the compiler's encoder leaves empty: an absent texture has no name
    /// (`kit_config::encode_with_names`).
    KitConfig,
    /// Tier 2: `UniColor.bin`, compared per `compare_uni_color`.
    UniColor,
}

const OUTER: &[(&str, Row)] = &[
    (FACE_CPK, Row::FaceCpk),
    (
        "common/character0/model/character/uniform/team/731/731_DEF_GK1st_realUni.bin",
        Row::KitConfig,
    ),
    (
        "common/character0/model/character/uniform/team/UniColor.bin",
        Row::UniColor,
    ),
    (
        "common/character0/model/character/uniform/texture/u0731g1.dds",
        Row::Texture,
    ),
    // The kit has no mask of its own: both write Red's template, the compiler's bundled copy.
    (
        "common/character0/model/character/uniform/texture/u0731g1_mask.dds",
        Row::Exact,
    ),
    ("common/etc/TeamColor.bin", Row::Exact),
];

/// How one entry of the reference's face CPK is compared.
enum Nested {
    /// Tier 1: the same bytes under the name the compiler packs it as.
    Model(&'static str),
    /// Tier 2: the same materials once each `./` texture path names the relocated directory
    /// (the reference copies the file, unwrapping a WESYS one; the compiler writes it back
    /// through `MaterialSet`, so the text layout may differ).
    Mtl,
    /// Tier 2: `face.xml`, compared per `FACE_XML`.
    FaceXml,
    /// Tier 2: relocated to the common subfolder; the same dimensions, codec and pixel
    /// payload (the compiler writes a WESYS-wrapped source unwrapped and its header in one
    /// canonical form: mip count 1 with its flags).
    Texture,
    /// Tier 2: relocated, and encoded from an uncompressed source to the codec named (raster
    /// sources are block-compressed on PES 15-17, `pipeline.md` "Texture conversion"): the same
    /// dimensions.
    Encoded(&'static [u8; 4]),
}

const NESTED: &[(&str, Nested)] = &[
    // The member's own `face.xml` named the boots and gloves by their packed names already;
    // the face model's packed name gains the compiler's `oral_` prefix.
    (
        "face_high_win32.model",
        Nested::Model("oral_face_high_win32.model"),
    ),
    (
        "oral_boots_win32.model",
        Nested::Model("oral_boots_win32.model"),
    ),
    (
        "oral_glove_l_win32.model",
        Nested::Model("oral_glove_l_win32.model"),
    ),
    (
        "oral_glove_r_win32.model",
        Nested::Model("oral_glove_r_win32.model"),
    ),
    ("boots.mtl", Nested::Mtl),
    ("face.mtl", Nested::Mtl),
    ("glove_l.mtl", Nested::Mtl),
    ("glove_r.mtl", Nested::Mtl),
    ("face.xml", Nested::FaceXml),
    ("eye_occlusion.dds", Nested::Texture),
    ("face.dds", Nested::Texture),
    ("face_normal.dds", Nested::Texture),
    ("face_normal_detail.dds", Nested::Texture),
    ("face_specular_roughness.dds", Nested::Texture),
    ("glove_c.dds", Nested::Texture),
    ("glove_n.dds", Nested::Texture),
    ("glove_sr.dds", Nested::Texture),
    ("k2012_c.dds", Nested::Texture),
    ("k2012_n.dds", Nested::Texture),
    ("k2012_sr.dds", Nested::Encoded(b"DXT1")),
];

/// A `face.xml` `<model>` entry: its type, model path and material path.
type XmlEntry = (&'static str, &'static str, &'static str);

/// The reference's `face.xml` entries (the member's own file, which the reference keeps) and
/// the compiler's generated entry for each: type, model path, material path. The member typed
/// the boots `boots`; the generator types them `parts`, as the reference's own generator
/// does when it writes the file. The generator orders entries by model name.
const FACE_XML: &[(XmlEntry, XmlEntry)] = &[
    (
        ("face_neck", "./face_high_*.model", "./face.mtl"),
        ("face_neck", "./oral_face_high_*.model", "./face.mtl"),
    ),
    (
        ("gloveL", "./oral_glove_l_*.model", "./glove_l.mtl"),
        ("gloveL", "./oral_glove_l_*.model", "./glove_l.mtl"),
    ),
    (
        ("gloveR", "./oral_glove_r_*.model", "./glove_r.mtl"),
        ("gloveR", "./oral_glove_r_*.model", "./glove_r.mtl"),
    ),
    (
        ("boots", "./oral_boots_*.model", "./boots.mtl"),
        ("parts", "./oral_boots_*.model", "./boots.mtl"),
    ),
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

fn archive(bytes: &[u8]) -> BTreeMap<String, Vec<u8>> {
    let mut cpk = cpk::CpkArchive::open(std::io::Cursor::new(bytes)).unwrap();
    let entries = cpk.entries().to_vec();
    let mut files = BTreeMap::new();
    for entry in &entries {
        files.insert(entry.path.clone(), cpk.read(entry).unwrap());
    }
    files
}

/// The materials of a `.mtl`, each `./` path to one of the folder's textures made to name the
/// relocated directory. A path to a texture the folder does not hold stays as it is on both
/// sides (`face.mtl` names a `face_edithair_specular_roughness.dds` the member never shipped).
fn relocated_materials(bytes: &[u8]) -> Vec<pes_model::format::mtl::Material> {
    let mut set = MaterialSet::read(bytes).unwrap();
    for material in &mut set.materials {
        for entry in &mut material.entries {
            if let MaterialEntry::Sampler(sampler) = entry
                && let Some(name) = sampler.path.strip_prefix("./")
                && NESTED.iter().any(|(texture, row)| {
                    *texture == name && matches!(row, Nested::Texture | Nested::Encoded(_))
                })
            {
                sampler.path = format!("{RELOCATED}{name}");
            }
        }
    }
    set.materials
}

/// A DDS's width, height and codec (the four-character code, zeros when uncompressed) and
/// pixel payload, a WESYS-wrapped file unwrapped first.
fn dds_content(bytes: &[u8]) -> (u32, u32, [u8; 4], Vec<u8>) {
    let bytes = wezlib::decompress_if_wrapped(bytes).unwrap();
    assert_eq!(&bytes[..4], b"DDS ");
    let field = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
    let fourcc = bytes[84..88].try_into().unwrap();
    (field(16), field(12), fourcc, bytes[128..].to_vec())
}

/// The `<model>` entries of a `face.xml` (type, path, material) and its `<dif>` decoded.
fn face_xml(bytes: &[u8]) -> (Vec<(String, String, String)>, Vec<u8>) {
    let text = std::str::from_utf8(bytes).unwrap();
    let document = roxmltree::Document::parse(text).unwrap();
    let mut entries = Vec::new();
    let mut dif = Vec::new();
    for node in document
        .root_element()
        .children()
        .filter(|node| node.is_element())
    {
        match node.tag_name().name() {
            "model" => entries.push((
                node.attribute("type").unwrap().to_owned(),
                node.attribute("path").unwrap().to_owned(),
                node.attribute("material").unwrap().to_owned(),
            )),
            "dif" => {
                let encoded: String = node
                    .text()
                    .unwrap()
                    .chars()
                    .filter(|c| !c.is_whitespace())
                    .collect();
                dif = base64::engine::general_purpose::STANDARD
                    .decode(encoded)
                    .unwrap();
            }
            other => panic!("unexpected element {other}"),
        }
    }
    (entries, dif)
}

/// Compares our face CPK with the reference's, entry by entry per `NESTED`, the relocated
/// textures looked up in our outer CPK `ours`. Returns the outer paths it accounted for.
fn compare_face_cpk(
    failures: &mut Vec<String>,
    our_face: &[u8],
    reference_face: &[u8],
    ours: &BTreeMap<String, Vec<u8>>,
) -> Vec<String> {
    let our_entries = archive(our_face);
    let reference_entries = archive(reference_face);
    let mut accounted = Vec::new();
    let mut nested_accounted = Vec::new();
    for (name, row) in NESTED {
        let reference = &reference_entries[&format!("{FACE_DIR}{name}")];
        let at = |our_name: &str| our_entries.get(&format!("{FACE_DIR}{our_name}"));
        let relocated = format!("common/character1/{RELOCATED}{name}");
        match row {
            Nested::Model(our_name) => {
                nested_accounted.push(*our_name);
                match at(our_name) {
                    None => failures.push(format!("{name}: no {our_name} in our face CPK")),
                    Some(ours) if ours != reference => {
                        failures.push(format!("tier1 {name} -> {our_name}: bytes differ"));
                    }
                    Some(_) => {}
                }
            }
            Nested::Mtl => {
                nested_accounted.push(name);
                match at(name) {
                    None => failures.push(format!("{name}: missing from our face CPK")),
                    Some(ours) => {
                        let theirs = relocated_materials(reference);
                        let set = MaterialSet::read(ours).unwrap();
                        if set.materials != theirs {
                            failures.push(format!("tier2 {name}: materials differ"));
                        }
                    }
                }
            }
            Nested::FaceXml => {
                nested_accounted.push(name);
                match at(name) {
                    None => failures.push(format!("{name}: missing from our face CPK")),
                    Some(ours) => compare_face_xml(failures, ours, reference, &our_entries),
                }
            }
            Nested::Texture | Nested::Encoded(_) => {
                accounted.push(relocated.clone());
                let Some(our_bytes) = ours.get(&relocated) else {
                    failures.push(format!("{name}: no {relocated} in our CPK"));
                    continue;
                };
                let (width, height, codec, payload) = dds_content(our_bytes);
                let (ref_width, ref_height, ref_codec, ref_payload) = dds_content(reference);
                if (width, height) != (ref_width, ref_height) {
                    failures.push(format!(
                        "tier2 {name}: {width}x{height}, reference {ref_width}x{ref_height}"
                    ));
                }
                match row {
                    Nested::Encoded(expected) if codec != **expected => failures.push(format!(
                        "tier2 {name}: codec {codec:?}, expected {expected:?}"
                    )),
                    Nested::Texture if (codec, &payload) != (ref_codec, &ref_payload) => {
                        failures.push(format!("tier2 {name}: codec or pixels differ"));
                    }
                    _ => {}
                }
            }
        }
    }
    for path in our_entries.keys() {
        let name = path.strip_prefix(FACE_DIR).unwrap_or(path);
        if !nested_accounted.contains(&name) {
            failures.push(format!("our face CPK holds {path}, not in NESTED"));
        }
    }
    for path in reference_entries.keys() {
        let name = path.strip_prefix(FACE_DIR).unwrap_or(path);
        if !NESTED.iter().any(|(row_name, _)| *row_name == name) {
            failures.push(format!(
                "the reference's face CPK holds {path}, not in NESTED"
            ));
        }
    }
    accounted
}

/// Our `face.xml` against the reference's per `FACE_XML`: the same `<dif>` bytes, the mapped
/// entries in model-name order, and every model and material it names in our face CPK
/// (`*` standing for `win32`).
fn compare_face_xml(
    failures: &mut Vec<String>,
    ours: &[u8],
    reference: &[u8],
    our_entries: &BTreeMap<String, Vec<u8>>,
) {
    let (our_models, our_dif) = face_xml(ours);
    let (reference_models, reference_dif) = face_xml(reference);
    if our_dif != reference_dif {
        failures.push("tier2 face.xml: <dif> bytes differ".to_owned());
    }
    let owned = |(kind, path, material): (&str, &str, &str)| {
        (kind.to_owned(), path.to_owned(), material.to_owned())
    };
    let listed: Vec<_> = FACE_XML.iter().map(|(theirs, _)| owned(*theirs)).collect();
    if reference_models != listed {
        failures.push(format!(
            "face.xml: the reference's entries are {reference_models:?}"
        ));
    }
    let mut expected: Vec<_> = FACE_XML
        .iter()
        .map(|(_, generated)| owned(*generated))
        .collect();
    expected.sort_by(|a, b| a.1.cmp(&b.1));
    if our_models != expected {
        failures.push(format!(
            "tier2 face.xml: entries {our_models:?}, expected {expected:?}"
        ));
    }
    for (_, path, material) in &our_models {
        for named in [path.replace('*', "win32"), material.clone()] {
            let file = named.strip_prefix("./").unwrap_or(&named);
            if !our_entries.contains_key(&format!("{FACE_DIR}{file}")) {
                failures.push(format!("face.xml names {named}, not in our face CPK"));
            }
        }
    }
}

/// The kit config's name fields of the four number textures, `kit_back`, `kit_chest`, `kit_leg`
/// and `kit_name`, 16 bytes each, after the main texture's at 0x28 (`kit_config`'s format
/// table).
const NUMBER_TEXTURE_NAMES: std::ops::Range<usize> = 0x38..0x78;

/// Our kit config against the reference's per `Row::KitConfig`: the same bytes outside the
/// four number-texture names, which the compiler leaves empty for the textures the kit does
/// not ship and the reference keeps.
fn compare_kit_config(failures: &mut Vec<String>, ours: &[u8], reference: &[u8]) {
    let (start, end) = (NUMBER_TEXTURE_NAMES.start, NUMBER_TEXTURE_NAMES.end);
    if ours[..start] != reference[..start] || ours[end..] != reference[end..] {
        failures.push("tier2 kit config: bytes outside the number-texture names differ".to_owned());
    }
    if ours[NUMBER_TEXTURE_NAMES].iter().any(|byte| *byte != 0) {
        failures
            .push("tier2 kit config: a number texture the kit does not ship is named".to_owned());
    }
    if reference[NUMBER_TEXTURE_NAMES]
        .iter()
        .all(|byte| *byte == 0)
    {
        failures.push("the reference's kit config no longer keeps the absent textures' names: make the row Exact".to_owned());
    }
}

/// The kit number of the first goalkeeper kit, `g1`, in a `UniColor.bin` entry
/// (`resources/bins/README.md`: player kits count from 0, goalkeeper kits from 0x10).
const GK1_KIT_NUMBER: u8 = 0x10;

/// Team 731's 85-byte record in a `UniColor.bin`: the `u32` team ID, the kit count, then ten
/// 8-byte entries of kit number, icon and two colors (`resources/bins/README.md`).
const UNI_RECORD: std::ops::Range<usize> = (731 - 100) * 85..(731 - 100) * 85 + 85;

/// Our `UniColor.bin` against the reference's per `Row::UniColor`: the tracer's one kit, `g1`,
/// gives the same entry on both. The rest of our file is the bundled base's, team 731's past
/// cup's record included, where the reference was built on Red's fallback base, whose record
/// for the team was empty, so its record holds the one entry. Ours is therefore the base with
/// the reference's `g1` entry set in the team's record (`pipeline.md` "Bins accumulation": the
/// base's other kits are kept).
fn compare_uni_color(failures: &mut Vec<String>, ours: &[u8], reference: &[u8]) {
    let base = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../resources/bins/UniColor.bin"),
    )
    .unwrap();
    let entry_of = |bin: &[u8]| {
        bin[UNI_RECORD.start + 5..UNI_RECORD.end]
            .chunks(8)
            .position(|entry| entry[0] == GK1_KIT_NUMBER)
            .map(|index| UNI_RECORD.start + 5 + 8 * index)
    };
    let red_record = &reference[UNI_RECORD];
    if red_record[4] != 1 {
        failures.push(format!(
            "the reference's record holds {} kits, not the tracer's one",
            red_record[4]
        ));
    }
    let (Some(red_at), Some(base_at)) = (entry_of(reference), entry_of(&base)) else {
        failures
            .push("UniColor.bin: no g1 entry in the reference's or the base's record".to_owned());
        return;
    };
    let mut expected = base;
    expected[base_at..base_at + 8].copy_from_slice(&reference[red_at..red_at + 8]);
    if ours != expected {
        failures.push(
            "tier2 UniColor.bin: not the bundled base with the reference's g1 entry set in team 731's record"
                .to_owned(),
        );
    }
}

#[test]
fn the_pre_fox_tracer_matches_the_reference_tree() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tracer_prefox");
    let sandbox = Sandbox::new("parity_prefox");
    let output = sandbox.root.join("parity output");
    let settings = format!(
        "[common]\npes_version = 17\n[team-compiler]\noutput_folder_path = '{}'\n",
        output.display()
    );
    let exports_root = fixture.join("studio");
    // Nothing to install into: the reference is the CPK itself.
    let run = sandbox.run(
        &settings,
        &["compile", "--no-deploy", exports_root.to_str().unwrap()],
    );
    assert_eq!(run.exit_code(), 0);
    let warnings: Vec<_> = run
        .events
        .iter()
        .filter_map(|envelope| match &envelope.event {
            PipelineEvent::Message(message) if message.severity >= Severity::Warning => {
                Some((message.code.code.to_string(), message.context.clone()))
            }
            _ => None,
        })
        .collect();
    // The one warning Red gave too: Fumos's `face.mtl` names a texture the folder does not
    // hold on a material its face binds, and the face played a cup as compiled, which is why
    // `mtl_texture_not_found` is a Warning on pre-Fox (`messages.md`, the paragraph after
    // "Texture existence is checked **deep**").
    let fumos_texture = (
        "mtl_texture_not_found".to_owned(),
        vec![
            ("file".to_owned(), "face.mtl".to_owned()),
            (
                "texture".to_owned(),
                "./face_edithair_specular_roughness.dds".to_owned(),
            ),
            ("materials".to_owned(), "head_phong".to_owned()),
        ],
    );
    assert_eq!(
        warnings,
        [fumos_texture],
        "the pre-Fox tracer compiles with notes and Red's one warning"
    );

    let ours = archive(&fs::read(output.join("4cc_99_test.cpk")).unwrap());
    let reference = tree(&fixture.join("red"));

    let mut failures = Vec::new();
    let mut accounted: Vec<String> = Vec::new();
    for (path, row) in OUTER {
        let reference_bytes = &reference[*path];
        accounted.push((*path).to_owned());
        let Some(our_bytes) = ours.get(*path) else {
            failures.push(format!("{path}: missing from our CPK"));
            continue;
        };
        match row {
            Row::Exact => {
                if our_bytes != reference_bytes {
                    failures.push(format!("tier1 {path}: bytes differ"));
                }
            }
            Row::FaceCpk => accounted.extend(compare_face_cpk(
                &mut failures,
                our_bytes,
                reference_bytes,
                &ours,
            )),
            Row::Texture => {
                let (width, height, codec, payload) = dds_content(our_bytes);
                let (ref_width, ref_height, ref_codec, ref_payload) = dds_content(reference_bytes);
                if (width, height) != (ref_width, ref_height) {
                    failures.push(format!(
                        "tier2 {path}: {width}x{height}, reference {ref_width}x{ref_height}"
                    ));
                }
                if (codec, payload) != (ref_codec, ref_payload) {
                    failures.push(format!("tier2 {path}: codec or pixels differ"));
                }
            }
            Row::KitConfig => compare_kit_config(&mut failures, our_bytes, reference_bytes),
            Row::UniColor => compare_uni_color(&mut failures, our_bytes, reference_bytes),
        }
    }
    for path in ours.keys() {
        if !accounted.contains(path) {
            failures.push(format!("our CPK holds {path}, not in the tables' rows"));
        }
    }
    for path in reference.keys() {
        if !OUTER.iter().any(|(row_path, _)| path == row_path) {
            failures.push(format!("reference holds {path}, which OUTER does not list"));
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
